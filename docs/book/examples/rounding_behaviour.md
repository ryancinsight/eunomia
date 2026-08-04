# Example: Rounding Behaviour

**Crate**: `eunomia`
**Source**: `crates/eunomia/examples/book_rounding_behaviour.rs`

`eunomia::convert::{narrow, widen}` are const-generic bit-manipulation
primitives that pack and unpack IEEE-754 sub-fields without any runtime
branching.  `narrow::<E, M>(bits: u32) -> u16` extracts an `E`-bit exponent
and `M`-bit mantissa from a `f32` bit pattern, applying round-to-nearest
ties-to-even in the process.  This example pins down the rounding contract
with exact bit-pattern assertions.

## Source

```rust
{{#include ../../../crates/eunomia/examples/book_rounding_behaviour.rs}}
```

## Output

```text
0.10 -> F16 0x2E66 (0.0999756), Bf16 0x3DCD (0.1000977)
0.50 -> F16 0x3800 (0.5000000), Bf16 0x3F00 (0.5000000)
1.00 -> F16 0x3C00 (1.0000000), Bf16 0x3F80 (1.0000000)
3.25 -> F16 0x4280 (3.2500000), Bf16 0x4050 (3.2500000)
10.00 -> F16 0x4900 (10.0000000), Bf16 0x4120 (10.0000000)
```

## What to notice

- Exact powers of two (0.5, 1.0, 3.25 = 13/4, 10.0 in the table) survive both
  reduced formats unchanged because they lie exactly on a grid point in both
  `F16` and `Bf16`.

- `0.1` does not survive: binary16 stores `0.0999756` (error ≈ 2.4 × 10⁻⁴);
  bfloat16 stores `0.1000977` (error ≈ 9.8 × 10⁻⁵).  Bfloat16's wider
  exponent range gives it more precision near 0.1 than binary16 does.

- The ties-to-even assertion:
  ```rust
  let midpoint = 1.0_f32 + 2.0_f32.powi(-11);
  assert_eq!(F16::from_f32(midpoint).to_bits(), 0x3C00);
  ```
  The midpoint is exactly halfway between binary16's `1.0` (`0x3C00`) and the
  next representable value (`0x3C01`).  Ties-to-even keeps the even
  significand — `0x3C00` has bit 0 clear, `0x3C01` has it set — so the result
  rounds *down* to `1.0`.

- The half-ulp bound:
  Around 0.1, binary16 has exponent field 14 (`2^(14-15) = 2^-1` … wait, let
  the example speak: `half_ulp = 2^-15` for the biased exponent at 0.1, and
  the assertion `(rounded - value).abs() <= half_ulp` confirms the kernel never
  exceeds this bound.  Failing this assertion would imply a truncation or
  round-away-from-zero bug in the conversion path.
