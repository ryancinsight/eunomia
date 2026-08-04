# 8. The Cast Lattice: CastFrom and CastTo

## Governing equations

Converting a value from one numeric representation to another is a *cast*.
Rust's `as` operator provides the primitive semantics: float-to-integer
conversions truncate toward zero and saturate values outside the destination
range. The lattice problem is to express "cast any scalar to any other
scalar" generically, so a kernel can accept an element of one precision and
deliver an element of another without naming every pair.

## The crate's abstraction

`CastFrom`/`CastTo` form the generic casting lattice:

```rust,ignore
pub trait CastFrom<T>: Copy {
    fn cast_from(val: T) -> Self;
}

pub trait CastTo: Copy {
    fn cast_to<U>(self) -> U
    where
        U: CastFrom<Self>,
    {
        U::cast_from(self)
    }
}
```

- **Primitive semantics preserved.** Primitive numeric implementations
  follow Rust's `as` conversion semantics — truncation toward zero for
  float-to-integer, saturation outside the destination range.
- **One direction, both spellings.** `CastFrom` is the primitive direction;
  `CastTo` is the blanket-enabled reverse spelling (`T::cast_from(self)`)
  so callers can write `x.cast_to::<F32>()` or `F32::cast_from(x)`.
- **Lattice, not lossless promise.** A cast may lose precision — the lattice
  says *how*, deterministically, not that it cannot lose. Precision-correct
  conversions between float formats go through the native kernel (§9).

## Outline of this chapter

- The cast lattice: every scalar to every scalar, generically
- `as` semantics: truncation and saturation on the primitive boundaries
- `CastFrom` as the primitive direction, `CastTo` as the reverse spelling
- Lattice vs lossless conversion — when to use `cast_from` vs the native
  kernel
- Generic kernels converting between precisions at their boundaries
