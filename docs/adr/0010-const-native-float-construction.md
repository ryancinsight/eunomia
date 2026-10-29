# ADR 0010: Canonical const native float construction

Status: Accepted  \
Date: 2026-10-15  \
Change class: [minor]  \
Delivery: EUNOMIA-CONST-NATIVE-FLOAT
Revision 2026-10-29: the delivery version moved from 0.11.0 to 0.12.0 because
0.11.0 was published on 2026-10-26 before this bundle merged, and the derive
package now inherits the workspace version instead of staying at 0.10.0.
The accepted decision (one provider-owned native const narrowing factory
over the full binary64 domain) is unchanged.

## Context

Coeus's public const activation-parameter decoder accepts every binary64
encoding and narrows its single parameter to binary32. Rust can evaluate that
native conversion in const expressions, but `FloatElement::from_f64` cannot
be called there because stable Rust does not support const traits. Standard
`From`/`TryFrom` implementations do not provide this narrowing either.

The consumer cast guard requires conversion ownership to stay in Eunomia.
Removing const, restricting parameter encodings, introducing a consumer-local
rounding kernel or exempting the consumer path changes the contract or policy.

## Decision

Add `pub const fn F32::from_float(value: f64) -> F32` beside the native count
and signed-integer factories in the existing canonical conversion module.
Its only numeric operation is Rust's native narrowing cast. Both primitive
and wrapper `FloatElement::from_f64` implementations delegate to this factory,
so const and runtime callers share the production conversion kernel.

All binary64 encodings are admitted. Finite results round to nearest with
ties to even; overflow yields signed infinity, and underflow yields subnormals
or signed zero. Signed zero is preserved. NaNs remain NaN, with no stronger
promise about payload/sign across const and runtime than Rust provides.

## Version and consumers

This adds a public item, so repository versioning policy requires a minor
release: Eunomia 0.11.0. The unchanged derive package stays at 0.10.0 and its
internal requirement remains 0.10; no unrelated macro publication is needed.
There is no removed item or changed runtime conversion behavior in this slice.

Existing canonical Git consumers requiring `^0.10` cannot resolve a default
that declares 0.11. Before merging this automatic-publisher prerequisite,
prepare/review the coordinated provider requirement and public-boundary
updates. Coeus must consume one verified canonical source throughout its
graph; no path patch, dual identity or old-line compatibility release is used.

## Verification

The missing-factory regressions fail to compile on the original provider.
Analytical const cases cover both signs of zero, subnormal/normal transitions,
even/odd midpoint ties and finite overflow. An additional const grid covers
every exponent and sign at twelve significand boundaries (49,152 encodings),
compared against the previous runtime native conversion. Non-NaNs compare
bits; NaNs compare classification, matching the documented language contract.

Require the existing six-way feature matrix, native and NumPy tests, strict
lint/docs/doctests, MSRV, supply-chain, standard package and blocking API gates
at the final head. Publication and immutable registry receipts remain pending
until those gates and consumer coordination complete. No Miri or hardware
verification is claimed by this conversion change.
