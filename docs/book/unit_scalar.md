# 7. UnitScalar: The Physical-Unit Seam

## Governing equations

Physical quantities are real values scaled by an SI-unit coefficient:

$$Q = u \cdot s,$$

where `u` is the unit (metres, seconds, pascals…) and `s` is the scalar in
that unit. When a downstream physical-quantity crate (`aequitas`) types a
quantity, it needs one provider-defined path to scale a scalar by a real
coefficient — for real storage types *and* for complex phasors.

## The crate's abstraction

`UnitScalar` is the seam that makes that path single-source:

```rust,ignore
pub trait UnitScalar: Copy {
    /// Scale this value by a real coefficient in the scalar's native precision.
    fn scale_by_f64(self, factor: f64) -> Self;
}
```

- **Owned by eunomia, used by aequitas.** The conversion is intentionally
  owned here so downstream physical-quantity crates use one provider-defined
  path for real storage types and complex phasors.
- **Componentwise complex scaling.** A complex value is scaled
  componentwise; its imaginary component is quadrature, *not* a second
  physical unit — the Eunomia real/complex rule that keeps SI units real.
- **Native precision.** The coefficient is applied in the scalar's native
  precision, so `F16`/`Bf16` scale through their own arithmetic rather than
  being widened first.

## Outline of this chapter

- Quantities as `unit × scalar` and why the conversion belongs in the
  datatype law
- The `scale_by_f64` contract and native-precision application
- The quadrature rule: complex imaginary parts are quadrature, never a
  second physical unit
- How `aequitas` consumes the seam (cross-reference to the aequitas book)
