# ADR 0011: Bundle optional NumPy 0.30 interop into the 0.12 cohort

Status: Accepted  \
Date: 2026-10-19  \
Change class: [major] (pre-1.0 minor boundary)  \
Delivery: EUNOMIA-NUMPY-030-UNRELEASED-BUNDLE
Revision 2026-10-29: the bundle ships in the 0.12.0 cohort because Eunomia
0.11.0 was published on 2026-10-26 before this merge; the accepted decision
(NumPy/PyO3 0.30 optional boundary, bridge-first consumer order) is
unchanged, and the derive package inherits the workspace version.

## Context

The user-approved provider plan (2026-10-19) targets Eunomia 0.11, Aequitas 0.4,
binding 0.2 and a common NumPy/PyO3 0.30 boundary, in bridge-first order. PR #182 already prepares
facade 0.11 for the const float factory. Its accepted ADR 0010 covers that
addition; its additive compatibility result does not cover dependency identity.

Eunomia's optional `numpy` feature implements `numpy::Element` for its canonical
complex types. NumPy 0.30's trait is distinct from NumPy 0.29's trait even when
their element methods are equivalent. PyO3's native `links` constraint rejects
both families in one Cargo graph. This is a breaking optional-feature migration;
passing public-item API checks alone does not establish consumer compatibility.

## Decision

Include the two optional requirement updates and their ordinarily resolved lock
delta in the still-unreleased 0.11 bundle. Preserve every factory source/test,
the accepted ADR 0010, facade 0.11 and unchanged derive 0.10. No numeric kernel,
representation, trait implementation body or feature selection changes.

Do not publish factory-only 0.11 first: this subsequent breaking optional-feature
change would then need 0.12 under normal pre-1.0 policy, beyond the Aequitas #140
bridge's current <0.12 ceiling. The approved version policy is resolved. The factory owner accepts this
decision as part of PR #182; the bridge owner retains the integration order.

The optional Python boundary requires Python 3.9 rather than 3.8. Core builds
retain Rust MSRV 1.95 and acquire no Python dependency. NumPy 2 runtime support is
verified by the existing dtype/value contract tests, not inferred from the Rust
dependency version. No free-threaded or all-platform guarantee follows here.

## Migration and invariants

Python extension consumers select Eunomia 0.11 with NumPy 0.30 and PyO3 0.30
together, regenerate their locks normally and rebuild their extensions. The
canonical Eunomia complex types and their byte/dtype contracts remain the only
provider; no duplicate types, compatibility aliases, link override, path patch,
manual lock SourceId or ABI-forward-compatibility switch is a release solution.

The saved [PR #182 factory assessment](https://github.com/ryancinsight/eunomia/pull/182)
already executes provider NumPy/all-
feature tests, real old/new consumer compatibility probes and combined Aequitas
tests. Those are revision-scoped local evidence, not final canonical acceptance.
The unchanged accepted factory regressions retain full binary64-domain coverage.

## Verification and release order

The factory owner reviews the complete bundle and its source-preservation
proof before integration. At the resulting exact owner head, require the existing
six-way feature matrix, native and NumPy suites, strict lint, docs/doctests,
Rust MSRV 1.95, no-std, supply-chain, standard packages and blocking API/release
validation. Keep explicit old/new NumPy consumer tests: signature checks alone
miss the dependency-identity break. Renew independent review and actual GitHub
required checks/approvals; a local review does not replace a server approval.

Coordinate Aequitas #140/141 and downstream public scalar/quantity boundaries
before the automatic publisher runs. Verify immutable 0.11 archive identity and
checksum after authorized publication. Owner integration changes the coordinated source bundle only. It changes no
workflow, baseline, gate, credential, registry or deployment.
Coeus Miri remains deferred and unverified.
