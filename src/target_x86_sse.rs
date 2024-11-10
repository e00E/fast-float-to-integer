use core::arch::x86::{_mm_cvttsd_si32, _mm_cvttss_si32, _mm_loadu_pd, _mm_loadu_ps};

use crate::{power_of_two_f32, power_of_two_f64};

#[inline(always)]
fn f32_to_i32(float: f32) -> i32 {
    // see crate::x86_64_sse::f32_to_i64

    let floats = [float, 0., 0., 0.];
    let floats_pointer = floats.as_ptr();
    let floats_register = unsafe { _mm_loadu_ps(floats_pointer) };
    unsafe { _mm_cvttss_si32(floats_register) }
}

#[inline(always)]
fn f32_to_u32(float: f32) -> u32 {
    // see crate::x86_64_sse::f32_to_u64

    const THRESHOLD: f32 = power_of_two_f32(31);

    let integer1 = f32_to_i32(float);
    let integer2 = f32_to_i32(float - THRESHOLD);
    let too_large = integer1 >> 31;
    (integer1 | (integer2 & too_large)) as u32
}

#[inline(always)]
fn f64_to_i32(float: f64) -> i32 {
    // see crate::x86_64_sse::f64_to_i64

    let floats = [float, 0.];
    let floats_pointer = floats.as_ptr();
    let floats_register = unsafe { _mm_loadu_pd(floats_pointer) };
    unsafe { _mm_cvttsd_si32(floats_register) }
}

#[inline(always)]
fn f64_to_u32(float: f64) -> u32 {
    // see crate::x86_64_sse::f64_to_u64

    const THRESHOLD: f64 = power_of_two_f64(31);

    let integer1 = f64_to_i32(float);
    let integer2 = f64_to_i32(float - THRESHOLD);
    let too_large = integer1 >> 31;
    (integer1 | (integer2 & too_large)) as u32
}

pub mod implementation {
    #[inline(always)]
    pub fn f32_to_i8(float: f32) -> i8 {
        super::f32_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u8(float: f32) -> u8 {
        super::f32_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i16(float: f32) -> i16 {
        super::f32_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u16(float: f32) -> u16 {
        super::f32_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i32(float: f32) -> i32 {
        super::f32_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f32_to_u32(float: f32) -> u32 {
        super::f32_to_u32(float) as _
    }

    #[inline(always)]
    pub fn f32_to_i64(float: f32) -> i64 {
        float as _
    }

    #[inline(always)]
    pub fn f32_to_u64(float: f32) -> u64 {
        float as _
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
        super::f64_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u8(float: f64) -> u8 {
        super::f64_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i16(float: f64) -> i16 {
        super::f64_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u16(float: f64) -> u16 {
        super::f64_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i32(float: f64) -> i32 {
        super::f64_to_i32(float) as _
    }

    #[inline(always)]
    pub fn f64_to_u32(float: f64) -> u32 {
        super::f64_to_u32(float) as _
    }

    #[inline(always)]
    pub fn f64_to_i64(float: f64) -> i64 {
        float as _
    }

    #[inline(always)]
    pub fn f64_to_u64(float: f64) -> u64 {
        float as _
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
