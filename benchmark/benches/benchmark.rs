// Unfortunately, these benchmarks are noisy. There are significant differences in the measured performance based on random code permutation or running the benchmarks at different times or on different machines. The same function benchmarked twice can appear to have very different performance.
//
// We've changed some of the criterion settings to help with this, but the problem persists. It would be nice to have a more real world benchmark.

use criterion::{criterion_group, criterion_main, Criterion};
use fast_float_to_integer as ffti;
use std::{hint::black_box, time::Duration};

// We create a dependency between the converted numbers so that compiler or CPU cannot skip the computation.
macro_rules! create_benchmark {
    ($c:ident, $name:literal, $function:path, $Float:ty) => {
        let floats = [0 as $Float; 1_000];
        $c.bench_function($name, |b| {
            b.iter(|| {
                let mut result = 0;
                for float in black_box(floats.as_slice()) {
                    let converted = $function(*float);
                    result ^= converted;
                }
                black_box(result);
            })
        });
    };
}

pub fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex");
    group
        .sample_size(10_000)
        .measurement_time(Duration::from_secs_f32(1.0))
        .warm_up_time(Duration::from_secs_f32(0.1))
        .nresamples(1);

    create_benchmark! {group, "f32_to_i8_optimized", ffti::f32_to_i8, f32}
    create_benchmark! {group, "f32_to_u8_optimized", ffti::f32_to_u8, f32}
    create_benchmark! {group, "f32_to_i16_optimized", ffti::f32_to_i16, f32}
    create_benchmark! {group, "f32_to_u16_optimized", ffti::f32_to_u16, f32}
    create_benchmark! {group, "f32_to_i32_optimized", ffti::f32_to_i32, f32}
    create_benchmark! {group, "f32_to_u32_optimized", ffti::f32_to_u32, f32}
    create_benchmark! {group, "f32_to_i64_optimized", ffti::f32_to_i64, f32}
    create_benchmark! {group, "f32_to_u64_optimized", ffti::f32_to_u64, f32}
    create_benchmark! {group, "f32_to_i128_optimized", ffti::f32_to_i128, f32}
    create_benchmark! {group, "f32_to_u128_optimized", ffti::f32_to_u128, f32}

    create_benchmark! {group, "f64_to_i8_optimized", ffti::f64_to_i8, f64}
    create_benchmark! {group, "f64_to_u8_optimized", ffti::f64_to_u8, f64}
    create_benchmark! {group, "f64_to_i16_optimized", ffti::f64_to_i16, f64}
    create_benchmark! {group, "f64_to_u16_optimized", ffti::f64_to_u16, f64}
    create_benchmark! {group, "f64_to_i32_optimized", ffti::f64_to_i32, f64}
    create_benchmark! {group, "f64_to_u32_optimized", ffti::f64_to_u32, f64}
    create_benchmark! {group, "f64_to_i64_optimized", ffti::f64_to_i64, f64}
    create_benchmark! {group, "f64_to_u64_optimized", ffti::f64_to_u64, f64}
    create_benchmark! {group, "f64_to_i128_optimized", ffti::f64_to_i128, f64}
    create_benchmark! {group, "f64_to_u128_optimized", ffti::f64_to_u128, f64}
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
