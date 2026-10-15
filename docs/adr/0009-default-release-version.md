# ADR 0009: Version the current default contract as 0.10

- Status: Accepted
- Date: 2026-10-15
- Item: EUNOMIA-DEFAULT-RELEASE-VERSION [major]

## Context

Before this release, the default branch declared 0.9.1 after the merged ADR 0007 retirement of
`CastFrom`, `CastTo` and the `NumericElement: CastFrom<i32>` supertrait.
An executed registry-baseline API comparison reports `trait_missing` and
`trait_removed_supertrait`. Those removals require a pre-1.0 minor boundary.

The already published 0.9.2 maintenance packages come from the distinct
[reviewed maintenance source](https://github.com/ryancinsight/eunomia/commit/06848422be64b878a5c205a976e4643d2a3e48a5),
which retains the 0.9 public conversion traits and supplies the Bessel APIs.
A consumer locked to that revision builds, but a fresh Git resolution of a
0.9.2 minimum fails while default still advertises 0.9.1. Changing only the
default version to 0.9.2 would conceal the merged breaking removal.

## Decision

Release the current default contract as 0.10.0, including the workspace-aligned
derive package and internal requirement. Preserve the merged native reduced
arithmetic, bulk conversion kernels and all tests. The only Rust edits
remove six redundant terminal ARM dispatch returns reported by strict Clippy;
those cfg-isolated blocks already end each function on ARM. Do not restore
the retired pair-generic conversion vocabulary or overwrite the maintenance
release. Keep draft PR #178 independent.

Consumers that follow default advance their requirement to `0.10`, migrate
former cast calls using ADR 0007's table, and retain exact reviewed revisions
in their locks. The compatible 0.9.2 packages remain immutable for existing
0.9 consumers; no duplicate publication or tag movement occurs.

## Verification and release

The release requires the existing six-way feature matrix, native and NumPy
tests, strict Clippy, docs/doctests, version-aware blocking API gate, standard
package verification and required exact-head CI. Existing trusted publishing
publishes the derive prerequisite before the dependent crate. Registry receipts
must identify the final merged source. Until those gates and receipts complete,
the backlog and checklist record the release as pending. Miri is deferred and
unverified; this increment adds no performance experiments.

## Publication receipts — 2026-10-15

[PR 179](https://github.com/ryancinsight/eunomia/pull/179) merges normally from
[reviewed head](https://github.com/ryancinsight/eunomia/commit/1e0fe9af0cb34e3e7c46241b7615e9778e1b8fbf)
into [published source](https://github.com/ryancinsight/eunomia/commit/a231e54dd3311bc916e78c91d994b00d651c1646).
The reviewed and merged trees are identical. All six actual PR checks pass;
no reviewer approval is required and no changes are requested. The blocking
[API and derive dry-run](https://github.com/ryancinsight/eunomia/actions/runs/38688096052)
passes directly against a buildable 0.9.2 baseline; its unavailable-baseline
probe is skipped. Both standard workspace packages verify at the reviewed head.

The existing [publisher](https://github.com/ryancinsight/eunomia/actions/runs/38688374647)
validates the complete two-package set and publishes derive before Eunomia.
Both registry entries are independently verified present and not yanked; their
trusted-publisher source and checksum-verified archive VCS metadata identify
that exact published merge.

| Package | Version | Registry SHA-256 |
| --- | --- | --- |
| eunomia-derive | 0.10.0 | `b635f0e2ce78dc34642386a1e969fb9c092c7c3a1eca90b6ced4887260a6d913` |
| eunomia | 0.10.0 | `7ea443187f337f60596a17130c6cd69ef4d6be6e73485af170cf929f6071d49c` |

Consumer planning precedes this provider release; final consumer requirements,
locks and acceptance follow its availability. No downstream-publication gate
exists in this provider's release pipeline. Compatible 0.9.2 remains immutable.
No deployment, credential/security change, review override, Miri run or held-task
work occurs. Miri remains deferred and explicitly unverified.
