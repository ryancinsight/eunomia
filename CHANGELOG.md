# Changelog

All notable changes to Eunomia are documented here.

## [Unreleased]

### Added

- E-022: `convert` module — `convert::{narrow, widen}`, a generic
  const-parameterized (`<const E, const M>`) IEEE-754 narrowing/widening kernel
  (round-to-nearest-ties-to-even, subnormals, infinity/NaN, `f32`-subnormal
  handling). The native conversion SSOT replacing `half`'s `f16`/`bf16`
  conversions; verified bit-exact against `half`. See ADR 0003.

## [0.2.0] - 2026-07-18

### Breaking

- E-021 removes the foreign `num_traits::Zero` and `num_traits::One`
  implementations from `Complex<T>`.

### Migration

- Use `Complex::ZERO` and `Complex::ONE`, or the generic
  `ComplexField::zero()` and `ComplexField::one()` identities.

### Changed

- Pin `Complex32` and `Complex64` size, alignment, and field offsets at compile
  time for GPU and FFI consumers.
- Align the optional NumPy/PyO3 element boundary with version 0.29.
