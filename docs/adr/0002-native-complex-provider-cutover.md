# ADR 0002: Native complex provider cutover

- Status: Accepted
- Date: 2026-07-18
- Class: [arch]

## Context

Eunomia owns Atlas scalar vocabulary, but `Complex<T>` still implements
`num_traits::Zero` and `num_traits::One`. Those foreign implementations are the
crate's only direct `num-traits` requirement. Hephaestus also uses
`num_complex::Complex<f32>` in device-buffer APIs even though
`eunomia::Complex<T>` is already the Leto eigenvalue type and is `repr(C)` plus
`bytemuck::Pod`.

The optional Eunomia NumPy element implementation targets NumPy/PyO3 0.27,
while Hephaestus targets 0.29. Rust trait identity is version-specific, so the
0.27 implementation cannot provide a zero-copy `PyArray` element to the 0.29
consumer.

## Decision

- Remove the direct `num-traits` dependency and the foreign `Zero`/`One`
  implementations. Native identities remain `Complex::ZERO`,
  `Complex::ONE`, and the `ComplexField` defaults.
- Pin `Complex32` and `Complex64` size, alignment, and field offsets with
  compile-time assertions. These invariants discharge typed GPU-buffer and
  NumPy dtype layout requirements.
- Align the optional NumPy/PyO3 dependencies to 0.29 and retain the direct
  `numpy::Element` implementation for `Complex32` and `Complex64`.
- Release this pre-1.0 breaking change as Eunomia 0.2.0 and update direct stack
  consumers without compatibility adapters.

## Alternatives rejected

- Retain `num-traits` as a compatibility dependency: rejected because it keeps
  the replaced provider in the foundation crate and duplicates native identity
  ownership.
- Cast between `num_complex::Complex` and `eunomia::Complex` at consumer
  boundaries: rejected because it preserves dual type ownership and adds
  conversion or reinterpretation seams.
- Reconstruct NumPy values in Hephaestus: rejected because Eunomia can satisfy
  the NumPy element contract directly with the same ABI.

## Consequences

- Generic code using the foreign identity traits must migrate to Eunomia's
  constants or `ComplexField`.
- Direct device and Python buffer paths can use one `eunomia::Complex` type
  without intermediate allocation.
- NumPy and differential-oracle dependencies may still contain transitive
  `num-complex`; those edges are external interop/test implementation details,
  not Atlas production vocabulary.
