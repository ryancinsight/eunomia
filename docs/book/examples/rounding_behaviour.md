# Example: Rounding Behaviour

**Crate**: `eunomia`
**Planned source**: `crates/eunomia/examples/book_rounding_behaviour.rs` (lands with the chapter as a DoR item)

## What This Example Will Demonstrate

Round-to-nearest, ties-to-even in action: converting a grid of `f32` values
through `narrow::<E, M>` into `F16`/`Bf16` bit patterns and confirming both
the half-ulp error bound and the ties-to-even rule at exact midpoints.

## Key API Surface

- `eunomia::convert::{narrow, widen}` (chapter 9)
- `F16`/`Bf16` bit patterns via the transparent `u16` storage
- `FloatElement::from_f64` precision-correct construction

## Outline

- Convert `1.0f32` and small powers of two, asserting exact
  round-trips (`0x3C00` for `F16`)
- Feed exact ties (values exactly midway between two representable
  values) and show the even-significand choice
- Measure the maximum relative error across a sampled range against the
  half-ulp bound
- Discussion: why RNE is the default and where a truncated cast would
  have silently biased the result
