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

## Publication receipt

Both crates are present and not yanked in the registry. Their trusted
publisher metadata identifies source
`06848422be64b878a5c205a976e4643d2a3e48a5`:

| Crate | Registry checksum | Publisher run |
| --- | --- | --- |
| eunomia-derive 0.9.2 | `491320a71f4148ba6fe393f3799430cb2e0db02c7b27154b62b04288de1cb975` | [38668990691](https://github.com/ryancinsight/eunomia/actions/runs/38668990691) |
| eunomia 0.9.2 | `b36c9a5eeec86bb06e611eec9b8bf4c781507dc7fa8d5b6e0b8c3c10f35fc82e` | [38670543920](https://github.com/ryancinsight/eunomia/actions/runs/38670543920) |

Exact-source ordinary native/features/NumPy checks, strict lint, docs,
196 API checks and standard package verification pass. The registry
receipts close the publication item; PR #177's subsequent completion-record
changes still require their own documentation-head checks before integration.
Miri remains deferred and unverified.
