# 14. Position in the Atlas Stack

## The layering

Eunomia is the foundation of the Atlas dependency graph — the datatype law
one rung below the placement law:

```text
eunomia        datatype law (scalars, complex, packed, field traits)   ← this repo
themis         placement law (NUMA, tier, worker locality)
   ↑ consumed by
hermes         SIMD execution over eunomia types
mnemosyne      allocation
moirai         execution
   ↑ consumed by
leto           array substrate (cache-tiled over hermes/eunomia)
   ↑ consumed by
coeus / hephaestus / apollo / …   domain
```

Dependency direction is strictly inward: eunomia depends on nothing
Atlas-local. Its core uses `bytemuck` and `libm`; `rkyv`, `serde`, and the
NumPy/PyO3 element boundary are optional. The `half` crate is a dev-only
differential oracle for the reduced-precision conversion tests.

## What this buys the stack

- **One numeric vocabulary.** Every Atlas crate computes over the same
  scalar types, element traits, and conversion kernels — no duplicated
  representations, no precision drift between crates.
- **Zero-cost generics.** Kernels are written once over `NumericElement`/
  `FloatElement`/`RealField`/`ComplexField` and monomorphized per precision;
  the sealed trait keeps the implementor set closed and exhaustive.
- **Stable datatype law.** Independently versioned and published, so the
  vocabulary can advance without dragging the SIMD/execution layers'
  cadence (the release-cadence coupling that motivated
  [ADR 0001](../adr/0001-eunomia-datatype-ssot.md)).

## Governance

- [ADR 0001](../adr/0001-eunomia-datatype-ssot.md) — the datatype-law foundation
- [ADR 0002](../adr/0002-native-complex-provider-cutover.md) — native `Complex<T>`
- [ADR 0003](../adr/0003-native-byte-layout-and-reduced-precision.md) — native
  layout and reduced precision
- [ADR 0004](../adr/0004-unit-scalar-provider-seam.md) — the `UnitScalar` seam

## Outline of this chapter

- The stack diagram and the inward dependency rule
- What one vocabulary buys: no duplicated representations, no precision drift
- Sealed traits and closed-set dispatch
- Independent versioning vs the old cadence coupling
- The ADR trail governing the datatype law
