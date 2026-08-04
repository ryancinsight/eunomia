# Example: Complex Arithmetic in a Solver

**Crate**: `eunomia`
**Planned source**: `crates/eunomia/examples/book_complex_arithmetic.rs` (lands with the chapter as a DoR item)

## What This Example Will Demonstrate

The `Complex<T>` vocabulary in a solver-shaped loop: constructing `Complex32`
and `Complex64` values, applying the field operators, and crossing the
GPU/FFI boundary through the `Pod`/`Zeroable` contract.

## Key API Surface

- `Complex32` / `Complex64` aliases (chapter 3)
- `re` / `im` fields and the `#[repr(C)]` layout
- Field operators `Add`/`Sub`/`Mul`/`Div`/`Neg`
- `bytemuck::cast_slice` round-trip through raw bytes (chapter 10)

## Outline

- Build a complex phasor `re + im·i` from real measurements
- Apply a complex product and quotient, checking results with
  `assert_relative_eq!`
- Cast a slice of `Complex64` to bytes and back via the `Pod` contract
- Discussion: the quadrature rule — the imaginary component is quadrature,
  never a second physical unit (chapters 3 and 7)
