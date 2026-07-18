# Changelog

All notable changes to Eunomia are documented here.

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
