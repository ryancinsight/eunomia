# Example: Choosing a Precision

**Crate**: `eunomia`
**Planned source**: `crates/eunomia/examples/book_choosing_precision.rs` (lands with the chapter as a DoR item)

## What This Example Will Demonstrate

The precision trade-off across the eunomia scalar vocabulary: computing the
same accumulated sum in `F32`, `F64`, `F16`, and `Bf16`, and reporting the
round-trip error against the `f64` reference.

| Scalar | Storage | Relative error vs `f64` |
|---|---|---|
| `F64` | `u64` | reference |
| `F32` | `u32` | ~1e-7 (single precision) |
| `F16` (binary16) | `u16` | ~1e-3 (half precision) |
| `Bf16` (E8M7) | `u16` | ~4e-3 (bfloat range) |

## Key API Surface

- `NumericElement::from_f64` / `FloatElement::from_f64` — precision-correct
  construction
- `assert_relative_eq!` — blended tolerance comparison (chapter 12)
- The `(E, M, B)` format table from chapter 1

## Outline

- Accumulate a known series in each precision
- Construct each scalar precision-correctly via `FloatElement::from_f64`
- Compare against the `f64` reference with `assert_relative_eq!`
- Report storage cost and error per format
- Discussion: when reduced precision is the right call (bandwidth-bound
  kernels, storage) versus when it is not (accumulation, ill-conditioning)
