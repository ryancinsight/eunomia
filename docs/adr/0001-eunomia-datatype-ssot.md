# ADR 0001: Eunomia — the datatype-law foundation repo

- Status: Accepted
- Date: 2026-06-29
- Class: [arch] (new foundation repo; promotes the numeric vocabulary out of
  hermes and rewires every stack consumer)

## Context

`hermes-numeric` declared itself *"Single Source of Truth for all numeric
representations"* and owned the scalar wrapper types, packed sub-byte formats,
conversion lattices, and (after the num-complex removal) `Complex<T>`. It is a
clean leaf — its only deps are `half`, `bytemuck`, `libm`, optional `rkyv`.

But it lived **inside the hermes workspace**, alongside seven `hermes-simd-*`
crates. Two problems follow:

1. **Release-cadence coupling.** The stable datatype vocabulary shared hermes's
   version with the perf-volatile SIMD kernels. Evidence this already hurts: the
   themis-0.9 hermes line regressed dense matmul 43–61% at 64²/128² (recorded in
   leto's `Cargo.toml` / `gap_audit.md §D`), so consumers could not take a
   numeric-vocabulary update without risking a SIMD perf regression.
2. **Bounded-context blur.** "Datatype vocabulary" and "SIMD execution" are
   distinct concerns; the simd crates depend on the numeric crate, never the
   reverse — a one-way edge that wants a clean repo boundary.

## Decision

Promote the numeric vocabulary to its own independently versioned, publicly
published foundation repo, **eunomia** (datatype law — sibling of themis, the
placement law):

- `crates/eunomia` — the migrated `hermes-numeric` content verbatim (scalars,
  complex, packed, casts, ops, element traits), package renamed `hermes-numeric`
  → `eunomia`.
- `crates/eunomia-gpu` — GPU datatype vocabulary: one const-generic
  `Vector<T, const N>` (`repr(C)`/`Pod`, std430/tightly-packed) with `Vec2`/
  `Vec3`/`Vec4` dimension aliases, the device-buffer element types compute
  backends need. Started here because adding GPU vector types to the numeric
  crate would re-create the cadence coupling.

Every consumer (`hermes-simd-*`, leto, coeus, hephaestus, …) switches its
`hermes-numeric` dependency and `hermes_numeric::` paths to `eunomia`; the
`hermes-numeric` crate is removed from hermes. No compatibility re-export is
kept (per the no-shim rule) — call sites update in the same migration.

## Alternatives rejected

- **Keep it in hermes, version the crate independently within the workspace**:
  rejected — workspace members share the repo's release/tag cadence in practice,
  and the bounded-context boundary stays blurred. The cadence-decoupling trigger
  (problem 1) is the mechanical promotion signal per the scope ladder.
- **Add GPU types into `hermes-numeric`**: rejected — re-creates the coupling
  this split exists to remove.

## Consequences

- One independently versioned datatype foundation at the bottom of the DAG,
  consumed by the whole stack; the SSOT is unchanged in content, only relocated.
- Verified: migrated content builds + all tests pass standalone as `eunomia`
  (2 + 11 + doctests); `eunomia-gpu` 3 tests green.
- Follow-ups: rewire stack consumers (tracked); consolidate coeus-core's own
  `Complex<T>` (with coeus `Scalar`/`FloatOps` impls) onto `eunomia::Complex`.
