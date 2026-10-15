# ADR 0009: Version the current default contract as 0.10

- Status: Accepted
- Date: 2026-10-15
- Item: EUNOMIA-DEFAULT-RELEASE-VERSION [major]

## Context

The default branch declares 0.9.1 after the merged ADR 0007 retirement of
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
