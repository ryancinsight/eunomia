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
  - **Phase 2 — gaia swap: bulk applied, 462 build errors remain (WIP, local,
    NOT pushed/merged).** Reduced 519→462 via the library components above.
    The residue is a **per-site tail**, NOT library-fixable:
    - E0369 ×113: mostly `scalar * vector` — leto **cannot** impl this (orphan
      rule: `Mul<Vector> for f64`), so each site flips to `vector * scalar`.
    - E0308 ×135: type mismatches from nalgebra↔leto API shape differences.
    - E0599 ×120: `.unwrap()`/`.unwrap_or_default()` on methods that are now
      infallible (try_normalize/to_usize) — remove per site.
    - E0433 ×45 (unresolved paths), E0277 ×41 (bounds).
    Requires a **dedicated focused pass** of careful per-file manual review
    across 57 files of geometry code (correctness-critical) — not batch-fixable
    and not responsibly rushable. Original atomic steps:
    enable leto's `serde` feature in gaia; `Scalar: nalgebra::RealField + Float`
    → `eunomia::RealField`; swap `nalgebra::{Vector3,Point3,Vector2,Point2,
    Isometry3,Unit,UnitQuaternion,Translation3}` → `leto::geometry::*` across 57
    files; map `nalgebra::distance_squared(a,b)` → `a.distance_squared(b)` (×6),
    drop `&` on `from_axis_angle(&axis,…)`; `.cholesky()` (×5) → `leto-ops`;
    drop nalgebra + num_traits; build + run gaia's full suite (geometry
    correctness is the gate). (leto/hephaestus keep nalgebra as a *dev*-oracle —
    out of scope.)
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

Note: leto/hephaestus retain `num-complex` as a **dev-dependency** (nalgebra
differential oracle returns `num_complex::Complex`) — that is correct and out of
scope for removal.
