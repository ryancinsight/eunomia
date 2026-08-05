# Example: Choosing a Precision

**Crate**: `eunomia`
**Source**: `crates/eunomia/examples/book_choosing_precision.rs`

Accumulating the same series in `F64`, `F32`, `F16`, and `Bf16` makes the
storage/accuracy trade-off concrete.  Each step is precision-correct: the
`FloatElement::from_f64` constructor is the single entry point for
constant-folded literals, and `assert_relative_eq!` enforces the expected
error budget rather than relying on an exact comparison.

## Source

```rust
# extern crate eunomia;
{{#include ../../../crates/eunomia/examples/book_choosing_precision.rs}}
```

## Output

```text
harmonic sum of the first 256 terms
 F64: 8 bytes, sum = 6.124344963, relative error = 0.000e0
 F32: 4 bytes, sum = 6.124345779, relative error = 1.333e-7
 F16: 2 bytes, sum = 6.085937500, relative error = 6.271e-3
Bf16: 2 bytes, sum = 5.062500000, relative error = 1.734e-1
```

## What to notice

| Scalar | Storage | Relative error |
|--------|---------|----------------|
| `F64`  | 8 bytes | reference      |
| `F32`  | 4 bytes | ~1.3 × 10⁻⁷   |
| `F16`  | 2 bytes | ~6.3 × 10⁻³   |
| `Bf16` | 2 bytes | ~1.7 × 10⁻¹   |

`Bf16` accumulates only 7 mantissa bits, so its harmonic sum diverges from
the double-precision reference by 17 %.  That is the intended result, not a
bug: the tolerance in `assert_relative_eq!` is set to `2.0e-1` specifically
to document the expected worst-case error for this accumulation length.

Use `F32` or `F64` for accumulation and iterative solvers.  Reach for `F16`
or `Bf16` only for weight storage, activations, or communication buffers
where bandwidth dominates accuracy.  The `size_of::<T>()` line confirms the
storage cost without any unsafe cast — it is a property of the Rust type, not
a runtime branch.
