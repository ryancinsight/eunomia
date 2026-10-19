# ADR 0011: Bundle optional NumPy 0.30 interop into unreleased 0.11

Status: Proposed  \
Date: 2026-10-19  \
Change class: [major] (pre-1.0 minor boundary)  \
Delivery: EUNOMIA-NUMPY-030-UNRELEASED-BUNDLE

## Context

The approved provider plan targets Eunomia0.11, Aequitas0.4/binding0.2 and a
common NumPy/PyO30.30 boundary, in bridge-first order. PR182 already prepares
facade0.11 for the const float factory. Its accepted ADR0010 covers that
addition; its additive compatibility result does not cover dependency identity.

Eunomia's optional `numpy` feature implements `numpy::Element` for its canonical
complex types. NumPy0.30's trait is distinct from NumPy0.29's trait even when
their element methods are equivalent. PyO3's native `links` constraint rejects
both families in one Cargo graph. This is a breaking optional-feature migration;
passing public-item API checks alone does not establish consumer compatibility.

## Proposed integration

Include the two optional requirement updates and their ordinarily resolved lock
delta in the still-unreleased0.11 bundle. Preserve every factory source/test,
the accepted ADR0010, facade0.11 and unchanged derive0.10. No numeric kernel,
representation, trait implementation body or feature selection changes.

Do not publish factory-only0.11 first: this subsequent breaking optional-feature
change would then need0.12 under normal pre-1.0 policy, beyond the Aequitas140
bridge's current <0.12 ceiling. The approved version policy is resolved; the
existing factory and bridge owners review the combined integration and ordering.

The optional Python boundary requires Python3.9 rather than3.8. Core builds
retain RustMSRV1.95 and acquire no Python dependency. NumPy2 runtime support is
verified by the existing dtype/value contract tests, not inferred from the Rust
dependency version. No free-threaded or all-platform guarantee follows here.

## Migration and invariants

Python extension consumers select Eunomia0.11 with NumPy0.30 and PyO30.30
together, regenerate their locks normally and rebuild their extensions. The
canonical Eunomia complex types and their byte/dtype contracts remain the only
provider; no duplicate types, compatibility aliases, link override, path patch,
manual lock SourceId or ABI-forward-compatibility switch is a release solution.

The assessment at factory head `bb769965` already executes provider NumPy/all-
feature tests, real old/new consumer compatibility probes and combined Aequitas
tests. Those are revision-scoped local evidence, not final canonical acceptance.
The unchanged accepted factory regressions retain full binary64-domain coverage.

## Verification and release order

Before normal owner integration, inspect the complete bundle and its source
preservation proof. At the resulting exact owner head, require the existing
six-way feature matrix, native and NumPy suites, strict lint, docs/doctests,
RustMSRV1.95, no-std, supply-chain, standard packages and blocking API/release
validation. Keep explicit old/new NumPy consumer tests: signature checks alone
miss the dependency-identity break. Renew independent review and actual GitHub
required checks/approvals; a local review does not replace a server approval.

Coordinate Aequitas140/141 and downstream public scalar/quantity boundaries
before the automatic publisher runs. Verify immutable0.11 archive identity and
checksum after authorized publication. This proposed local bundle changes no
owner branch, workflow, baseline, gate, credential, registry or deployment.
Coeus Miri remains deferred and unverified.
