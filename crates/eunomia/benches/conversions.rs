//! Reduced-precision conversion throughput baselines.
//!
//! Measures reduced-precision conversions, arithmetic, and packed unpack.
//! Comparisons against the former f32 arithmetic roundtrip expose the cost of
//! exact destination-format operations without changing production paths.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use eunomia::{unpack_f8_to_f32, Bf16, F16, F32, F8};
use std::time::Duration;

const N: usize = 4096;

fn bench_f16_widen(c: &mut Criterion) {
    let src: Vec<F16> = (0..N)
        .map(|i| {
            F16(u16::try_from(i)
                .expect("invariant: benchmark index fits u16")
                .wrapping_mul(7)
                .wrapping_add(1))
        })
        .collect();
    let mut group = c.benchmark_group("f16_widen");
    group.throughput(Throughput::Elements(
        u64::try_from(N).expect("invariant: benchmark size fits u64"),
    ));
    group.bench_function("bulk_f16c", |b| {
        let mut dst = vec![0.0f32; N];
        b.iter(|| {
            F16::widen_slice(black_box(&src), black_box(&mut dst));
            black_box(&dst);
        });
    });
    group.bench_function("scalar_loop", |b| {
        let mut dst = vec![0.0f32; N];
        b.iter(|| {
            for (out, &s) in dst.iter_mut().zip(src.iter()) {
                *out = s.to_f32();
            }
            black_box(&dst);
        });
    });
    group.finish();
}

fn bench_f16_narrow(c: &mut Criterion) {
    let src: Vec<f32> = (0..N)
        .map(|i| {
            f32::from(u16::try_from(i).expect("invariant: benchmark index fits u16"))
                .mul_add(0.013, -20.0)
        })
        .collect();
    let mut group = c.benchmark_group("f16_narrow");
    group.throughput(Throughput::Elements(
        u64::try_from(N).expect("invariant: benchmark size fits u64"),
    ));
    group.bench_function("bulk_f16c", |b| {
        let mut dst = vec![F16::default(); N];
        b.iter(|| {
            F16::narrow_slice(black_box(&src), black_box(&mut dst));
            black_box(&dst);
        });
    });
    group.bench_function("scalar_loop", |b| {
        let mut dst = vec![F16::default(); N];
        b.iter(|| {
            for (out, &s) in dst.iter_mut().zip(src.iter()) {
                *out = F16::from_f32(s);
            }
            black_box(&dst);
        });
    });
    group.finish();
}

fn bench_bf16_widen(c: &mut Criterion) {
    let src: Vec<Bf16> = (0..N)
        .map(|i| {
            Bf16(
                u16::try_from(i)
                    .expect("invariant: benchmark index fits u16")
                    .wrapping_mul(7)
                    .wrapping_add(1),
            )
        })
        .collect();
    let mut group = c.benchmark_group("bf16_widen");
    group.throughput(Throughput::Elements(
        u64::try_from(N).expect("invariant: benchmark size fits u64"),
    ));
    group.bench_function("bulk", |b| {
        let mut dst = vec![0.0f32; N];
        b.iter(|| {
            Bf16::widen_slice(black_box(&src), black_box(&mut dst));
            black_box(&dst);
        });
    });
    group.bench_function("scalar_loop", |b| {
        let mut dst = vec![0.0f32; N];
        b.iter(|| {
            for (out, &s) in dst.iter_mut().zip(src.iter()) {
                *out = s.to_f32();
            }
            black_box(&dst);
        });
    });
    group.finish();
}

fn bench_bf16_narrow(c: &mut Criterion) {
    let src: Vec<f32> = (0..N)
        .map(|i| {
            f32::from(u16::try_from(i).expect("invariant: benchmark index fits u16"))
                .mul_add(0.013, -20.0)
        })
        .collect();
    let mut group = c.benchmark_group("bf16_narrow");
    group.throughput(Throughput::Elements(
        u64::try_from(N).expect("invariant: benchmark size fits u64"),
    ));
    group.bench_function("bulk", |b| {
        let mut dst = vec![Bf16::default(); N];
        b.iter(|| {
            Bf16::narrow_slice(black_box(&src), black_box(&mut dst));
            black_box(&dst);
        });
    });
    group.bench_function("scalar_loop", |b| {
        let mut dst = vec![Bf16::default(); N];
        b.iter(|| {
            for (out, &s) in dst.iter_mut().zip(src.iter()) {
                *out = Bf16::from_f32(s);
            }
            black_box(&dst);
        });
    });
    group.finish();
}

fn bench_packed_unpack(c: &mut Criterion) {
    let packed: Vec<F8> = (0..N)
        .map(|i| {
            F8(u8::try_from(i % (usize::from(u8::MAX) + 1))
                .expect("invariant: reduced benchmark index fits u8")
                .wrapping_mul(3))
        })
        .collect();
    let mut group = c.benchmark_group("packed_unpack_f8");
    group.throughput(Throughput::Elements(
        u64::try_from(N).expect("invariant: benchmark size fits u64"),
    ));
    group.bench_function("unpack_f8_to_f32", |b| {
        let mut dst = vec![F32::default(); N];
        b.iter(|| {
            unpack_f8_to_f32(black_box(&packed), black_box(&mut dst));
            black_box(&dst);
        });
    });
    group.finish();
}

fn reduced_inputs<T>(encode: impl Fn(f32) -> T) -> Vec<(T, T)> {
    (0_u32..8192)
        .map(|index| {
            let left = 1.0
                + f32::from(
                    u16::try_from(index % 97).expect("invariant: benchmark value fits u16"),
                ) / 32.0;
            let right = 1.0
                + f32::from(
                    u16::try_from(index % 31).expect("invariant: benchmark value fits u16"),
                ) / 8.0;
            (encode(left), encode(right))
        })
        .collect()
}

fn bench_reduced_operation<
    T: Copy,
    Current: Fn(T, T) -> T + Copy,
    Previous: Fn(T, T) -> T + Copy,
>(
    criterion: &mut Criterion,
    format: &str,
    operation: &str,
    inputs: &[(T, T)],
    current: Current,
    previous: Previous,
) {
    for size in [64, 8192] {
        let pairs = &inputs[..size];
        let mut group =
            criterion.benchmark_group(format!("reduced_arithmetic_{format}_{operation}_{size}"));
        group.sample_size(10);
        group.warm_up_time(Duration::from_secs(1));
        group.measurement_time(Duration::from_secs(3));
        group.throughput(Throughput::Elements(
            u64::try_from(size).expect("invariant: benchmark size fits u64"),
        ));
        group.bench_function("integer_kernel", |bench| {
            let mut output = vec![pairs[0].0; size];
            bench.iter(|| {
                for ((left, right), result) in pairs.iter().copied().zip(output.iter_mut()) {
                    *result = current(black_box(left), black_box(right));
                }
                black_box(&output);
            });
        });
        group.bench_function("f32_roundtrip", |bench| {
            let mut output = vec![pairs[0].0; size];
            bench.iter(|| {
                for ((left, right), result) in pairs.iter().copied().zip(output.iter_mut()) {
                    *result = previous(black_box(left), black_box(right));
                }
                black_box(&output);
            });
        });
        group.finish();
    }
}

fn bench_reduced_arithmetic(c: &mut Criterion) {
    let f16 = reduced_inputs(F16::from_f32);
    bench_reduced_operation(
        c,
        "f16",
        "add",
        &f16,
        |a, b| a + b,
        |a, b| F16::from_f32(a.to_f32() + b.to_f32()),
    );
    bench_reduced_operation(
        c,
        "f16",
        "mul",
        &f16,
        |a, b| a * b,
        |a, b| F16::from_f32(a.to_f32() * b.to_f32()),
    );
    bench_reduced_operation(
        c,
        "f16",
        "div",
        &f16,
        |a, b| a / b,
        |a, b| F16::from_f32(a.to_f32() / b.to_f32()),
    );
    bench_reduced_operation(
        c,
        "f16",
        "rem",
        &f16,
        |a, b| a % b,
        |a, b| F16::from_f32(a.to_f32() % b.to_f32()),
    );

    let bf16 = reduced_inputs(Bf16::from_f32);
    bench_reduced_operation(
        c,
        "bf16",
        "add",
        &bf16,
        |a, b| a + b,
        |a, b| Bf16::from_f32(a.to_f32() + b.to_f32()),
    );
    bench_reduced_operation(
        c,
        "bf16",
        "mul",
        &bf16,
        |a, b| a * b,
        |a, b| Bf16::from_f32(a.to_f32() * b.to_f32()),
    );
    bench_reduced_operation(
        c,
        "bf16",
        "div",
        &bf16,
        |a, b| a / b,
        |a, b| Bf16::from_f32(a.to_f32() / b.to_f32()),
    );
    bench_reduced_operation(
        c,
        "bf16",
        "rem",
        &bf16,
        |a, b| a % b,
        |a, b| Bf16::from_f32(a.to_f32() % b.to_f32()),
    );
}

criterion_group!(
    benches,
    bench_f16_widen,
    bench_f16_narrow,
    bench_bf16_widen,
    bench_bf16_narrow,
    bench_packed_unpack,
    bench_reduced_arithmetic
);
criterion_main!(benches);
