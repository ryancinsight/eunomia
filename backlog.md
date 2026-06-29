# Eunomia backlog

Sprint target: 0.1.0 (datatype-law foundation extracted from hermes-numeric).

## Done

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

## Roadmap — consumer num-complex → eunomia migration (tracked [arch], per-repo)

Prereqs now satisfied: full Complex surface (E-008), serde (E-009), mnemosyne
ScratchElement (E-010). Each repo is its own verified pass (build + tests):

- **E-011 [arch]** **apollo** (~249 files; the FFT/spectral core). Steps: add
  eunomia git-dep + path-patch; swap `num_complex::Complex` → `eunomia::Complex`,
  `num-complex` features (bytemuck→native Pod, serde→E-009); switch the
  `mnemosyne` dep from `features=["num-complex"]` → `["eunomia"]`; consolidate the
  8 GPU `ComplexPod {re,im}` structs (apollo-{hilbert,mellin,qft,sdft,sft,sht,
  stft}) onto `eunomia::Complex<f32>`. Large — likely split per-crate.
- **E-012 [arch]** **CFDrs** (8 files) + **ritk** (12 files): swap `num_complex`
  usages (mostly stability/spectral) to `eunomia::Complex`; add eunomia wiring.
- **E-013 [minor]** Replace direct `num-traits` deps (CFDrs, ritk, gaia, leto)
  with `eunomia::FloatElement` where the `Float` surface suffices (now covers
  transcendentals); keep num-traits only where a richer trait is genuinely used.
- **E-007 [minor]** GPU layout vector types grow as a backend consumer appears;
  std140 UBO alignment stays a buffer-packing concern, not a type property.

Note: leto/hephaestus retain `num-complex` as a **dev-dependency** (nalgebra
differential oracle returns `num_complex::Complex`) — that is correct and out of
scope for removal.
