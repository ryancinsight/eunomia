# 6. Scalar Fields: RealField and ComplexField

## Governing equations

A *scalar field* is the algebraic setting in which linear algebra and
geometry are expressed. For real numbers the field carries the total order
and the standard constants:

$$\pi,\quad 2\pi,\quad \pi/2,\quad e,\quad \ln 2,\quad \sqrt{2},\quad \varepsilon_{\text{mach}}.$$

For complex numbers the same operations exist, but the order does not — the
field is the pair `(re, im)` with componentwise arithmetic.

## The crate's abstraction

`RealField` and `ComplexField` are eunomia's replacement for
`nalgebra::RealField` / `nalgebra::ComplexField`. They let generic numeric
and geometric code be written once over "a real scalar" or "a real-or-complex
scalar" without pulling in nalgebra:

```rust,ignore
pub trait RealField: FloatElement + PartialOrd + Neg<Output = Self> {
    const PI: Self;
    const TAU: Self;
    const FRAC_PI_2: Self;
    const E: Self;
    const LN_2: Self;
    const SQRT_2: Self;
    const EPSILON: Self;

    fn infinity() -> Self;
    fn neg_infinity() -> Self;
    fn nan() -> Self;
    fn min_value() -> Self;
    // ...
}
```

- **Ordered real scalars.** `RealField` adds the total order (`PartialOrd`),
  the mathematical constants, and sign helpers to the `FloatElement` math
  surface. Implemented for `f32`/`f64` (and the reduced float wrappers).
- **Real-or-complex.** `ComplexField` is the superset abstraction for code
  that must run over both real and complex scalars.
- **Boundary discipline.** Eunomia owns only the *scalar field vocabulary*.
  The linear algebra built on top (matrices, decompositions) lives in `leto`
  (CPU) / `hephaestus` (GPU) — this trait is their scalar contract.

## Outline of this chapter

- The algebraic definition of a field and why constants matter
- `RealField`: order + constants + sign helpers on the `FloatElement` surface
- `ComplexField`: the real-or-complex superset
- Boundary discipline: scalar vocabulary here, linear algebra in `leto`/`hephaestus`
- Generic geometry code written once over `RealField`, instantiated for
  `F32` and `F64`
