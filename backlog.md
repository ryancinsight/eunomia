# Eunomia backlog

Sprint target: 0.6.0 (native reduced-precision provider contract).

## Recently completed

- **E-034 [minor] — done; owner: Codex `/root` stale-peer takeover.** Published
  provider-owned relative-equality assertions for `f32`, `f64`, complex,
  reference, slice, and array values. Native-precision tolerances, positive and
  negative relative scaling, equal infinities, unequal infinities, NaN, and
  signed zero are covered. Format, no-default-feature check, warning-denied
  Clippy, 94/94 Nextest, 9/9 doctests, warning-denied Rustdoc, 196/196 SemVer
  checks, clean-provider Helios compilation, and hosted Helios run `29882508040`
  passed.
- **E-021 [arch]** Remove Eunomia's direct `num-traits` dependency and foreign
  `Zero`/`One` implementations, pin the native `Complex32`/`Complex64` ABI at
  compile time, and align the optional NumPy element boundary with
  NumPy/PyO3 0.29. Owner: Codex. Scope: Eunomia manifests, complex identity and
  layout modules, provider-contract tests, ADR 0002, and PM artifacts.
  Acceptance: all-feature and no-default-feature gates pass; Hephaestus can use
  `eunomia::Complex` directly in device buffers and Python NumPy results.
  Delivered by Eunomia PR #36, Hephaestus PR #48, and Leto PR #42.

## Byte-layout & reduced-precision (E-022…E-030)

Workstream: own native reduced-precision conversion + a byte-layout vocabulary;
retire the `half` runtime dep; bridge (not replace) bytemuck. Owner: Claude.
Scope: `convert/`, `types/floats.rs`, `packed/`, `casts/`, `impls/wrappers/`,
`traits/` — disjoint from E-021 (complex/numpy). ADR 0003; gap_audit
§Byte-layout / reduced-precision. Dependency order: E-022 → {E-023, E-025, E-026}
→ {E-024, E-027}; E-028/E-029/E-030 independent.

- **E-022 [minor] — done** Native `binary16`/`bfloat16` conversion kernel.
  `convert::{narrow, widen}`: one generic const-parameterized IEEE narrow/widen
  (`<const E, const M>`, round-to-nearest-ties-to-even, subnormals, inf/NaN,
  f32-subnormal). Acceptance (met): bit-exact vs `half` — exhaustive widen (2¹⁶,
  both formats) + exhaustive finite round-trip + ~4.2M rounding sweep + pinned
  ties-to-even; fmt/clippy-D/nextest(52/52)/doctest/rustdoc clean. Additive
  `pub mod convert`. Merged by PR #37 (`6f431f2d`).
- **E-023 [minor] — done; owner: Codex; scope: conversion kernel,
  sub-byte types/tests, packed conversion tables, numeric constants, and PM
  artifacts.** Fold `F8`/`Bf8`/`F4`/`Bf4` conversions onto the E-022 kernel
  (generalize the kernel's special-value handling as needed), deleting the four
  hand-rolled `types/floats.rs` copies and the truncation defect (G-C2/G-A3);
  pin each format's convention in Rustdoc + add reference-value & round-trip
  tests (no external oracle — G-C3/G-T1). Dep: E-022. Acceptance: value-semantic
  reference tests per format; kernel is the single conversion home; RNE replaces
  truncation with a documented derivation. Acceptance met: analytical and
  exhaustive value tests, Nextest 60/60, all feature/doc/lint/semver gates, and
  a local-source Leto/Hephaestus integration check pass.
- **E-024 [arch]** OCP-MXFP `FP8`(E4M3)/`FP4`(E2M1) formats (no infinity) via a
  kernel special-value **policy parameter**, added when coeus/hephaestus
  quantization needs them. Dep: E-023 + a driving consumer. Acceptance: OCP
  reference vectors match; existing IEEE-style types unchanged. ADR 0003 D3.
- **E-025 [arch/major] — done**
  `F16`/`Bf16` are now native `u16`-backed (breaking `.0` field change): conversions
  route through the E-022 kernel (bit-exact vs `half`, proven exhaustively over all
  2¹⁶ patterns + an f32 narrow sweep), `PartialEq`/`PartialOrd` are manual
  **float-semantic** (not bitwise — preserving `half`'s ±0/NaN/ordering), and the
  whole wrapper + packed path (scalar + AVX2/AVX-512/NEON + slice) is half-free.
  E-025c removed the foreign raw-half trait/cast surface and moved `half` to the
  dev-only differential-oracle graph. Completed as a **takeover** of
  a concurrent peer's packed-side WIP (dispatch/intrinsics) combined with my
  wrapper-side re-back; I also fixed the aarch64 NEON path the peer had not
  converted. Evidence: fmt / clippy-D / nextest 76/76 / doctest / rustdoc / no_std
  clean; aarch64 NEON correct-by-construction (cross-compile blocked by a missing
  C toolchain, unrelated).
  **E-025b done — owner: Codex; scope: native `F16`/`Bf16` bit-pattern
  constructors/accessors, contract tests, Rustdoc, and PM artifacts.** Hermes'
  intrinsic kernels require representation-preserving construction and
  extraction without reaching through Eunomia's public tuple field. Acceptance:
  const `from_bits`/`to_bits` APIs preserve all `u16` patterns, signed zero, and
  NaN payloads; full Eunomia gates and a path-overridden Hermes check pass.
  **E-025c done — owner: Codex; scope: Eunomia manifests, raw-half
  primitive/cast impls, provider tests, Rustdoc, and PM artifacts.** Hermes and
  Leto now use `eunomia::F16`/`Bf16` directly. Remove Eunomia's foreign
  `half::f16`/`bf16` numeric and cast surface and retire `half` from the
  production dependency graph; retain it only as the independent dev-time
  differential oracle. Acceptance: production source/manifests contain no
  `half` type dependency, all Eunomia gates pass, and path-overridden Hermes and
  Leto checks remain green. Apollo's raw-half FFT surface is a separate
  Apollo-owned migration and does not require Eunomia's foreign impls.
  Acceptance met: normal dependency graph is half-free; all producer gates pass
  (86/86 Nextest, 5/5 doctests); Hermes 0.4, Leto 0.39, and Hephaestus 0.17
  compile against Eunomia 0.6. Hephaestus required its stale Hermes 0.3/Leto
  0.38 lock closure to advance to their merged native-provider defaults.
- **E-026 [arch] — vocabulary done; gating deferred** Native `layout` module
  (G-A2): `Zeroable`/`Pod` unsafe markers (per-impl `// SAFETY:`, layout pinned by
  the existing `types` `const _` asserts) + the used safe casts (`bytes_of`(+mut),
  `cast_slice`(+mut), `from_bytes`, `try_*` fallible variants + `PodCastError`,
  `pod_read_unaligned`). Impl'd for the primitives, scalar wrappers, and
  `Complex<T>`; `core`-only (no_std). Additive — the existing `bytemuck::Pod`
  impls are untouched, so eunomia types satisfy **both** vocabularies. 11
  value-semantic tests; fmt/clippy-D/nextest 70/70/doctest/rustdoc clean.
  **Deferred (co-evolution, pairs with E-027):** making `bytemuck` an optional
  (default-on) feature and gating its impls is NOT purely additive —
  `default-features = false` consumers (e.g. hephaestus GPU buffers) rely on
  `eunomia::Complex: bytemuck::Pod`, so it requires consumer verification, not a
  solo eunomia commit.
- **E-027 [arch]** Migrate consumer GPU-ABI structs (hephaestus ~130, coeus
  ~115, …) onto the E-026 vocabulary while preserving the `bytemuck::Pod` wgpu
  contract via the bridge; add eunomia as a direct hephaestus dep. Dep: E-026.
- **E-028 [patch] — done** Fixed `packed/unpack/arch.rs` no_std AVX-512 detection
  (G-C1). The `"avx512bw"`→`"avx512f"` copy-paste was already corrected under
  E-023; restored the `avx512vl` guard both no_std branches dropped, matching the
  `std` runtime check and the intrinsics' `#[target_feature]`. No unit test — the
  no_std path is not exercised on the std toolchain; correct-by-construction
  against the std/intrinsic requirements.
- **E-029 [patch] — done** Unsafe/doc discipline: removed the blanket
  `#![allow(clippy::missing_safety_doc)]`; added `# Safety` docs to the 18 intrinsic
  `unsafe fn` (avx2/avx512/neon), and `// SAFETY:` to the 18 dispatch blocks, the
  `avx512` table `transmute`, and the 22 scalar `unsafe impl bytemuck::…` (one
  block-level rationale). Removed the `#[cfg(test)]`-in-doc-comment cruft + stale
  "frozen" note in `impls/field.rs` (G-C5). Bf8 "1.4.3" mislabel (G-D1) was already
  corrected under E-023. Evidence: clippy `-D warnings` with `missing_safety_doc`
  enforced; fmt / nextest 59/59 / doctest / rustdoc clean.
- **E-030 [patch] — done** Vectorized `neon::unpack_f8_to_f32` (G-T2). The
  scalar 256-entry gather LUT is replaced by a branchless arithmetic decode
  over 16-element blocks: normals rebias `(exp + 120) << 23`; subnormals
  evaluate `mant * 2^-9`, exact in `f32` and reproducing the scalar
  normalizer's pattern including signed zero; the finite-only top exponent
  selects NaN with the canonical quiet bit forced on zero payload, all
  bit-exact against `widen_finite::<4, 3>`. The 1,024-byte `TABLE_BITS`
  static is deleted; the kernel adds ~20 NEON ops per 4-lane block, an
  estimated net ~700-byte rodata/text reduction (analytical bloat note —
  no two-revision release build was run in the contended shared cache). The
  `unsafe_intrinsics` re-export gates in `lib.rs` and `packed/mod.rs` widen
  from `x86_64` to `x86_64 | aarch64`, matching the inner module gate so the
  NEON path is differential-testable through the public contract (additive
  on aarch64; zero public-surface delta on x86_64 — SemVer `[patch]` by
  analysis, no mechanical run). Evidence: aarch64 lib and test-target checks
  pass warning-free under the pinned 1.95.0 toolchain; exhaustive
  256-value × 16-phase × 257-length NEON-vs-scalar differential test added
  (`neon_f8_unpack_bit_matches_scalar_for_every_byte_and_phase`, executes on
  aarch64; not executed on this x86-64 host — aarch64 hardware/CI run remains
  the open execution evidence); host gates pass: fmt, Clippy `-D warnings`,
  nextest 93/93, doctests 9/9, rustdoc clean.
- **E-031 [minor] — done** F16C-accelerated bulk `f16`↔`f32` conversion + eunomia's
  first criterion benchmark suite (the E-025 "hardware conversion ladder"
  follow-up). `F16::widen_slice`/`narrow_slice` dispatch to `vcvtph2ps`/`vcvtps2ph`
  (8 lanes/instruction, round-to-nearest-even) on x86-64 F16C, scalar-kernel
  fallback elsewhere — bit-exact vs `half` (exhaustive widen + rounding sweep,
  scalar remainder covered). Measured (4096 elems, F16C host): **widen ~38×**
  (2305 ns → 61 ns), **narrow ~49×** (3723 ns → 75 ns) vs the scalar loop;
  `benches/conversions.rs` commits the baselines to gate regressions. Dogfoods
  the E-026 `layout::cast_slice` for the transparent `F16`↔`u16` reinterpret.
  Consumer: native reduced-precision processing in Hermes/Leto (E-025).
- **E-032 [patch] — investigated, no change (finding recorded)** AVX2
  `unpack_f8_to_f32` optimization: the E-031 benchmark flagged it at ~7 Gelem/s
  (gather-bound hypothesis). A branchless SIMD E4M3 decode (`significand × 2^scale`,
  flush-to-zero-safe, bit-exact incl. NaN payloads) was implemented and measured
  **~27% slower** (684 ns vs 537 ns, p < 0.05) than the `_mm256_i32gather_ps` LUT
  on this µarch — the hot 1 KB LUT gather out-throughputs the ~20-op ALU dependency
  chain. **Hypothesis refuted; the gather is retained** (no regression shipped).
  Kept the deliverable: an exhaustive direct `avx2::unpack_f8_to_f32` differential
  test (all 256 bytes vs the scalar kernel, valid on AVX-512 hosts too). **Do not
  re-attempt the branchless-compute path for f8→f32.**
- **E-033 [minor] — done** Bulk `bfloat16`↔`f32` conversion (`Bf16::widen_slice`/
  `narrow_slice`), completing the bulk conversion surface begun in E-031.
  `bfloat16` is the high 16 bits of an `f32`, so widen is a shift and narrow a
  round-and-truncate (RNE) — both branch-light enough to autovectorize, no gather
  or intrinsics. Bit-exact vs the kernel `narrow::<8,7>` **and** `half` (exhaustive
  widen + a rounding sweep confirming the fast round-and-truncate matches the
  kernel). Measured (4096 elems): **widen ~16×** (2471 ns → 150 ns),
  **narrow ~3.5×** (3625 ns → 1036 ns) vs the scalar kernel loop. Unlike E-032
  (f8, where compute lost to the gather), the analytical model held —
  autovectorization confirmed by measurement.

## Done

- **E-012 [minor]** Added provider-owned float-to-`usize` `CastFrom` edges for
  native indexing consumers. Every primitive and wrapped `FloatElement` family
  now uses the canonical cast trait. Warning-denied all-feature Clippy passes;
  focused cast Nextest passes 2/2 and full Eunomia Nextest passes 44/44.
  Driver: RITK native interpolation migration.
- **E-001 [arch]** Scaffold eunomia workspace; migrate `hermes-numeric` content
  verbatim → `crates/eunomia` (scalars, complex, packed, casts, ops, traits).
  Builds + all tests pass standalone. — ADR 0001.
- **E-002 [minor]** `eunomia-gpu`: const-generic `Vector<T, N>` (`Vec2`/`Vec3`/
  `Vec4`), `repr(C)`/`Pod`, std430 storage layout. 3 tests green.

- **E-003 [arch]** Rewire `hermes-simd-*` off `hermes-numeric` → `eunomia`;
  removed `crates/hermes-numeric` from hermes. — hermes PR #2, full workspace
  tests green.
- **E-004 [arch]** Rewired `leto` (`leto::Complex` → `eunomia::Complex`) and
  `coeus`/`hephaestus` eunomia path-patches. — leto PR #5, coeus PR #93,
  hephaestus PR #25; 213 leto-ops tests + GPU chain green.
- **E-005 [patch]** atlas meta-repo README + submodule updated (eunomia
  foundation row, hermes row revised). — atlas `1d22b3d6`.
- **E-006 [arch]** Consolidated `coeus-core::Complex` onto `eunomia::Complex`
  (kept coeus `Scalar`/`FloatOps`/`CpuUnaryDispatch` impls on it; added `Rem` +
  `PartialOrd` to eunomia for the sealed `Scalar` bound). — eunomia PR #1, coeus
  PR #94; coeus-core 57 tests green. Last duplicate native complex type closed.

## Capabilities (done — enable stack-wide num-complex/num-traits removal)

- **E-008 [minor]** Full `num_complex`-equivalent Complex surface
  (`norm`/`norm_sqr`/`l1_norm`/`arg`/`conj`/`scale`/`to_polar`/`from_polar`/`cis`/
  `exp`/`ln`/`sqrt`/`powf`/`sin`/`cos`/`tanh`/`i()`) + `FloatElement`
  transcendentals (`exp`/`ln`/`sin`/`cos`/`tan`/`sinh`/`cosh`/`tanh`/`atan2`/
  `powf`/`recip`, libm, native precision). — eunomia PR #2, analytic tests green.
- **E-009 [minor]** Optional `serde` derive on Complex (`{re,im}`, num_complex
  wire-compatible). — eunomia PR #3.
- **E-010 [minor]** mnemosyne-arena `eunomia` feature: `ScratchElement` for
  `eunomia::Complex<f32>/<f64>`, parallel to its `num-complex` feature. —
  mnemosyne PR #3.
- **E-011 [minor]** Platform-sized primitive numeric elements: `isize` and
  `usize` now implement `NumericElement` through the same sealed primitive SSOT
  as fixed-width integers, with the required `CastFrom<i32>` edge and
  pointer-width metadata tests. Driver: Leto scalar SSOT migration (CR-4) can
  keep platform-sized scalar support without Leto-local aliases. Evidence:
  `cargo check -p eunomia`, `cargo clippy -p eunomia --all-targets -- -D warnings`,
  and `cargo nextest run -p eunomia` pass.

## Roadmap — consumer num-complex → eunomia migration (tracked [arch])

Prereqs now satisfied: full Complex surface (E-008), serde (E-009), mnemosyne
ScratchElement (E-010).

**DEPENDENCY ORDERING (corrected 2026-06-29 — these are NOT independent passes).**
The consumers are chained through apollo: `num_complex::Complex` crosses crate
boundaries in public APIs, so a downstream repo cannot drop num-complex until its
upstream does. Verified order:

  apollo (FFT/spectral core, num_complex in its public `FftPlan` API)
     ↑ consumed by
  ritk-filter (calls `apollo_fft::FftPlan1D::*_complex_slice_inplace(&mut [Complex<f32>])`)
  CFDrs spectral paths
     ↑ consumed by
  (end consumers)

Concretely: **ritk is BLOCKED on apollo.** A trial swap of ritk-filter's 10
num_complex files to `eunomia::Complex` failed to build — `apollo_fft`'s
`forward_complex_slice_inplace(&mut [num_complex::Complex<f32>])` mandates the
num_complex type at the boundary. eunomia::Complex IS layout-identical
(`#[repr(C)]` `{re,im}`, Pod), so a `bytemuck::cast_slice_mut` boundary hack
*could* unblock ritk while keeping num-complex as a thin dep — but that does NOT
remove the dependency, so it is rejected. apollo must migrate its public API first.

**eunomia-side prerequisites for apollo: COMPLETE** (the swap is now purely
mechanical — no eunomia round-trips needed mid-migration):
  - Complex method surface verified against apollo-fft's usage
    (`new`/`sin`/`cos`/`norm`/`sqrt`/`tanh`/`exp`/`ln`/`conj`/`arg`/… all present);
    the one gap, `powi`, was added (eunomia `4538864`, exact exponentiation-by-
    squaring).
  - Reference operators (`&a·&b`, `a·&b`, `&a·b` for Add/Sub/Mul/Div) added
    (eunomia `4f07783`) so apollo's bulk `&a*&b` sites compile unchanged — the
    same lever leto's operators provided for gaia.
  - `Complex` is `#[repr(C)]`/Pod, layout-identical to `num_complex::Complex`, so
    device buffers and FFI round-trip identically.
  - Full num_complex **operator** surface now on eunomia::Complex (added across
    this session): `Complex32`/`Complex64` aliases (`8da90f8`); reference ops
    (`4f07783`); scalar `Complex*T`/`/T` + complex compound-assign (`cb5ff34`);
    `ZERO`/`ONE` consts (`eb19b03`); `Sum`/`Product` (`da9d2c3`); scalar
    `Complex±T` + scalar compound-assign `*= s`/`/= s`/`+= s`/`-= s` (`762e58f`).

**apollo-fft LIB migration: DONE (local WIP, branch `refactor/apollo-fft-eunomia`,
apollo `3b00eb0` — NOT merged).** The apollo-fft *library* compiles on
eunomia::Complex (0 lib errors); 122 files. What it took (the reusable playbook
for the remaining ~19 apollo crates):
  - `num_complex::Complex` → `eunomia::Complex` in apollo-fft src **and**
    `apollo-fft-macros` quote!{} templates (the macros emit Complex code — they
    must migrate too; their compile-time `ComplexF64` is a *local* type, leave it).
  - num_traits scalar bounds: `num_traits::Float` → `eunomia::RealField` (NOT
    `FloatElement` — `RealField` carries `Neg`/ordering); `num_traits::NumAssign`
    → `core::ops::{Add,Sub,Mul,Div}Assign`; drop the `type Complex: num_traits::Zero`
    bound (apollo's `complex(0,0)` is the zero).
  - `F::zero()`/`F::one()` (scalar) → `<F as eunomia::NumericElement>::ZERO`/`ONE`;
    BUT a self-contained numeric trait with its own `zero()` (e.g. `KernelScalar`,
    impl'd for real **and** Complex) keeps `T::zero()` — do not force NumericElement
    on it (Complex isn't NumericElement).
  - wire eunomia into the workspace (`[patch]` + git dep); mnemosyne feature
    `num-complex` → `eunomia`; drop num-complex/num-traits from the crate Cargo.toml.
- **apollo → leto arrays (ndarray → leto).** apollo uses ndarray pervasively
  (~166 files / 17 of 20 crates: `Array1/2/3::<Complex>::zeros()` etc.), so the
  full migration replaces **ndarray with leto** — the leto integration in scope.
  - **No `num_traits::Zero` blocker after all.** ndarray's `zeros()` needs
    `A: num_traits::Zero` (which eunomia::Complex can't impl, orphan rule), but
    **leto's `Array::zeros` requires only `T: Default + Clone`** and
    eunomia::Complex *derives* `Default` (→ `0 + 0i`). So `leto::Array1::
    <Complex>::zeros([n])` just works — the ndarray boundary dissolves once apollo
    is on leto arrays; no eunomia num_traits bridge needed.
  - **Real cost = leto array-API parity** for the ndarray methods apollo uses:
    `.iter` (480), `.len` (132), `.to_owned` (64), `.dim` (52), `.view` (37),
    `.sum`/`.mapv`/`.fold` (23 each), `.assign` (12), `.iter_mut` (8), `.row` (5).
    Name-maps (apollo-side rewrites, no redundant leto aliases): `.len()`→
    `.size()`, `.dim()`→`.shape()`, `.to_owned()`→`.to_contiguous()` (already on
    `ArrayView`/`Array`), `.sum()`→`leto::reduction`, `.iter_mut()`→
    `.as_slice_mut().unwrap().iter_mut()` (contiguous; apollo's FFT buffers are).
    Genuine leto gaps — **SUBSTANTIALLY DONE**: `mapv`+`fold` (`c22a916`),
    `mapv_inplace`+`assign` (`f996308`), `as_slice`+`as_slice_mut` (`e760726`).
    The leto array surface now covers apollo's ndarray usage. Only `row`/`column`
    (→ `index_axis`/`lanes`) and a general *strided* mutable iterator remain, both
    minor / already worked around (`as_slice_mut` for contiguous in-place iter).
  - Construction-shape note: leto constructors take `[usize; N]` not ndarray's
    tuple/usize, e.g. `Array2::zeros((r, c))` → `Array2::zeros([r, c])`.
  - Sequencing: grow leto's array API to parity (clean mergeable enablers, like
    `mapv`/`fold`) → then the per-file ndarray→leto swap across apollo. Until the
    whole workspace is on leto + eunomia, apollo-fft can't merge (the ~19 consumer
    crates + tests keep it red).

- **E-011 [arch]** **apollo FIRST (critical-path upstream).** ~250 num_complex
  files across ~20 crates (apollo-{fft,czt,dctdst,dht,frft,fwht,gft,hilbert,mellin,
  ntt,nufft,qft,radon,sdft,sft,sht,stft,…}). apollo-fft *alone* is 121 num_complex
  files (larger than the whole gaia migration), and is consumed by ~19 sibling
  crates, so its public `Complex` API change ripples across the whole workspace —
  migrate **bottom-up, apollo-fft first** (it is what ritk needs). Each crate: add
  eunomia dep+patch; swap `num_complex::Complex` → `eunomia::Complex` (layout-
  identical, mechanical bulk — ref-ops + `powi` now exist, see prereqs above);
  `num-complex` `bytemuck` feature → native Pod, `serde` → E-009; `mnemosyne`
  `features=["num-complex"]` → `["eunomia"]`; consolidate the GPU `ComplexPod
  {re,im}` structs onto `eunomia::Complex<f32>`. **FFT-numerics correctness is the
  gate** (differential vs reference transforms) — not rushable.
- **E-012 [arch]** **ritk** — UNBLOCKED ONLY AFTER apollo-fft (E-011). Then a clean
  ~10-file swap of ritk-filter (`num_complex::Complex` → `eunomia::Complex`,
  drop the `num-complex` bytemuck dep). Trial run done + reverted; resume post-apollo.
- **E-012b [arch]** **CFDrs.** Sparse-linalg blocker **largely RESOLVED** (earlier
  "needs a whole new sparse crate" was wrong). **Sparse linear algebra lives in
  leto-ops (CPU) + hephaestus (GPU), hermes-SIMD-backed** — the same layering as
  the dense linalg, NOT a separate crate. `leto-ops::application::sparse` already
  had `CsrMatrix` + `spmv` + `spmm`; **`CooMatrix` (assembly/triplet format with
  duplicate-accumulating `to_csr`) added** (leto `34e4019`), covering CFDrs's
  `CsrMatrix`/`CooMatrix` usage. Crucially, CFDrs implements its **own** iterative
  solvers (`cfd-math/linear_solver/{conjugate_gradient,bicgstab,gmres,
  preconditioners}`) over SpMV — no external sparse-solver dependency — and direct
  `nalgebra_sparse` usage is just 2 sites (`pattern::SparsityPattern`,
  `ops::serial`). Dense linalg is covered (leto-ops cholesky/LU/QR/SVD/eigen/schur).
  So CFDrs's remaining migration is the mechanical swap `nalgebra_sparse::
  {CsrMatrix,CooMatrix}` → `leto_ops::…::sparse::{CsrMatrix,CooMatrix}` + the 2
  minor nalgebra_sparse sites, then the dense `nalgebra`→leto + `num_complex`→
  eunomia passes (spectral via apollo, E-011). No remaining hard infra blocker.
  Possible leto follow-ups if CFDrs needs them: GPU SpMV in hephaestus; a
  `SparsityPattern` analogue; CSC format. (gaia-style per-site migration.)
- **E-013 [minor]** Replace direct `num-traits` deps with `eunomia::FloatElement`/
  `NumericElement`.
  - **leto: DONE** — leto PR #6 (reduction/statistics off num_traits::Float/Zero/
    One; num-traits dropped from leto + leto-ops; full suite green). Enabled by
    the float-surface completion below.
  - **ritk: DONE** — ritk PR #28 (gaussian_kernel → FloatElement; num-traits dropped from ritk-tensor-ops).
  - **CFDrs, gaia: NOT a clean swap** — their central `Scalar` traits bound
    `num_traits::Float` *alongside* `nalgebra::RealField` (gaia
    `src/domain/core/scalar.rs`: `Scalar: RealField + Float + ToPrimitive`),
    so the domain math leans on RealField. Dropping num_traits there is
    entangled with the larger **nalgebra → eunomia** migration, which first
    needs a eunomia `RealField`-equivalent (ordered/field/real surface beyond
    `FloatElement`). Promote as its own [arch] item — see E-017.

- **E-017 [arch]** nalgebra → eunomia (+ leto/hephaestus for the actual linear
  algebra).
  - **Scalar field surface: DONE** — `RealField`/`ComplexField` (eunomia PR #7,
    `traits/field.rs` + `impls/field.rs`). Generic field code now runs over
    f32/f64 *and* `Complex<f32>/<f64>` without nalgebra.
  - **Geometry library: in leto, NOT eunomia.** eunomia is datatypes (scalars/
    complex + the RealField/ComplexField scalar field traits). The nalgebra
    *geometry* (Vector/Point/Unit/Quaternion/Isometry + cross/norm/transforms)
    is linear algebra → **leto** (CPU; `leto::geometry`) / **hephaestus** (GPU).
    (An earlier mis-placement in eunomia-gpu was reverted, PR #9.)
    **Phase 1 — geometry library: COMPLETE** (in `leto::geometry`, 6 leto PRs):
    - Vector<T,N> (PR #7): dot/norm/norm_squared/normalize/distance/cross.
    - Point<T,N> (affine) + Unit<T,N> (PR #2 in leto-geometry series).
    - Quaternion/UnitQuaternion (Hamilton product, from_axis_angle,
      transform_vector q·v·q⁻¹) + Translation3/Isometry3 (from_parts,
      transform_point/vector, inverse). Rotations analytically tested.
    - Axis constructors (x/y/z_axis → Unit), Point From, optional `serde`
      (manual tuple impl for const-generic Vector). leto main `a81476d`.
    Covers gaia's entire geometry surface; 9 tests, clippy clean.
  - **Phase 1.5 — geometry library nalgebra-API completion: DONE.** scalar
    constructors (`Vector3::new(x,y,z)`), `.x/.y/.z` field access (repr(C)
    swizzle Deref, gaia uses ~1667×), serde, full operator set (Div + assign
    ops), and `eunomia::RealField::{infinity,neg_infinity,nan,min/max_value}`.
    leto main `5fa1a0a`+, eunomia main `7e4e64e`. The library is now a faithful
    nalgebra geometry replacement.
  - **Phase 2 — gaia swap: DONE.** gaia PR #5 (`refactor/migrate-to-leto-
    geometry`, gaia main `dfcf3a4`). 462 build errors → 0; **922 tests pass**,
    clippy clean; `nalgebra` + `num-traits` dropped from gaia Cargo.toml
    (incl. dev-deps). The per-site tail was resolved by category:
    - `Scalar: nalgebra::RealField + Float + ToPrimitive` → `eunomia::RealField`.
    - `nalgebra::{Vector,Point,Unit,UnitQuaternion,Isometry3,Translation3}` →
      `leto::geometry::*`.
    - `Float::min/max/clamp/abs/sqrt/sin/atan2(...)` free-fns → method calls
      (`min_scalar`/`max_scalar`/native).
    - `scalar * vector` → `vector * scalar` (orphan rule); `&v`/`&unit` args
      dropped or derefed for leto's by-value `Copy` ops (incl. `.dot`/`.cross`,
      `from_axis_angle`); `to_f64(&x).unwrap()` → `to_f64(x)` (eunomia
      conversions are infallible, take `self` by value).
    - **Root unifier:** added `[patch]` redirecting leto's git-eunomia to the
      local path so gaia saw **one** eunomia instance — this alone cleared the
      ~44 "`norm`/`dot`/`cross` exists but `T: RealField` not satisfied" errors.
    - Enabled by 4 leto additions this pass (leto `f2e3d5f`..`1b07ff0`):
      nalgebra-compat reference + transform operators (`&p-&q`, `rotation*v`,
      `iso*p`), `Vector3::{x,y,z}()` basis ctors, `Point += / -= Vector` +
      `Point: Default` (origin).
    (leto/hephaestus keep nalgebra as a *dev*-oracle — out of scope.)
- **E-007 [minor]** GPU layout vector types grow as a backend consumer appears;
  std140 UBO alignment stays a buffer-packing concern, not a type property.

## Capabilities + cleanup (done, this round)

- **E-014 [patch]** Vertical-hierarchy/SRP cleanup: `eunomia-gpu` flat lib.rs →
  `vector.rs`+`ops.rs` (ops via `core::array::from_fn`); `traits.rs` god-file →
  `traits/{numeric,float,cast}.rs`. — eunomia PR #4.
- **E-015 [patch]** Native f64 transcendentals for the `F64` wrapper (was
  f32-routed → widen-narrow defect). — eunomia PR #5.
- **E-016 [minor]** Completed `FloatElement` to num-traits `Float` parity:
  floor/ceil/round/trunc/signum + exact `powi`. — eunomia PR #6.
- **E-020 [minor]** Added `FloatElement::acos` with libm-backed reduced-
  precision defaults and native f64 primitive/wrapper overrides. Driver: CFDrs
  Murray's-law bifurcation migration needs inverse-cosine geometry without
  reintroducing nalgebra/num-traits scalar bounds or fake generic casts.
  Evidence: `rustup run nightly cargo fmt -- crates/eunomia/src/traits/float.rs
  crates/eunomia/src/impls/primitives/float.rs
  crates/eunomia/src/impls/wrappers/float.rs`, `rustup run nightly cargo check
  -p eunomia --lib`, `rustup run nightly cargo clippy -p eunomia --lib -- -D
  warnings`, focused `rustup run nightly cargo nextest run -p eunomia -E
  'test(f64_wrapper_transcendentals_are_native_precision) |
  test(rounding_and_powi_surface)'` (2/2), and downstream CFDrs
  `rustup run nightly cargo check -p cfd-1d --tests`.

Note: Leto and Hephaestus no longer declare `num-complex`. Leto retains
`nalgebra` and `ndarray` only as external test/benchmark oracles, so their
transitive `num-complex` edges are excluded from the production graph and
tracked for independent-oracle retirement in Leto.

## Cleanup (redundancy removal)

- **E-019 [patch]** Removed the dead `eunomia-gpu` crate: it had zero external
  consumers and its `Vector<T,N>` was redundant with `leto::geometry::Vector`
  (also `repr(C)`/Pod, GPU-buffer-compatible). Per the corrected architecture
  (eunomia = datatypes; vectors/geometry = leto CPU / hephaestus GPU), the
  GPU-vector concern does not belong in the datatype crate. eunomia is now a
  single-crate workspace. README + diagram synced.
