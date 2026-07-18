# Eunomia gap audit

## Byte-layout / transmutation / reduced-precision (E-022…E-030)

Audit of eunomia's datatype surface against `bytemuck`, `zerocopy`, `half`, and
the std `core::mem::TransmuteFrom` trait — what eunomia should own natively to
shrink its dependency surface and consolidate the stack's byte/precision
vocabulary. Method: full read of eunomia internals + web-grounded capability
matrix + stack-wide consumer sweep (12 repos). Decision:
[ADR 0003](docs/adr/0003-native-byte-layout-and-reduced-precision.md). Baseline:
stable **1.95.0** pin (no `#![feature]` gates; SIMD via stable `#[target_feature]`
+ `is_x86_feature_detected!`).

### Owns vs delegates

| Concern | State | Backing |
| --- | --- | --- |
| `Complex<T>`, `F32`/`F64`/`I8`/`I16`/`I32` | Native | `#[repr(C/transparent)]` + `const _` layout asserts |
| `F8`(E4M3)/`Bf8`(E5M2)/`F4`(E3M0)/`Bf4`(E2M1) | **Native** with canonical RNE conversion | `u8`, `convert` kernel (E-023) |
| `F16`/`Bf16` | **Delegated** | wrappers over `half::f16`/`half::bf16` |
| f16/bf16 ↔ f32 conversion | **Delegated → now native** | `half` → `convert::{narrow,widen}` (E-022, done) |
| `Pod`/`Zeroable` markers | **Delegated** | `unsafe impl bytemuck::…` (`types/mod.rs`) |
| `cast_slice`/`bytes_of`/`from_bytes` / `*_ne_bytes` / `from_raw_parts` | **None** | eunomia supplies markers only; 1 `transmute` total |

### Capability matrix (BM bytemuck 1.14 · ZC zerocopy 0.8 · HF half 2.3 · TF std TransmuteFrom · EU eunomia)

| Capability | BM | ZC | HF | TF | EU |
| --- | --- | --- | --- | --- | --- |
| Compile-time layout check | ✗ silent-UB marker | ✓ derive+`KnownLayout` | — | ✓ compiler-proven | partial (`const _`) |
| Unaligned read | ✓ | ✓ | — | ~ | ✗ |
| Runtime-checked transmute | ✓ `CheckedBitPattern` | ✓ `TryFromBytes` | — | ✗ | ✗ |
| Slice reinterpret | ✓ `cast_slice` | ✓ DST/`Ref` | ~ `HalfFloatSliceExt` | ~ Sized | ✗ |
| Sub-byte floats (f8/f4) | ✗ | ✗ | ✗ | ✗ | **✓ only provider** |
| f16/bf16 hardware conversion | ✗ | ✗ | ✓ F16C/fp16 | ✗ | ✗ (E-025 follow-up) |
| Derive-checked safety | partial | ✓ | — | ✓ | ✗ |
| no_std | ✓ | ✓ | ✓ | ✓ nightly | ✓ |

Safety-model spectrum: bytemuck (`unsafe` marker, silent UB) → zerocopy (derive
compile-checked) → std `TransmuteFrom` (compiler-proven, but nightly/unstable/
[unsound #129097]). **`TransmuteFrom` is not adoptable** (stable pin);
**bytemuck cannot be dropped** (wgpu/metal/cuda fix `Pod` at the buffer
boundary); **`half` is fully replaceable** (no external lock-in, RNE oracle).

### Gaps

Correctness — **G-C1** `packed/unpack/arch.rs:34` no_std `has_avx512f` tests
`"avx512bw"` (copy-paste) + both no_std AVX-512 branches drop `avx512vl` (E-028).
**G-C2 (resolved E-023)** sub-byte narrowing now uses round-to-nearest-even.
**G-C3 (pinned E-023)** Bf8 is IEEE E5M2; F8/Bf4/F4 are finite-only with the top
exponent reserved for NaN. OCP types remain gated on a consumer (E-024).
**G-C4** 18 dispatch `unsafe` blocks + 18 intrinsic `unsafe fn` + the
sole `transmute` (`avx512.rs:56`) + 22 `unsafe impl bytemuck::…` carry no
`// SAFETY:` (crate-wide `#![allow(clippy::missing_safety_doc)]`) (E-029).
**G-C5** `impls/field.rs` "frozen" tests' `#[cfg(any())]` gate landed inside a
doc-comment → tests are live (E-029).

Architecture — **G-A1** `half` is a removable hard runtime dep (E-025).
**G-A2** no eunomia byte-layout vocabulary; own markers + used cast fns at
checked tier + bytemuck bridge (E-026/E-027). **G-A3 (resolved E-023)** all
sub-byte scalar and packed-table conversions use the E-022 kernel.

Tests/docs — **G-T1 (resolved E-023)** analytical and exhaustive sub-byte
conversion tests now pin all four formats. **G-T2**
`neon::unpack_f8_to_f32` is scalar, not vectorized (E-030). **G-D1**
`impls/wrappers/numeric.rs:217` mislabelled Bf8 "1.4.3"; corrected by E-023.

**Resolved:** G-C1 via E-028 (arch.rs no_std `avx512vl` guard); G-C4/G-C5 via
E-029 (unsafe SAFETY across the packed SIMD path + scalar `bytemuck` impls;
field.rs doc cruft); G-C2/G-C3/G-A3/G-D1 via E-023 (sub-byte fold onto the kernel:
RNE + pinned conventions + one generic home); G-A2 byte-layout vocabulary
delivered via E-026 (native `layout` module; `bytemuck`-gating deferred to the
E-027 consumer co-evolution); G-A1 half-drop eunomia-side done (E-025 — `F16`/`Bf16`
native `u16`; wrapper + packed paths half-free; half now raw-impl-path + oracle
only). Still open: E-025b/c (consumer half migration then full drop), G-T2
(vectorize `neon::unpack_f8_to_f32`, E-030), and E-027 (consumer bytemuck→eunomia
migration).

Non-gaps (verified, do not chase): `TransmuteFrom` not adoptable; `zerocopy` not
a migration target (sole stack use is out-of-scope consus `IntoBytes::as_bytes`);
bytemuck bridged not dropped.

### Consumer blast radius (source sites; `/target/` excluded)

`bytemuck` (~530) is **entirely internal GPU-ABI** (Pod-derive uniforms,
`bytes_of` upload, `cast_slice` readback) — no cross-crate public `Pod` type, so
the bridge is mechanical. `half` public gates chain **eunomia → hermes**
(`SimdKernel<f16>`, deepest) **→ leto-ops** (`Scalar`/`RealScalar` → `Array<f16>`)
**→ coeus** (`Tensor<f16>`) / **apollo-fft** (`pub fn forward_f16`). Raw
type-punning clients (a proper transmute vocabulary's users): coeus (3), hermes
(~6), moirai (4). Migration ranking: hephaestus ~130 bytemuck / 0 half (bridge
only; not yet a direct eunomia consumer); coeus ~115/~75; kwavers ~99/0; apollo
~38/~470; hermes ~6/~340 (deepest half). leto declares bytemuck but never uses it.

### E-022 (native f16/bf16 kernel) — delivered

- `convert::{narrow, widen}`: one generic const-parameterized IEEE narrow/widen
  kernel (RNE ties-to-even, subnormals, inf/NaN, f32-subnormal path).
- Evidence: bit-exact vs `half` — exhaustive widen (all 2¹⁶, f16+bf16), exhaustive
  finite round-trip, ~4.2M-case rounding sweep (every exponent/round/guard/sticky
  decision), pinned known-value + ties-to-even cases. `fmt`/`clippy -D warnings`/
  `nextest` 52/52 / doctest / rustdoc clean. Additive `pub mod convert` — [minor].

### E-023 (canonical sub-byte conversion) — delivered

- E5M2 instantiates the IEEE policy; finite-only E2M1/E4M3/E3M0 instantiate the
  reserved-top-exponent policy. Both policies monomorphize the same arithmetic.
- The four scalar conversion bodies and the separate packed-table conversion
  module are deleted. Exact widening, finite-encoding round trips,
  special-value mappings, ties-to-even boundaries, and every packed scalar/ISA
  dispatch encoding are value-tested.
- The cutover corrects E5M2/E2M1 subnormal scales, E2M1 finite limits, and the
  four-bit negative-limit sign encodings.
- Evidence: no-default/all-feature checks, warning-denied all-target/all-feature
  Clippy, Nextest 60/60, doctest 1/1, rustdoc, and semver checks pass. A
  temporary integration workspace path-patched Leto, Hephaestus Core, and
  Hephaestus WGPU to Eunomia 0.4.0 and passed `cargo check`. An AArch64
  dependency-free compile harness includes the actual kernel and NEON module,
  verifying the changed cross-ISA source at compile time; execution evidence is
  x86-64.

### Residual risk (byte-layout / reduced-precision)

- **R1** Native RNE is bit-exact vs `half` for f16/bf16 (E-022). Sub-byte
  formats have no external oracle; E-023 therefore pins them with analytical
  reference values, exhaustive finite-encoding round trips, and rounding laws.
- **R2** OCP-vs-IEEE sub-byte convention (G-C3) is a product decision with GPU-
  quantization impact; settle before E-024 migrates formats onto the kernel.
- **R3** Re-backing `F16`/`Bf16` storage (`half::f16` → `u16`, E-025) is breaking;
  ≥1 consumer constructs `Bf16(half::bf16::…)` (hermes test) → co-evolution unit.
- **R4** bytemuck 1.14 (pin) predates several 1.25 helpers; the E-026 bridge must
  target the 1.14 surface.
- **R5** eunomia is consumed by 9 repos (git, some pinned/patched); every API
  change triggers the pin-discipline integration sweep.

---

## E-021 (complex provider cutover) — delivered

- Eunomia PR #36 (`34d0cc8`) removes the direct `num-traits` dependency and
  publishes the native complex identity, ABI, and NumPy contracts.
- Hephaestus PR #48 (`82bb3a7`) removes direct `num-complex` ownership from its
  manifests and production sources. Device eigenvalue buffers and Python NumPy
  results now use Eunomia complex values without an intermediate vector copy.
- Leto PR #42 (`cf47686`) removes the root `num-complex` dependency and direct
  source use. Its production graph contains no `num-complex`, `nalgebra`, or
  `ndarray`; `nalgebra` and `ndarray` remain test/benchmark oracles with their
  retirement tracked in Leto.

### Evidence

- Compile-time assertions pin both complex layouts and the selected NumPy
  `Element` implementations.
- Eunomia passes no-default/all-feature checks, warning-denied all-target and
  all-feature Clippy, Nextest 53/53, doctests, rustdoc, and semver checks. The
  provider-contract suite covers ABI round-trips, native/generic identities,
  and NumPy trait identity.
- Hephaestus passes affected all-target/all-feature checks and Clippy, Nextest
  264/264 on real devices, the supported minimal feature set, doctests,
  rustdoc, semver checks, and the installed PyO3 NumPy eigenvalue test.
- Leto passes warning-denied all-target/all-feature Clippy, Nextest 305/305,
  doctests 8/8, rustdoc, and semver checks. Manifest/source and production-graph
  residue scans confirm the dependency boundary.
