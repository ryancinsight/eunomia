# Eunomia backlog

Sprint target: 0.1.0 (datatype-law foundation extracted from hermes-numeric).

## Done

- **E-001 [arch]** Scaffold eunomia workspace; migrate `hermes-numeric` content
  verbatim → `crates/eunomia` (scalars, complex, packed, casts, ops, traits).
  Builds + all tests pass standalone. — ADR 0001.
- **E-002 [minor]** `eunomia-gpu`: const-generic `Vector<T, N>` (`Vec2`/`Vec3`/
  `Vec4`), `repr(C)`/`Pod`, std430 storage layout. 3 tests green.

## In progress / next (cross-repo migration — co-evolution)

- **E-003 [arch]** Rewire `hermes-simd-*` off `hermes-numeric` → `eunomia`
  (`hermes_numeric::` → `eunomia::`, dep + path-patch swap); remove
  `crates/hermes-numeric` from hermes. Owner: this migration.
- **E-004 [arch]** Rewire `leto` (`pub use hermes_numeric::Complex` →
  `eunomia::Complex`, dep + patch) and `coeus` `hermes-numeric` reference.
- **E-005 [patch]** Update atlas meta-repo README stack table: add eunomia
  (datatype law) as a foundation row.

## Follow-ups (filed)

- **E-006 [arch]** Consolidate `coeus-core::Complex` (carries coeus-specific
  `Scalar`/`FloatOps`/`CpuUnaryDispatch` impls) onto `eunomia::Complex` — move
  the trait impls, re-export. Closes the last duplicate native complex type.
- **E-007 [minor]** GPU layout vector types grow as a backend consumer appears;
  std140 UBO alignment stays a buffer-packing concern, not a type property.
