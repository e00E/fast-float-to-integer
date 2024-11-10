// TODO: Consider rewriting this with traits instead of macros.

use float_next_after::NextAfter;

trait InRange<Integer> {
    /// Is this float value in range for this integer type?
    #[allow(clippy::wrong_self_convention)]
    fn in_range(self) -> bool;
}

macro_rules! implement_is_valid {
    ($Float:ty, $Integer:ty, $signed:expr) => {
        impl InRange<$Integer> for $Float {
            fn in_range(self) -> bool {
                let bits = <$Integer>::BITS as i32;
                let base: $Float = 2.;
                if $signed {
                    self >= -base.powi(bits - 1) && self < base.powi(bits - 1)
                } else {
                    self >= 0. && self < base.powi(bits)
                }
            }
        }
    };
}

implement_is_valid! {f32, i8, true}
implement_is_valid! {f32, u8, false}
implement_is_valid! {f32, i16, true}
implement_is_valid! {f32, u16, false}
implement_is_valid! {f32, i32, true}
implement_is_valid! {f32, u32, false}
implement_is_valid! {f32, i64, true}
implement_is_valid! {f32, u64, false}
implement_is_valid! {f32, i128, true}
implement_is_valid! {f32, u128, false}

implement_is_valid! {f64, i8, true}
implement_is_valid! {f64, u8, false}
implement_is_valid! {f64, i16, true}
implement_is_valid! {f64, u16, false}
implement_is_valid! {f64, i32, true}
implement_is_valid! {f64, u32, false}
implement_is_valid! {f64, i64, true}
implement_is_valid! {f64, u64, false}
implement_is_valid! {f64, i128, true}
implement_is_valid! {f64, u128, false}

// We can test all f32 values in 10 seconds on a modern processor. On qemu it is too slow.

macro_rules! create_all_f32_test {
    ($name:ident, $convert_custom:path, $Integer:ty) => {
        #[test]
        #[ignore]
        fn $name() {
            for i in u32::MIN..=u32::MAX {
                let float = f32::from_bits(i);
                let result = $convert_custom(float);
                let expected = float as $Integer;
                // We skip the assert but not the computation. This detects failing debug assertions in the implementation.
                if InRange::<$Integer>::in_range(float) {
                    assert_eq!(result, expected, "{float:.0}");
                }
            }
        }
    };
}

create_all_f32_test! {all_f32_i8, fast_float_to_integer::f32_to_i8, i8}
create_all_f32_test! {all_f32_u8, fast_float_to_integer::f32_to_u8, u8}
create_all_f32_test! {all_f32_i16, fast_float_to_integer::f32_to_i16, i16}
create_all_f32_test! {all_f32_u16, fast_float_to_integer::f32_to_u16, u16}
create_all_f32_test! {all_f32_i32, fast_float_to_integer::f32_to_i32, i32}
create_all_f32_test! {all_f32_u32, fast_float_to_integer::f32_to_u32, u32}
create_all_f32_test! {all_f32_i64, fast_float_to_integer::f32_to_i64, i64}
create_all_f32_test! {all_f32_u64, fast_float_to_integer::f32_to_u64, u64}
create_all_f32_test! {all_f32_i128, fast_float_to_integer::f32_to_i128, i128}
create_all_f32_test! {all_f32_u128, fast_float_to_integer::f32_to_u128, u128}

macro_rules! create_interesting_floats_function {
    ($name:ident, $Float:ty) => {
        fn $name() -> impl Iterator<Item = $Float> {
            let signs = |float: $Float| [float, -float];

            let neighbors = |float: $Float| {
                [
                    float.next_after(<$Float>::INFINITY),
                    float
                        .next_after(<$Float>::INFINITY)
                        .next_after(<$Float>::INFINITY),
                    float.next_after(<$Float>::NEG_INFINITY),
                    float
                        .next_after(<$Float>::NEG_INFINITY)
                        .next_after(<$Float>::NEG_INFINITY),
                ]
            };

            let offsets = |float: $Float| [-2, -1, 0, 1, 2].map(|offset| float + offset as $Float);

            let exponents = 0..70;
            exponents.flat_map(move |exponent| {
                let float = (2 as $Float).powi(exponent);
                offsets(float)
                    .into_iter()
                    .chain(neighbors(float))
                    .chain([float * 1.5])
                    .flat_map(signs)
            })
        }
    };
}

create_interesting_floats_function! {interesting_floats_f32, f32}
create_interesting_floats_function! {interesting_floats_f64, f64}

#[test]
#[ignore]
fn print_interesting_floats() {
    for float in interesting_floats_f32() {
        println!("{float:.e} {float:.0} {:.x}", float.to_bits());
    }
}

macro_rules! create_interesting_floats_test {
    ($name:ident, $interesting_floats_function:ident, $convert_custom:path, $Integer:ty) => {
        #[test]
        fn $name() {
            let mut valid_count: u32 = 0;
            for float in $interesting_floats_function() {
                let result = $convert_custom(float);
                let expected = float as $Integer;
                if InRange::<$Integer>::in_range(float) {
                    valid_count += 1;
                    assert_eq!(result, expected, "{float:.0}");
                }
            }
            assert!((50..2000).contains(&valid_count), "{valid_count}");
        }
    };
}

create_interesting_floats_test! {interesting_f32_i8, interesting_floats_f32, fast_float_to_integer::f32_to_i8, i8}
create_interesting_floats_test! {interesting_f32_u8, interesting_floats_f32, fast_float_to_integer::f32_to_u8, u8}
create_interesting_floats_test! {interesting_f32_i16, interesting_floats_f32, fast_float_to_integer::f32_to_i16, i16}
create_interesting_floats_test! {interesting_f32_u16, interesting_floats_f32, fast_float_to_integer::f32_to_u16, u16}
create_interesting_floats_test! {interesting_f32_i32, interesting_floats_f32, fast_float_to_integer::f32_to_i32, i32}
create_interesting_floats_test! {interesting_f32_u32, interesting_floats_f32, fast_float_to_integer::f32_to_u32, u32}
create_interesting_floats_test! {interesting_f32_i64, interesting_floats_f32, fast_float_to_integer::f32_to_i64, i64}
create_interesting_floats_test! {interesting_f32_u64, interesting_floats_f32, fast_float_to_integer::f32_to_u64, u64}
create_interesting_floats_test! {interesting_f32_i128, interesting_floats_f32, fast_float_to_integer::f32_to_i128, i128}
create_interesting_floats_test! {interesting_f32_u128, interesting_floats_f32, fast_float_to_integer::f32_to_u128, u128}

create_interesting_floats_test! {interesting_f64_i8, interesting_floats_f64, fast_float_to_integer::f64_to_i8, i8}
create_interesting_floats_test! {interesting_f64_u8, interesting_floats_f64, fast_float_to_integer::f64_to_u8, u8}
create_interesting_floats_test! {interesting_f64_i16, interesting_floats_f64, fast_float_to_integer::f64_to_i16, i16}
create_interesting_floats_test! {interesting_f64_u16, interesting_floats_f64, fast_float_to_integer::f64_to_u16, u16}
create_interesting_floats_test! {interesting_f64_i32, interesting_floats_f64, fast_float_to_integer::f64_to_i32, i32}
create_interesting_floats_test! {interesting_f64_u32, interesting_floats_f64, fast_float_to_integer::f64_to_u32, u32}
create_interesting_floats_test! {interesting_f64_i64, interesting_floats_f64, fast_float_to_integer::f64_to_i64, i64}
create_interesting_floats_test! {interesting_f64_u64, interesting_floats_f64, fast_float_to_integer::f64_to_u64, u64}
create_interesting_floats_test! {interesting_f64_i128, interesting_floats_f64, fast_float_to_integer::f64_to_i128, i128}
create_interesting_floats_test! {interesting_f64_u128, interesting_floats_f64, fast_float_to_integer::f64_to_u128, u128}
