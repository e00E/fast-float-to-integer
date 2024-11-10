use core::arch::x86_64::{_mm_cvttsd_si64, _mm_cvttss_si64, _mm_loadu_pd, _mm_loadu_ps};

use crate::{power_of_two_f32, power_of_two_f64};

/// Convert f32 to i64 using the CVTTSS2SI instruction. If the input f32 is out of range of the output i64, then the result is i64::MIN.
#[inline(always)]
fn f32_to_i64(float: f32) -> i64 {
    // The compiler optimizes this function into a single instruction without the need for inline assembly.

    let floats = [float, 0., 0., 0.];
    let floats_pointer = floats.as_ptr();
    let floats_register = unsafe { _mm_loadu_ps(floats_pointer) };
    unsafe { _mm_cvttss_si64(floats_register) }
}

// For f32_to_i32 we could use CVTTSS2SI with 32 bit output (_mm_cvttss_si64) instead of the 64 bit output. That might be faster.

// We can't use the same approach for u64 output because the conversion instruction only works on i64. This is a problem for floats that exceed i64::MAX. We cannot handle this with one instruction, but we can still do better than the as operator.

// This approach branches into a special case if the input is too large. The branchless approach below is faster and is the one we use. We keep this code around for documentation purposes.
#[inline(always)]
fn _f32_to_u64_branchful(float: f32) -> u64 {
    const THRESHOLD_FLOAT: f32 = power_of_two_f32(63);
    const THRESHOLD_INTEGER: u64 = 2u64.pow(63);

    let in_range = float <= THRESHOLD_FLOAT;
    if in_range {
        f32_to_i64(float) as u64
    } else {
        // Subtract the threshold from the float. The result is >= 0 because the input is larger than the subtrahend. The result is <= i64::MAX because `u64::MAX - i64::MAX == i64::MAX`.
        let in_range_float = float - THRESHOLD_FLOAT;
        let integer = f32_to_i64(in_range_float) as u64;
        // Overflow is benign because it can only occur for invalid inputs.
        integer.overflowing_add(THRESHOLD_INTEGER).0
    }
}

// This approach avoids the branch. It is faster than the branchful approach.
#[inline(always)]
fn f32_to_u64_branchless(float: f32) -> u64 {
    const THRESHOLD: f32 = power_of_two_f32(63);

    let integer1 = f32_to_i64(float);
    let integer2 = f32_to_i64(float - THRESHOLD);
    // If the input is larger than i64::MAX, then integer1 is i64::MIN. This value has 1 as the leftmost bit and 0 as the remaining bits. Right shift on signed values is arithmetic, not logical [1]. We end up with all 0 (in range) or all 1 (out of range).
    let too_large = integer1 >> 63;
    // # If the input is not too large:
    //
    // Integer1 has the correct value. The mask is all 0, which makes the Or result in integer1.
    //
    // # If the input is too large:
    //
    // Integer1 is i64::MIN and the mask is all 1. The Or results in `i64::MIN | integer2`. integer2 has the correct result minus 2**63. This is the correct result without the leftmost bit. The Or adds the missing leftmost bit back.
    (integer1 | (integer2 & too_large)) as u64

    // [1] https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
}

#[inline(always)]
fn f32_to_u64(float: f32) -> u64 {
    f32_to_u64_branchless(float)
}

// Repeat for f64.

#[inline(always)]
fn f64_to_i64(float: f64) -> i64 {
    // see convert_f32

    let floats = [float, 0.];
    let floats_pointer = floats.as_ptr();
    let floats_register = unsafe { _mm_loadu_pd(floats_pointer) };
    unsafe { _mm_cvttsd_si64(floats_register) }
}

#[inline(always)]
fn f64_to_u64(float: f64) -> u64 {
    // see f32_to_u64

    const THRESHOLD: f64 = power_of_two_f64(63);

    let integer1 = f64_to_i64(float);
    let integer2 = f64_to_i64(float - THRESHOLD);
    let too_large = integer1 >> 63;
    (integer1 | (integer2 & too_large)) as u64
}

pub mod implementation {
    #[inline(always)]
    pub fn f32_to_i8(float: f32) -> i8 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u8(float: f32) -> u8 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i16(float: f32) -> i16 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u16(float: f32) -> u16 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i32(float: f32) -> i32 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u32(float: f32) -> u32 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i64(float: f32) -> i64 {
        super::f32_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u64(float: f32) -> u64 {
        super::f32_to_u64(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i128(float: f32) -> i128 {
        float as _
    }

    #[inline(always)]
    pub fn f32_to_u128(float: f32) -> u128 {
        float as _
    }

    #[inline(always)]
    pub fn f64_to_i8(float: f64) -> i8 {
        f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u8(float: f64) -> u8 {
        super::f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i16(float: f64) -> i16 {
        super::f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u16(float: f64) -> u16 {
        super::f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i32(float: f64) -> i32 {
        super::f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u32(float: f64) -> u32 {
        super::f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i64(float: f64) -> i64 {
        super::f64_to_i64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u64(float: f64) -> u64 {
        super::f64_to_u64(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i128(float: f64) -> i128 {
        float as _
    }

    #[inline(always)]
    pub fn f64_to_u128(float: f64) -> u128 {
        float as _
    }
}
