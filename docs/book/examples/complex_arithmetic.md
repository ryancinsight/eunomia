# Example: Complex Arithmetic in a Solver

**Crate**: `eunomia`
**Source**: `crates/eunomia/examples/book_complex_arithmetic.rs`

`Complex32` and `Complex64` are `#[repr(C)]` pairs of real and imaginary
components.  That layout makes them `bytemuck::Pod`, which in turn lets
`bytemuck::cast_slice` round-trip them to raw bytes and back without any
unsafe code.  This is the property that lets the GPU staging layer in
`hephaestus` and the FFI layer in `kwavers-python` handle complex buffers as
plain byte slices.

## Source

```rust
# extern crate eunomia;
# extern crate bytemuck;
{{#include ../../../crates/eunomia/examples/book_complex_arithmetic.rs}}
```

## Output

```text
phasor = 3+4i, response = -4+3i, norm = 5
marshalled 2 Complex32 values as 16 bytes
```

## What to notice

- Multiplying by `i` rotates the phasor 90 ° counter-clockwise: `3+4i` →
  `-4+3i`.  Dividing by `i` undoes the rotation.  Both operations are checked
  with `assert_relative_eq!` rather than `==` because floating-point
  multiplication does not commute with exact inversion at the bit level.

- Two `Complex32` values occupy exactly 16 bytes: `2 × (4 + 4)`.
  `cast_slice::<Complex32, u8>` confirms this at runtime without any manual
  size calculation.

- The `Display` output for `3+4i` comes from `Complex64`'s `fmt::Display`
  impl, which writes the imaginary part as `+4i` when positive.

## The quadrature rule

A `Complex<T>` value represents one observable in quadrature — `re` and `im`
are the in-phase and quadrature components of a single physical quantity.
They are **not** two independent physical units.  Storing a pressure
magnitude in `re` and a temperature in `im` would violate Eunomia's
one-observable-unit contract (chapter 7) even though the code would compile.
