# 13. Element Operations

## Governing equations

Element operations are the pointwise arithmetic of the scalar vocabulary —
the `Add`/`Sub`/`Mul`/`Div` (and their `Assign` forms) that kernels assume
of any element. For floats these are IEEE-754 operations with their rounding
rules; for integers, two's-complement wrap arithmetic; for complex values,
the field operations of §3.

## The crate's abstraction

The `ops` module provides the operation impls across the vocabulary:

- `ops::floats` — arithmetic for the float wrappers (`F16`/`Bf16`/`F32`/
  `F64` and the sub-byte formats), with float-semantic ordering.
- `ops::ints` — arithmetic for `I8`/`I16`/`I32`, with exact wrap semantics.

Together with the operator supertraits on `NumericElement` (§4), this gives
generic kernels a complete, uniform arithmetic surface:

```rust,ignore
fn l2_norm_sq<T: NumericElement>(x: T, y: T) -> T {
    x * x + y * y   // Add, Mul, and AddAssign are assumed by the trait
}
```

## Outline of this chapter

- The element arithmetic surface: `Add`/`Sub`/`Mul`/`Div`/`Assign`
- Float operations and their rounding rules
- Integer wrap arithmetic and why it is exact
- Complex field operations
- Building kernels over the operator supertraits of `NumericElement`
