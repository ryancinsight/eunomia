# Changelog

All notable changes to Eunomia are documented here.

## [Unreleased]

### Added

- `FloatElement::Accumulator` — the associated type naming the format a
  reduction over `T` accumulates in, plus `to_accumulator`/`from_accumulator`
  to move values through it. Identity for `f32`/`f64`/`F32`/`F64`; `f32` for
  `F16`/`Bf16`/`Bf8`/`Bf4`/`F8`/`F4`, for which widening is exact (≤11-bit
  significand into 24) and necessary: the binding limit on a reduced-precision
  reduction is stagnation at `n ≈ 1/ε` (256 elements for `bf16`), not gradual
  accuracy loss. This is the crate's only sanctioned widening route — without
  it every downstream reduced-precision reduction must either lose precision or
  hand-roll a cast-compute-cast body, which is the fake-generic pattern the
  datatype law exists to forbid. Additive and non-breaking: `FloatElement` is
  sealed, so no out-of-crate implementor can exist to be broken.

- `FloatElement::cbrt` — the sign-preserving cube root, defined for all reals
  (`cbrt(-8) == -2`), unlike `powf(x, 1/3)` which is NaN for negative
  operands. The default routes through `libm::cbrtf` (the correct
  reduced-precision path for `F16`/`Bf16`); primitive `f64` and the `F64`
  wrapper override with native `libm::cbrt`. Added because Aequitas'
  `Quantity::cbrt` was rooting through `powf(1/3)` — this is the scalar-math
  SSOT home for the operation.

- `FloatElement::rsqrt` — the reciprocal square root `1 / √x`, computed as
  `sqrt(self).recip()` at native precision (no f32 widen-narrow). `NaN` for
  negative inputs. Consumers previously emulated it as `powf(x, -0.5)`.

- `FloatElement::nth_root(n)` — the real `n`th root, sign-preserving for odd
  `n` (`nth_root(-8, 3) == -2`), `NaN` for negative operands with even `n`.
  libm has no general-root primitive, so the default composes
  `powf(|x|, 1/n)` with `copysign` for the odd case; primitive `f64` and the
  `F64` wrapper override with native double-precision. Consumers previously
  emulated roots as `powf(x, 1/n)` / `powf(x, 0.25)` / `powf(x, 0.2)`.

### Fixed

- The `F64` wrapper evaluated `log10`, `log2`, `erf`, `erfc`, and `lgamma` in
  single precision. Its `FloatElement` impl overrode 19 of the trait's 24
  `f32`-routed defaults but not these five, so each ran
  `from_f32(libm::<op>f(self.to_f32()))` — an `f64 → f32 → f64` round trip that
  discarded roughly nine decimal digits on the one type whose entire contract
  is `f64` precision. Measured before the fix: `log10(2.0)` returned
  `0.3010300099849701` against the true `0.3010299956639812` (error `1.4e-8`),
  and `lgamma(5.0)` returned `3.178053855895996` against `3.1780538303479458`
  (error `2.6e-8`). All five now dispatch the native `libm` `f64` entry points,
  matching the primitive `f64` impl, and `tests/float_special.rs` pins them
  against both that impl and closed-form references to `1e-15`.

- `Bf8`, `Bf4`, `F8`, and `F4` compared and ordered by raw bit pattern. They
  carried `#[derive(PartialEq, PartialOrd)]` over their `u8` code while the
  sign bit is that code's most significant bit, so every negative value sorted
  **above** every positive one, `-0` compared unequal to `+0`, and NaN compared
  ordered and reflexive. Because `NumericElement::min_scalar`/`max_scalar` are
  defaults over exactly that `PartialOrd` and are not overridden for these
  types, every Min/Max reduction, `clamp`, and sort over a sub-byte float was
  silently sign-inverted — and a Max reduction seeded with `MIN_VALUE` returned
  its own identity element for all input, since the seed's sign bit dominated
  every finite operand. All six bit-pattern float wrappers now share one
  float-semantic comparison (widening to `f32`, which is exact for each), and
  `tests/float_order.rs` verifies it exhaustively over every encoding against
  an independent sign-magnitude oracle. No `Eq`/`Ord`/`Hash` is added: these
  are floats. Bit-exactness contracts (encoding round trips, rkyv archive
  identity) now assert on the raw code rather than through `PartialEq`.

- ATLAS-EUNOMIA-044: Unsigned primitive `u8`/`u16`/`u32`/`u64`/`usize` and
  the wrapper integer types `I8`/`I16`/`I32` now correctly implement
  `checked_add`/`checked_mul` (returning `None` on overflow) and
  `saturating_add`/`saturating_mul` (capping at `MIN_VALUE`/`MAX_VALUE`) on
  `NumericElement`, matching their primitive `i8`/`i16`/`i32` siblings.
  Before this patch, all of these types inherited the trait's float default
  `Some(self OP self)`, which silently wrapped in release / panicked in debug
  on integer overflow. Wrapper `I32` sqrt (and the corresponding `I8`/`I16`
  wrappers for parity) now route through the exact `i32::isqrt` /
  `i8::isqrt` / `i16::isqrt` primitives instead of the previous
  `(self as f64/f32).sqrt() as Self` path, eliminating precision loss for
  operands near `MAX_VALUE`. New regression tests cover overflow and signed
  underflow; all 76+ existing focused Nextest cases, doctests, and
  `cargo clippy -D warnings` continue to pass.

### Changed

- Added a GitHub Release workflow that validates crate identity and package
  contents before publishing through crates.io Trusted Publishing.

- E-035 adds provider-owned `UnitScalar` conversion and scaling for all
  shipped real storage types plus `Complex32`/`Complex64`, allowing Aequitas to
  convert real quantities and complex phasors through one native seam.

- E-030 vectorizes `neon::unpack_f8_to_f32` as a branchless 16-element
  arithmetic decode, deleting the 1,024-byte scalar gather table. The NEON
  path is bit-exact against the scalar conversion kernel for every byte
  value, verified by an exhaustive phase-and-length differential test on
  aarch64. The `unsafe_intrinsics` re-export now includes aarch64, matching
  the inner module gate.

## [0.7.0] - 2026-07-21

### Added

- E-034 adds provider-owned absolute and relative equality assertions for
  primitive, complex, reference, slice, and array values. Combined tolerances
  retain absolute-or-relative semantics, including negative-value scaling.

## [0.6.0] - 2026-07-19

### Breaking

- E-025c removes Eunomia's `NumericElement`, `FloatElement`, sealing, and
  `CastFrom` implementations for `half::f16`/`half::bf16`. Use Eunomia's native
  `F16`/`Bf16` provider types instead. The `half` crate is now a dev-only
  differential oracle and is absent from Eunomia's production dependency graph.

### Migration

- Replace `half::f16`/`half::bf16` trait-bound usage with `F16`/`Bf16`.
  Representation-preserving boundaries use `from_bits`/`to_bits`; numeric
  boundaries use `from_f32`/`to_f32`.

### Added

- Value-semantic `NumericElement`/`FloatElement` contract tests for the native
  float wrappers (`F16`, `Bf16`, `F32`, `F64`) in `tests/float_element.rs` — the
  float analogue of `integer_element.rs`. Pins, across all four types, that the
  reduced-precision wrappers are first-class scalars and not mere storage:
  constants, exact arithmetic, `abs`/`sqrt`/`signum`/`scalar_fmadd`, ordering
  reductions, `powi`, the transcendental defaults at exact points, `CastFrom<i32>`,
  and native-grid rounding (`F16`'s 10-bit grid strictly finer than `Bf16`'s
  7-bit at `0.1`). This is the contract the hermes SIMD kernels bind as
  `SimdKernel<F16>`/`<Bf16>`; every assertion is analytic, no tolerances.

## [0.5.0] - 2026-07-18

### Breaking

- E-025: `F16` and `Bf16` are now native `u16`-backed (`pub struct F16(pub u16)`)
  instead of wrapping `half::f16`/`half::bf16`. Conversions route through the
  native `convert` kernel (bit-exact vs `half`); `PartialEq`/`PartialOrd` are
  float-semantic (±0 equal, NaN unordered), not the `u16` bitwise derive. The
  wrapper and packed-unpack paths no longer use `half` — it remains only for the
  raw `half::f16`/`bf16` trait impls (consumers still on raw half) and the
  differential-oracle tests. Consumers constructing/reading `F16(_).0` as a
  `half::f16` must migrate to the native surface (`F16::from_f32`, `F16(bits)`).

### Added

- E-025b: const, representation-preserving `F16::from_bits`/`to_bits` and
  `Bf16::from_bits`/`to_bits` APIs for SIMD and device kernels. Exhaustive tests
  preserve every `u16` encoding, including signed zero and NaN payloads.
- E-033: bulk `bfloat16`↔`f32` conversion (`Bf16::widen_slice` /
  `Bf16::narrow_slice`) — ~16× (widen) / ~3.5× (narrow) faster than the scalar
  kernel via autovectorized shift/round-and-truncate (no intrinsics), bit-exact
  vs `half` and the kernel. Completes the bulk conversion surface.
- E-031: F16C-accelerated bulk `f16`↔`f32` conversion (`F16::widen_slice` /
  `F16::narrow_slice`) — ~38–49× faster than the scalar kernel on x86-64 with
  F16C (bit-exact vs `half`), scalar fallback elsewhere. Adds eunomia's first
  criterion benchmark suite (`benches/conversions.rs`) with committed baselines.
- E-026: native `layout` byte-layout vocabulary — `Pod`/`Zeroable` unsafe marker
  traits and the safe reinterpretation surface (`bytes_of`, `cast_slice`,
  `from_bytes`, `pod_read_unaligned`, fallible `try_*` variants, `PodCastError`),
  owned natively rather than borrowed from `bytemuck`. Impl'd for the primitives,
  scalar wrappers, and `Complex<T>`; `core`-only. The existing `bytemuck::Pod`
  impls are retained, so eunomia types satisfy both vocabularies.

### Fixed

- E-028: restore the `avx512vl` guard in `packed/unpack/arch.rs`'s no_std
  AVX-512 feature detection, matching the `std` check and the intrinsics'
  `#[target_feature]` requirement.

### Changed

- E-029: remove the blanket `#![allow(clippy::missing_safety_doc)]` and document
  every unsafe site in the packed/unpack SIMD path (`# Safety` docs on the 18
  intrinsic functions, `// SAFETY:` on the 18 dispatch blocks and the `avx512`
  table `transmute`) and the scalar `bytemuck` marker impls.

## [0.4.0] - 2026-07-18

### Changed

- E-023 routes E5M2, E2M1, E4M3, and E3M0 through the native conversion kernel.
  Narrowing now rounds to nearest with ties to even; finite-only formats
  saturate infinity and overflow at their signed maximum finite value.

### Fixed

- Correct E5M2 and E2M1 subnormal scales, E2M1 finite limits, and the sign bits
  of the four-bit minimum-value constants.
- Remove the duplicate scalar and packed-table conversion implementations.

## [0.3.0] - 2026-07-18

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
