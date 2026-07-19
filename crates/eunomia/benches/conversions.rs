//! Reduced-precision conversion throughput baselines.
//!
//! Compares the F16C-accelerated bulk `f16` conversions against the scalar
//! kernel, and exercises the SIMD packed-unpack dispatch. Committed baselines
//! gate regressions per the performance policy.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use eunomia::{unpack_f8_to_f32, Bf16, F16, F32, F8};

const N: usize = 4096;

fn bench_f16_widen(c: &mut Criterion) {
    let src: Vec<F16> = (0..N)
        .map(|i| F16((i as u16).wrapping_mul(7).wrapping_add(1)))
        .collect();
    let mut group = c.benchmark_group("f16_widen");
    group.throughput(Throughput::Elements(N as u64));
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
    let src: Vec<f32> = (0..N).map(|i| (i as f32).mul_add(0.013, -20.0)).collect();
    let mut group = c.benchmark_group("f16_narrow");
    group.throughput(Throughput::Elements(N as u64));
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
        .map(|i| Bf16((i as u16).wrapping_mul(7).wrapping_add(1)))
        .collect();
    let mut group = c.benchmark_group("bf16_widen");
    group.throughput(Throughput::Elements(N as u64));
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
    let src: Vec<f32> = (0..N).map(|i| (i as f32).mul_add(0.013, -20.0)).collect();
    let mut group = c.benchmark_group("bf16_narrow");
    group.throughput(Throughput::Elements(N as u64));
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
    let packed: Vec<F8> = (0..N).map(|i| F8((i as u8).wrapping_mul(3))).collect();
    let mut group = c.benchmark_group("packed_unpack_f8");
    group.throughput(Throughput::Elements(N as u64));
    group.bench_function("unpack_f8_to_f32", |b| {
        let mut dst = vec![F32::default(); N];
        b.iter(|| {
            unpack_f8_to_f32(black_box(&packed), black_box(&mut dst));
            black_box(&dst);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_f16_widen,
    bench_f16_narrow,
    bench_bf16_widen,
    bench_bf16_narrow,
    bench_packed_unpack
);
criterion_main!(benches);
