# ADR 0008: Compatible Bessel maintenance release

- Status: Accepted
- Date: 2026-10-15

## Context

Published Eunomia 0.9.1 is commit 47e571489cc2c8dd00ba162864a9ef4bb81adefc.
Hephaestus now uses the additive `RealField` Bessel methods merged by PR #173
at 9f1c2ecbafcbd25353e425980a04968332597541. That provider increment has
not reached crates.io. Later main removes the public `CastFrom`/`CastTo`
traits; a direct API comparison against 0.9.1 rejects that removal.

## Decision

Release the existing Bessel increment as 0.9.2 on a 0.9 maintenance branch,
retaining every published 0.9.1 API. The maintenance branch starts at the
published commit and contains only the already merged Bessel increment,
release metadata, verification records, and removal of six terminal ARM
returns diagnosed by strict lint. Intrinsic calls and fallback bodies stay unchanged. It does not merge backwards
into main or alter main's later implementation history.

The Bessel formulas, precision-specific implementations, references, and
boundary tests stay byte-identical to the merged provider increment.
Verify the complete ordinary feature matrix, the registry API comparison,
and packaged crates at the exact release head before uploading. Retain
existing coverage and timeouts. Miri remains deferred and unverified.

## Consequences

Consumers requiring the published 0.9 vocabulary can resolve Bessel methods
without a breaking scalar identity migration. Main's cast-trait retirement
still requires its own compatible version boundary before publication.
The maintenance branch accepts only compatible corrections; it is not a
compatibility wrapper around main's removed traits.
