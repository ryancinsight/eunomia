# ADR 0004: Provider-Owned Unit Scalar Scaling

- Status: Accepted
- Date: 2026-07-28
- Class: [minor]

Revision 2026-09-21 ([major]): add native inverse scaling for
[EUNOMIA-UNIT-DIVISION](../../backlog.md#EUNOMIA-UNIT-DIVISION).
Multiplication by a reciprocal can overflow before applying it to a
subnormal value. `UnitScalar::divide_by_factor` divides in the storage precision,
componentwise for complex values. External implementations must add this
required method; manifests are not bumped until an authorized release.

Revision 2026-09-21 (cont., [patch]): the new method's first name,
`divide_by_f64`, embedded the coefficient's type in the identifier —
the same defect the sibling `scale_by_f64` already carried. Both rename to
`divide_by_factor`/`scale_by_factor` for
[EUNOMIA-TYPE-SUFFIXED-UNIT-METHODS-2026-09-21](../../backlog.md#eunomia-type-suffixed-unit-methods-2026-09-21):
the coefficient's concrete `f64` precision is a real, ADR-mandated contract
(native storage-precision division, never a widen/narrow), so it stays in
the signature (`factor: f64`); it does not need restating in the name. Since
neither name had shipped in a release, this is documentation and identifier
churn only — no version or migration-note change beyond this ADR and the
"# Migration" doc comment.

## Context

Aequitas linear-unit conversion must support Eunomia's reduced-precision real
storage types and native complex phasors without defining a second scalar
vocabulary or overlapping real/complex blanket implementations. A complex
phasor carries one physical dimension: its real and imaginary components are
scaled together, while the imaginary component remains quadrature data.

## Decision

Eunomia owns `UnitScalar`, with native `scale_by_factor` and `divide_by_factor` operations. Implementations
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
