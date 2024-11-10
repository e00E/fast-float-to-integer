//! Convert floating point values to integer types faster than the standard `as` operator.
//!
//! The standard way of converting floating point values to integers is with the [`as` operator](https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions). This conversion has various guarantees as listed in the reference. One of them is that it saturates: Input values out of range of the output type convert to the minimal/maximal value of the output type.
//!
//! ```
//! assert_eq!(300f32 as u8, 255);
//! assert_eq!(-5f32 as u8, 0);
//! ```
//!
//! This contrasts C/C++, where this kind of cast is [undefined behavior](https://github.com/e00E/cpp-clamp-cast). Saturation comes with a downside. It is slower than the C/C++ version. On many [hardware targets](https://doc.rust-lang.org/nightly/rustc/platform-support.html) a float to integer conversion can be done in one instruction. For example [`CVTTSS2SI`](https://www.felixcloutier.com/x86/cvttss2si) on x86_84+SSE. Rust has to do more work than this, because the instruction does not provide saturation.
//!
//! Sometimes you want faster conversions and don't need saturation. This is what this crate provides. The behavior of the conversion functions in this crate depends on whether the input value is in range of the output type. If in range, then the conversion functions work like the standard `as` operator conversion. If not in range (including NaN), then you get an unspecified value.
//!
//! You never get undefined behavior but you can get unspecified behavior. In the unspecified case, you get an arbitrary value. The function returns and you get a valid value of the output type, but there is no guarantee what that value is.
//!
//! # Targets
//!
//! This crate picks an implementation automatically at compile time based on the [target](https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch) and [features](https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute). If there is no specialized implementation, then this crate picks the standard `as` operator conversion. This crate has optimized implementations on the following targets:
//!
//! - `target_arch = "x86_64", target_feature = "sse"`: all conversions except 128 bit integers
//! - `target_arch = "x86", target_feature = "sse"`: all conversions except 64 bit and 128 bit integers
//!
//! # to_int_unchecked
//!
//! The functions in this crate are similar to the std's [`to_int_unchecked`](f32::to_int_unchecked). The difference is that `to_int_unchecked` is unsafe. You need to ensure the input is in range for the output before calling the function. This is similar to the C/C++ behavior. This crate on the other hand provides a safe function while compiling to the same assembly as the unsafe function.
//!
//! # Assembly
//!
//! The [repository](https://github.com/e00E/fast-float-to-integer) contains generated assembly for every conversion and target. Here are some typical examples on x86_64+SSE.
//!
//
// We could do something like `#![doc = include_str!("../generated assembly/x86_64_default/f32_to_i64")]` to include the assembly directly. The downside of that is that compiling the library requires the assembly file to be there and we have to publish the file.
//
//! standard:
//!
//! ```asm
//! f32_to_i64:
//!     cvttss2si rax, xmm0
//!     ucomiss xmm0, dword ptr [rip + .L_0]
//!     movabs rcx, 9223372036854775807
//!     cmovbe rcx, rax
//!     xor eax, eax
//!     ucomiss xmm0, xmm0
//!     cmovnp rax, rcx
//!     ret
//! ```
//!
//! fast:
//!
//! ```asm
//! f32_to_i64:
//!     cvttss2si rax, xmm0
//!     ret
//! ```
//!
//! standard:
//!
//! ```asm
//! f32_to_u64:
//!     cvttss2si rax, xmm0
//!     mov rcx, rax
//!     sar rcx, 63
//!     movaps xmm1, xmm0
//!     subss xmm1, dword ptr [rip + .L_0]
//!     cvttss2si rdx, xmm1
//!     and rdx, rcx
//!     or rdx, rax
//!     xor ecx, ecx
//!     xorps xmm1, xmm1
//!     ucomiss xmm0, xmm1
//!     cmovae rcx, rdx
//!     ucomiss xmm0, dword ptr [rip + .L_1]
//!     mov rax, -1
//!     cmovbe rax, rcx
//!     ret
//! ```
//!
//! fast:
//!
//! ```asm
//! f32_to_u64:
//!     cvttss2si rcx, xmm0
//!     addss xmm0, dword ptr [rip + .L_0]
//!     cvttss2si rdx, xmm0
//!     mov rax, rcx
//!     sar rax, 63
//!     and rax, rdx
//!     or rax, rcx
//!     ret
//! ```

#![cfg_attr(not(test), no_std)]

/// Raise two to some power.
///
/// This function exists because libcore does not provide the [`f32::powi]`] family of functions.
#[allow(dead_code)]
const fn power_of_two_f32(exponent: u32) -> f32 {
    (2u128).pow(exponent) as f32
}

/// Like power_of_two_f32 but for f64.
#[allow(dead_code)]
const fn power_of_two_f64(exponent: u32) -> f64 {
    (2u128).pow(exponent) as f64
}

macro_rules! create_target {
    ($name:ident) => {
        use $name as active_target;

        // Create a test with the target name so we can check that the expected target is active. The following command prints the active target through the test name:
        //
        // cargo test --quiet --package fast-float-to-integer --lib -- --list
        #[test]
        fn $name() {}
    };
}

// Conditionally compiled target specific modules.The condition is set based on the availability of the intrinsics they use. This makes it safe to use the module. See the `default` module for the interface.
//
// We would put the mod declaration inside of the create_target macro too, but then rustfmt does not understand it.
cfg_if::cfg_if! {
    if #[cfg(feature = "force-default")] {
        mod target_default;
        create_target!(target_default);
    } else if #[cfg(all(target_arch = "x86_64", target_feature = "sse"))] {
        mod target_x86_64_sse;
        create_target!(target_x86_64_sse);
    } else if #[cfg(all(target_arch = "x86", target_feature = "sse"))] {
        mod target_x86_sse;
        create_target!(target_x86_sse);
    } else {
        mod target_default;
        create_target!(target_default);
    }
}

macro_rules! create_function {
    ($name:ident, $Float:ty, $Integer:ty) => {
        /// Convert the input floating point value to the output integer type.
        ///
        /// If the input value is out of range of the output type, then the result is unspecified. Otherwise, the result is the same as the standard `as` conversion.
        #[cfg_attr(feature = "show-asm", inline(never))]
        #[cfg_attr(not(feature = "show-asm"), inline(always))]
        pub fn $name(float: $Float) -> $Integer {
            active_target::implementation::$name(float)
        }
    };
}

create_function! {f32_to_i8, f32, i8}
create_function! {f32_to_u8, f32, u8}
create_function! {f32_to_i16, f32, i16}
create_function! {f32_to_u16, f32, u16}
create_function! {f32_to_i32, f32, i32}
create_function! {f32_to_u32, f32, u32}
create_function! {f32_to_i64, f32, i64}
create_function! {f32_to_u64, f32, u64}
create_function! {f32_to_i128, f32, i128}
create_function! {f32_to_u128, f32, u128}

create_function! {f64_to_i8, f64, i8}
create_function! {f64_to_u8, f64, u8}
create_function! {f64_to_i16, f64, i16}
create_function! {f64_to_u16, f64, u16}
create_function! {f64_to_i32, f64, i32}
create_function! {f64_to_u32, f64, u32}
create_function! {f64_to_i64, f64, i64}
create_function! {f64_to_u64, f64, u64}
create_function! {f64_to_i128, f64, i128}
create_function! {f64_to_u128, f64, u128}
