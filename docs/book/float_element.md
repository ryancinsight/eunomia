# 5. FloatElement: The Transcendental Surface

## Governing equations

Real transcendental functions — the exponentials, logarithms, and
trigonometric functions — form the mathematical surface a solver expects of
a float scalar:

$$e^{x},\quad \ln x,\quad \sin x,\quad \cos x,\quad \tanh x,\ldots$$

The accuracy of these functions across precisions is the *precision
contract*: `f64` must use native double-precision `libm` functions, while the
reduced formats route through `f32` (the correct reduced-precision path,
since reduced formats have no hardware transcendentals).

## The crate's abstraction

`FloatElement` extends `NumericElement` with float-specific conversions and
the real transcendental surface:

```rust,ignore
pub trait FloatElement: Sealed + NumericElement {
    fn from_f32(val: f32) -> Self;
    fn from_f64(val: f64) -> Self;
    fn to_f32(self) -> f32;

    fn exp(self) -> Self;   // e^self
    fn ln(self) -> Self;    // natural logarithm
    fn sin(self) -> Self;   // sine, radians
    fn cos(self) -> Self;   // cosine, radians
    // ... the rest of the transcendental surface
}
```

- **Default routing through `f32`.** The default implementations widen to
  `f32`, call the `libm` function, and narrow back — native for `f32`, and
  the correct reduced-precision path for `f16`/`bf16`.
- **`f64` overrides.** `f64` overrides each with the native double-precision
  `libm` function, so it is never widen-narrowed.
- **Precision-correct construction.** `from_f64` is where `F16`/`Bf16`
  round through the native kernel rather than a truncating `as` cast.

## Outline of this chapter

- The transcendental surface and its precision contract
- Default `f32` routing vs native `f64` overrides
- Precision-correct `from_f32`/`from_f64` construction
- `to_f32` and the widening boundary
- Generic math kernels over `FloatElement`, instantiated across every
  float scalar
