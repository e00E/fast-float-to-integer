use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
    path::PathBuf,
    process::{Command, Output},
    sync::LazyLock,
};

use anyhow::{anyhow, Context, Result};
use regex::Regex;

fn main() -> Result<()> {
    let command = std::env::args()
        .nth(1)
        .context("missing command argument")?;
    let command = match command.as_str() {
        "check" => |target: &Target| check(target, false),
        "clippy" => |target: &Target| check(target, true),
        "target" => |target: &Target| expected_target(target),
        "test" => |target: &Target| qemu_test(target),
        "asm" => |target: &Target| show_asm(target),
        "all" => |target: &Target| {
            check(target, true).context("check")?;
            expected_target(target).context("target")?;
            qemu_test(target).context("test")?;
            show_asm(target).context("asm")?;
            Ok(())
        },
        _ => return Err(anyhow!("unknown command")),
    };

    for target in TARGETS {
        println!("Handling target {}.", target.name);
        install_rustup_target(target.rust_target).context("install rustup target")?;
        command(target)?;
    }

    Ok(())
}

struct Target {
    name: &'static str,
    rust_target: &'static str,
    expected_target_module: &'static str,
    feature: &'static str,
    qemu: &'static str,
    generate_assembly: bool,
    force_default: bool,
}

const TARGETS: &[Target] = &[
    Target {
        name: "x86_64_sse",
        rust_target: "x86_64-unknown-linux-gnu",
        expected_target_module: "x86_64_sse",
        feature: "+sse",
        qemu: "x86_64",
        generate_assembly: true,
        force_default: false,
    },
    Target {
        name: "x86_64_default",
        rust_target: "x86_64-unknown-linux-gnu",
        expected_target_module: "default",
        feature: "",
        qemu: "x86_64",
        generate_assembly: true,
        force_default: true,
    },
    Target {
        name: "x86_sse",
        rust_target: "i686-unknown-linux-gnu",
        expected_target_module: "x86_sse",
        feature: "+sse",
        qemu: "i386",
        generate_assembly: true,
        force_default: false,
    },
    Target {
        name: "default",
        rust_target: "i686-unknown-linux-gnu",
        expected_target_module: "default",
        feature: "-sse",
        qemu: "i386",
        generate_assembly: false,
        force_default: false,
    },
];

/// Convert a Command to a string representation you can paste in your terminal.
///
/// Assumes that the command does not run into tricky formatting edge cases with characters that need to be escaped.
fn command_to_string(command: &Command) -> String {
    fn string_is_not_tricky(string: &str) -> bool {
        string.chars().all(|char| {
            char.is_ascii_alphanumeric() || ['-', '_', '=', '/', '.', '+', ' '].contains(&char)
        })
    }

    fn handle_space(s: &str) -> Cow<str> {
        if s.contains(' ') {
            format!("\"{s}\"").into()
        } else {
            s.into()
        }
    }

    let mut string = String::new();

    let envs = command.get_envs();
    let has_envs = envs.len() > 0;
    if has_envs {
        write!(&mut string, "env").unwrap();
    }
    for (key, value) in envs {
        let key = key.to_str().unwrap();
        let value = value.unwrap_or_default().to_str().unwrap();
        assert!(string_is_not_tricky(key), "{key:?}");
        assert!(string_is_not_tricky(value), "{value:?}");
        let key = handle_space(key);
        let value = handle_space(value);
        write!(&mut string, " {key}={value}").unwrap();
    }
    if has_envs {
        write!(&mut string, " ").unwrap();
    }

    let program = command.get_program().to_str().unwrap();
    assert!(string_is_not_tricky(program), "{program:?}");
    let program = handle_space(program);
    write!(&mut string, "{program}").unwrap();

    for arg in command.get_args() {
        let arg = arg.to_str().unwrap();
        assert!(string_is_not_tricky(arg), "{arg:?}");
        let arg = handle_space(arg);
        write!(&mut string, " {}", arg).unwrap();
    }

    string
}

/// Run a command while checking status code and providing a better error message.
fn run_command(command: &mut Command) -> Result<Output> {
    let make_string = |command: &Command| format!("command: {}", command_to_string(command));
    let output = command
        .output()
        .context("command failed")
        .with_context(|| make_string(command))?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(output.stdout.as_slice());
        let stderr = String::from_utf8_lossy(output.stderr.as_slice());
        return Err(anyhow!("command status indicates error")
            .context(format!("command: {}", make_string(command)))
            .context(format!("stdout: {stdout}"))
            .context(format!("stderr: {stderr}")));
    }
    Ok(output)
}

fn install_rustup_target(target: &str) -> Result<()> {
    run_command(Command::new("rustup").args(["--quiet", "target", "add", target]))?;
    Ok(())
}

fn cargo_with_target(
    Target {
        rust_target: target,
        feature,
        ..
    }: &Target,
    subcommand: &str,
    rustflags: &[&str],
) -> Command {
    let mut flags = String::new();
    write!(&mut flags, "-Ctarget-feature={feature}").unwrap();
    for flag in rustflags {
        write!(&mut flags, " {flag}").unwrap();
    }
    let target_arg = format!("--target={target}");
    let mut command = Command::new("cargo");
    command
        .env("RUSTFLAGS", flags.as_str())
        .args([subcommand, target_arg.as_str()]);
    command
}

fn check(target: &Target, clippy: bool) -> Result<()> {
    let command = match clippy {
        true => "clippy",
        false => "check",
    };
    let features = if target.force_default {
        "--features=force-default"
    } else {
        "--features="
    };
    let mut command = cargo_with_target(target, command, &[]);
    command.args([
        "--quiet",
        "--frozen",
        "--package=fast-float-to-integer",
        "--all-targets",
        features,
    ]);
    if clippy {
        command.args(["--", "-D=warnings"]);
    }
    run_command(&mut command)?;
    Ok(())
}

fn show_asm(target: &Target) -> Result<()> {
    if !target.generate_assembly {
        return Ok(());
    }

    let functions = [
        "f32_to_i8",
        "f32_to_u8",
        "f32_to_i16",
        "f32_to_u16",
        "f32_to_i32",
        "f32_to_u32",
        "f32_to_i64",
        "f32_to_u64",
        "f32_to_i128",
        "f32_to_u128",
        "f64_to_i8",
        "f64_to_u8",
        "f64_to_i16",
        "f64_to_u16",
        "f64_to_i32",
        "f64_to_u32",
        "f64_to_i64",
        "f64_to_u64",
        "f64_to_i128",
        "f64_to_u128",
    ];

    let mut features = "--features=show-asm".to_owned();
    if target.force_default {
        features.push_str(",force-default");
    }

    for function in functions {
        let output = run_command(cargo_with_target(target, "asm", &[]).args([
            // "--quiet", // will be supported in next cargo asm release
            "--no-color",
            "--simplify",
            "--include-constants",
            "--package=fast-float-to-integer",
            "--lib",
            features.as_str(),
            "--profile=show-asm",
            function,
        ]))?;
        let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        let output = normalize_assembly(output);

        let mut path = PathBuf::new();
        path.push("generated assembly");
        path.push(target.name);
        std::fs::create_dir_all(&path).context("create_dir_all")?;
        path.push(function);
        std::fs::write(&path, output.as_ref()).context("write generated assembly")?;
    }

    Ok(())
}

/// We diff the generated assembly to make sure it doesn't accidentally change. This requires the assembly to be deterministic. By default, some parts of the assembly like labels are not deterministic. This function fixes that.
fn normalize_assembly(assembly: &str) -> Cow<str> {
    const REGEX: &str = r"\.L([[:alnum:]]|_)+";
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(REGEX).unwrap());

    let mut matches = RE.find_iter(assembly).peekable();
    if matches.peek().is_none() {
        return Cow::Borrowed(assembly);
    }

    let mut result = String::new();
    let mut labels = HashMap::<&str, usize>::new();
    let mut previous_match_end = 0usize;
    for label in matches {
        let mut label_index = labels.len();
        label_index = match labels.entry(label.as_str()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => *entry.insert(label_index),
        };

        let range = label.range();
        result.push_str(&assembly[previous_match_end..range.start]);
        write!(&mut result, ".L_{label_index}").unwrap();
        previous_match_end = range.end;
    }
    result.push_str(&assembly[previous_match_end..]);
    Cow::Owned(result)
}

#[test]
fn normalize_assembly_() {
    let input = "abcd";
    let expected = "abcd";
    let actual = normalize_assembly(input);
    assert_eq!(actual, expected);

    let input = "a .LCPI2_0 b .LCPI3_0 c .LCPI2_0 d";
    let expected = "a .L_0 b .L_1 c .L_0 d";
    let actual = normalize_assembly(input);
    assert_eq!(actual, expected);
}

fn qemu_test(target: &Target) -> Result<()> {
    let features = if target.force_default {
        "--features=force-default"
    } else {
        "--features="
    };
    let output = run_command(cargo_with_target(target, "test", &[]).args([
        "--frozen",
        "--no-run",
        "--package=fast-float-to-integer",
        "--test=test",
        features,
    ]))?;
    let stderr = std::str::from_utf8(output.stderr.as_slice()).context("output is not utf8")?;

    let test_binary_path = stderr
        .rsplit('\n')
        .nth(1)
        .context("unexpected output")?
        .strip_prefix("  Executable tests/test.rs (")
        .context("unexpected output")?
        .strip_suffix(')')
        .context("unexpected output")?;

    run_command(
        Command::new(format!("qemu-{}", target.qemu)).args([test_binary_path, "--test-threads=1"]),
    )?;

    Ok(())
}

fn expected_target(target: &Target) -> Result<()> {
    let features = if target.force_default {
        "--features=force-default"
    } else {
        "--features="
    };
    let output = run_command(cargo_with_target(target, "test", &[]).args([
        "--quiet",
        "--package=fast-float-to-integer",
        features,
        "--lib",
        "--",
        "--list",
    ]))?;
    let actual = std::str::from_utf8(output.stdout.as_slice())
        .context("output is not utf8")?
        .strip_prefix("target_")
        .context("unexpected stdout")?
        .strip_suffix(": test\n")
        .context("unexpected stdout")?;
    if actual != target.expected_target_module {
        return Err(anyhow!(
            "actual target {} does not match expected target {}",
            actual,
            target.expected_target_module,
        ));
    }
    Ok(())
}
