# ADR 0012: Eunomia release scope for 1.0

Status: Accepted  \
Date: 2026-11-10  \
Change class: [arch] (pre-1.0 ladder; no numeric kernel, representation, or public-surface change)  \
Delivery: backlog/milestones.md

## Context

Eunomia (workspace 0.12.0) provides the stack's scalar datatype law: native
complex, reduced-precision conversion kernels, the `RealField`/`ComplexField`
scalar field surface, const native float construction, and the unit-scalar
provider seam. The backlog carries open provider work (the blocked 0.12 cohort
bundle, the in-review digamma provider, the in-progress layout-derive
migration) and consumer-owned migrations chained through apollo, and no
milestone ladder exists to 1.0.

## Decision

Adopt a five-milestone ladder to 1.0 recorded in `backlog/milestones.md`:

- **M-special-functions** (0.13.0): the special-function surface (digamma
  provider) lands with hosted gates.
- **M-layout-derives** (0.13.0): the provider-owned `Pod`/`Zeroable` derives
  and the bytemuck-optional gating land with consumer GPU-buffer verification.
- **M-0.12-cohort** (0.12.0): the blocked NumPy 0.30 bundle publishes with
  receipts.
- **M-1.0** (1.0.0): stable scalar-law surface (authority-gated) with
  documented contracts and release gates satisfied.
- **M-ecosystem** (needs M-1.0): consumer-owned migrations (apollo-first chain,
  ritk, CFDrs, gaia remainder) and on-demand OCP-MXFP formats proceed against
  the stable surface; they do not gate 1.0.

## Alternatives

- Leave progress implicit in backlog items: rejected because stack policy
  requires a runnable capability ladder, and every ladder above Eunomia cites
  scalar law.
- Target 1.0 in one step: rejected because the blocked cohort bundle and the
  in-flight provider migrations are unresolved.
- Gate 1.0 on the apollo-first consumer chain: rejected because consumer-owned
  migrations follow the stable surface; ordering is M-ecosystem needs M-1.0,
  not the reverse.

## Consequences

- Open items carry `milestone:` assignments; milestones close only when exit
  checks pass on fetched default commits; 1.0 release/publication remains
  authority-gated.
- Proposed ADR 0007 (retire CastFrom/CastTo) must resolve to Accepted or
  Declined before M-1.0 closes; the ladder does not prejudge it.
- E-024 (OCP-MXFP formats) stays on-demand: it enters M-ecosystem only when a
  driving quantization consumer appears.
- This ADR is rewritten in place as scope evidence changes.
