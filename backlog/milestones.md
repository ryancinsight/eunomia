# Milestone ladder ([ADR 0012](../docs/adr/0012-release-scope-1-0.md))
<a id="M-special-functions"></a> **M-special-functions** (0.13.0) needs: (none) — special-function provider surface.
- exit: land the digamma provider (`EUNOMIA-DIGAMMA`, PR #185) with final-head review, hosted gates, and publication receipts; no open correctness item in the surface.
- exit: keep every `float_special` contract leaf at or under the 500-line target (done: split in `4b846d9`, largest leaf 455 lines).
<a id="M-layout-derives"></a> **M-layout-derives** (0.13.0) needs: (none) — provider-owned byte-layout vocabulary and derives.
- exit: land the `Pod`/`Zeroable` derive migration (`E-027`, branch `feat/eunomia-layout-derives`) with hephaestus GPU-buffer verification of the marker bounds.
- exit: complete the deferred E-026 gating (bytemuck optional, default-on) only with `default-features = false` consumer verification; fmt/clippy/nextest/doc gates pass.
<a id="M-0.12-cohort"></a> **M-0.12-cohort** (0.12.0) needs: (none) — publish the NumPy 0.30 bundle (`EUNOMIA-NUMPY-030-UNRELEASED-BUNDLE`) in bridge-first order with receipts; no open correctness item in the interop boundary.
<a id="M-1.0"></a> **M-1.0** (1.0.0) needs: M-special-functions, M-layout-derives, M-0.12-cohort — stable scalar-law release (authority-gated): fmt/clippy/nextest/doctest/doc gates, SemVer and publish dry-run, current book/rustdoc, ADR 0007 resolved, no open correctness item.
<a id="M-ecosystem"></a> **M-ecosystem** needs: M-1.0 — consumer-owned migrations against the stable surface (do not gate 1.0).
- exit: apollo-first num-complex migration (`E-011`), then ritk (`E-012`), CFDrs (`E-012b`), the CFDrs/gaia num-traits remainder (`E-013`), and on-demand OCP-MXFP formats (`E-024`) with OCP vectors matching.
