# 9. The Native Conversion Kernel

## Governing equations

Converting between IEEE-754 formats of different widths is the operation
that decides whether a reduced-precision pipeline loses accuracy silently.
The correct rounding rule is round-to-nearest, ties-to-even (RNE): a real
value is mapped to the nearest representable value, and exact ties map to
the neighbour with an even significand. This bounds the conversion error at
half an ulp of the destination format — the smallest achievable.

## The crate's abstraction

Eunomia's conversion module is the native soft-float conversion SSOT — one
generic const-parameterized kernel converts between `f32` and any reduced
IEEE-754 binary format `(E, M)`:

```rust,ignore
use eunomia::convert::{narrow, widen};

// binary16 round-trips 1.0 exactly.
let one_f16 = narrow::<5, 10>(1.0f32.to_bits());
assert_eq!(one_f16, 0x3C00);
assert_eq!(f32::from_bits(widen::<5, 10>(one_f16)), 1.0);
```

| Format | `E` | `M` | Bias |
| --- | --- | --- | --- |
| `binary16` (`F16`) | 5 | 10 | 15 |
| `bfloat16` (`Bf16`) | 8 | 7 | 127 |

- **One implementation, all precisions.** `narrow::<E, M>` / `widen::<E, M>`
  are const-parameterized over the format geometry, so every precision shares
  one authoritative, monomorphized implementation — including the sub-byte
  formats' conversions.
- **RNE guaranteed.** Round-to-nearest, ties-to-even is baked into the
  kernel; finite-variant kernels (`narrow_finite`/`widen_finite`) are
  provided where the format has no infinity/NaN encoding.
- **Replacement for `half`.** This kernel replaces `half`'s `f16`/`bf16`
  conversions; `half` remains only as a dev-only differential oracle in
  eunomia's own conversion tests.

## Outline of this chapter

- Round-to-nearest-ties-to-even and the half-ulp error bound
- Format geometry as const parameters `(E, M)`
- `narrow`/`widen` and the finite variants
- Why one kernel replaces the `half` crate (and how `half` survives as a
  test oracle)
- Worked example in [Rounding Behaviour](examples/rounding_behaviour.md)
