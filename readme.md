Convert floating point values to integer types faster than the standard `as` operator.

See the [library documentation](https://docs.rs/fast-float-to-integer) for documentation targeting users of the library.

---

# Development

We use the [xtask](https://github.com/matklad/cargo-xtask) pattern to implement automation tasks in Rust rather than shell scripts. This provides an easy way to compile for different targets and run the tests through qemu.

CI enforces that all targets compile, pass tests, and that the generated assembly committed to the repository is up to date.

# Releasing

- Update the changelog.
- Update the version in Cargo.toml.
- Create a git tag for the version.

# Improvements

## More targets

We should add common targets like aarch64.

## AVX512

AVX512 can convert float to u64 in [one instruction](https://www.felixcloutier.com/x86/vcvttps2udq), but the intrinsics are [not stable](https://github.com/rust-lang/rust/issues/111137).

We should make sure that AVX512 is actually faster in practice than the current approach.

## Cross compilation

The current cross compilation setup is brittle. It assume the host is x86 and that all the targets are x86 variants. This breaks for other architectures like aarch64 that need a custom linker. See the following links for more information:

- https://rust-lang.github.io/rustup/cross-compilation.html
- https://github.com/japaric/rust-cross/blob/master/README.md#c-cross-toolchain
- https://github.com/cross-rs/cross

We should improve this setup. Either setup the linking tools manually or use cargo cross.
