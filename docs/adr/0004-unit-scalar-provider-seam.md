# ADR 0004: Provider-Owned Unit Scalar Scaling

- Status: Accepted
- Date: 2026-07-28
- Class: [minor]

## Context

Aequitas linear-unit conversion must support Eunomia's reduced-precision real
storage types and native complex phasors without defining a second scalar
vocabulary or overlapping real/complex blanket implementations. A complex
phasor carries one physical dimension: its real and imaginary components are
scaled together, while the imaginary component remains quadrature data.

## Decision

Eunomia owns `UnitScalar`, with a native `scale_by_f64` operation. Implementations
cover every shipped real `FloatElement` storage type and `Complex32`/`Complex64`.
Aequitas binds one generic quantity-conversion path to this provider seam. No
imaginary-unit type or separate physical dimension is introduced.

## Rejected alternatives

- Separate Aequitas real and complex inherent conversion methods: rejected
  because overlapping impls are not a valid extensible API boundary.
- Aequitas-owned scalar conversion trait: rejected because Eunomia owns scalar
  representations and native precision rules.
- Treat the imaginary component as a second unit: rejected because it is
  quadrature of the same observable, not a physical dimension.

## Verification

- Eunomia tests scale a complex value componentwise and verify real embedding.
- Aequitas tests round-trip a complex length through a kilometer unit and
  derive complex electrical impedance from potential/current quantities.
- Kwavers consumer contracts use typed pressure phasors and electrical
  impedance while retaining raw complex values only at formula boundaries.
