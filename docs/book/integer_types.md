# 2. Integer Scalar Types

## Governing equations

An integer scalar is an exact value in a fixed-width two's-complement
representation over `N` bits, ranging over

$$[-2^{N-1},\, 2^{N-1}-1].$$

Eunomia ships `I8`, `I16`, and `I32` — the integer element of the numeric
vocabulary. Unlike floats, integers carry no rounding error; their
operations are exact up to the wrap boundary.

## The crate's abstraction

Each integer type is a `#[repr(transparent)]` wrapper over the corresponding
Rust primitive:

```rust,ignore
pub struct I8(pub i8);
pub struct I16(pub i16);
pub struct I32(pub i32);
```

- **`NumericElement` membership.** Integers implement the same element trait
  as floats, so generic kernels can be written once over `NumericElement`
  and instantiate for `I8`/`I16`/`I32` without type-suffix duplication.
- **Exact `Eq`/`Ord`.** Unlike the float wrappers (which are
  float-semantic), the integer wrappers derive `Eq`, `Ord`, and `Hash`.
- **Source construction.** Integer callers use the literal `v as Self`
  truncating cast; there is deliberately no generic `from_usize` on
  `NumericElement`, so callers express precision-correct construction
  explicitly.
- **`Pod`/`Zeroable`.** Every integer wrapper is plain-old-data (§10) and
  carries exact layout guarantees.

## Outline of this chapter

- Two's-complement representation and the wrap boundary
- Why integers are exact while floats are approximate
- The wrapper contract: transparent storage, exact equality/order, `Pod`
- Integer construction discipline: literal casts, not a generic `from_usize`
- Generic kernels over `NumericElement` instantiating for integer scalars
