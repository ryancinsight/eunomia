# ADR 0005: Real-scalar minimum and maximum special values

- Status: Accepted
- Date: 2026-08-21
- Class: [major] [arch]
- Driver: `ATLAS-EUNOMIA-NAN-CONTRACT-2026-08-21`

## Context

`NumericElement::min_scalar` and `max_scalar` are consumed by generic
reductions and by `RealField::clamp`. The former defaulted to operand-order
dependent `PartialOrd` checks. Primitive `f32` and `f64` used native operations,
while reduced-precision wrappers inherited the default. `PartialOrd` is the
correct comparison surface for NaN, but it is not a complete min/max value
contract; it also does not specify which signed zero survives an equal-value
tie.

The split made one-NaN results depend on operand order and left signed-zero
results unspecified across the shipped real scalar representations.

## Decision

`NumericElement` owns one value contract for all real floating-point
implementations:

| Inputs | `min_scalar` | `max_scalar` |
| --- | --- | --- |
| one NaN and one non-NaN | non-NaN | non-NaN |
| two NaNs | NaN | NaN |
| `-0` and `+0` | `-0` | `+0` |
| all other values | numeric minimum | numeric maximum |

The default implementation checks the NaN predicate and sign bit in the
element's native representation, then uses `PartialOrd` for ordinary values.
It does not widen or narrow a value. Primitive `f32` and `f64` retain native
overrides because their operations implement the same table. Wrapper types use
the one trait default. `Complex<T>` keeps its explicit lexicographic
implementation and is not a real scalar.

`RealField::clamp` inherits the table through its existing min/max composition:
a NaN input reduces to the lower bound, and a NaN bound is ignored.

## Alternatives rejected

- Propagate any NaN from min/max: rejected because it changes the existing
  native float behavior and makes reduction results depend on invalid samples
  rather than the available numeric value.
- Define min/max directly with `PartialOrd`: rejected because one NaN is
  unordered and because `max(-0, +0)` needs an explicit tie rule.
- Add per-wrapper overrides: rejected because it duplicates the same law and
  allows a new scalar implementation to drift from the provider contract.
- Widen wrappers to `f32` for comparison: rejected because the operation must
  preserve native precision and representation.

## Consequences

- Reductions and clamps have operand-order-independent real-scalar semantics.
- NaN comparison remains unordered; this decision does not add `Eq` or `Ord`.
- The trait default performs the special-value checks for wrapper types, while
  primitive native overrides retain the existing zero-cost path.
- Consumers do not need API changes; consumer validation is still required at
  their next exact-head convergence.

## Verification

`crates/eunomia/tests/float_order.rs` instantiates the contract for every
shipped real scalar type and checks both operand orders, two-NaN behavior,
signed-zero bit selection, and generic clamp composition. It also checks
`RealField::clamp` directly for primitive real fields. Provider format,
warning-denied Clippy, Nextest, doctest, Rustdoc, package, and hosted gates are
the remaining acceptance evidence for the implementation revision.
