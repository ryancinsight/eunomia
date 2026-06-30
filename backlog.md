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
- **BLOCKER for the rest of apollo — ndarray → leto.** apollo-fft *tests* (88
  errors) and likely sibling crates use `ndarray::Array::zeros()`, which requires
  the element `A: num_traits::Zero`. eunomia::Complex cannot impl num_traits::Zero
  (orphan rule; eunomia avoids the num_traits dep). So the full apollo migration
  requires replacing **ndarray with leto** for complex-element arrays — the leto
  integration in scope for this campaign, a large additional layer. Until then
  apollo cannot fully drop num-complex/num-traits, and apollo-fft cannot merge
  (the ~19 consumer crates + tests keep the workspace red).

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

Note: leto/hephaestus retain `num-complex` as a **dev-dependency** (nalgebra
differential oracle returns `num_complex::Complex`) — that is correct and out of
scope for removal.

## Cleanup (redundancy removal)

- **E-019 [patch]** Removed the dead `eunomia-gpu` crate: it had zero external
  consumers and its `Vector<T,N>` was redundant with `leto::geometry::Vector`
  (also `repr(C)`/Pod, GPU-buffer-compatible). Per the corrected architecture
  (eunomia = datatypes; vectors/geometry = leto CPU / hephaestus GPU), the
  GPU-vector concern does not belong in the datatype crate. eunomia is now a
  single-crate workspace. README + diagram synced.
