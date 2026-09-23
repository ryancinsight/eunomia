# ADR 0006: Binary scaling in FloatElement

- Status: Accepted
- Date: 2026-09-23
- Item: [EUNOMIA-BINARY-SCALE-001](../../backlog.md#EUNOMIA-BINARY-SCALE-001)
- Stack contract: [ATLAS-GAIA-NURBS-SCALE-001](https://github.com/ryancinsight/atlas/pull/268)

## Context

GAIA's rational derivative terms can have a finite result even when an
intermediate weight ratio overflows. The operation needs format-owned binary
exponent extraction and power-of-two scaling so the consumer can normalize
every product factor before multiplying or dividing. `FloatElement` owns scalar
operations and is sealed; adding this capability there prevents downstream
float decomposition from being duplicated.

A conventional `frexp` result uses a significand in `[0.5, 1)`. That form
cannot be returned exactly as `Self` for every shipped format: E2M1 `Bf4`
represents `1.5` but not its `frexp` significand `0.75`.

## Decision

Add `binary_exponent`, returning `None` for zero or non-finite inputs and
otherwise the floor of `log2(abs(x))`. For the returned exponent `e`, scaling
the input by `2^-e` produces a significand with magnitude in `[1, 2)`. Add
`scale_binary` for scaling by an integer power of two.

The default implementation uses the locked `libm` f32 exponent and scaling
operations. This is exact for primitive `f32`, the `F32` wrapper, and every
reduced format because their values are exactly representable in f32. Primitive
`f64` and `F64` override both methods with native f64 operations. Representable
scaling is exact; overflow, underflow, NaN, and signed-zero behavior follow each
format's existing conversion contract.

## Alternatives rejected

- Return the conventional `frexp` pair: `Bf4` cannot represent every
  `[0.5, 1)` significand.
- Decompose values separately in each consumer: this duplicates scalar-format
  rules outside the sealed provider contract.
- Route all formats through f64: this changes native f32 arithmetic and hides
  the representation's actual rounding contract.

## Consequences

The two defaulted trait methods are additive because `FloatElement` is sealed.
Tests instantiate positive and negative normalization and round trips across
every implementation, check each reduced format's smallest nonzero encoding
(including subnormals where supported), and exercise native
f32/f64 exponent limits, underflow ties, signed zero, NaN, and infinity. The
API does not promise cancellation-safe summation or prevent overflow in a
coordinate subtraction; those remain consumer-level numerical obligations.

Overturning evidence: a shipped scalar implementation cannot normalize every
finite nonzero value into an exactly representable `[1, 2)` significand, or
the locked provider scaling operation fails the documented format semantics.
