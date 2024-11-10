// There is an inner module to separate the implementation from the interface.

macro_rules! create_function {
    ($name:ident, $Input:ty, $Output: ty) => {
        #[inline(always)]
        pub fn $name(float: $Input) -> $Output {
            float as _
        }
    };
}

pub mod implementation {
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
}
