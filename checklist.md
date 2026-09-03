# Eunomia checklist

Target version: 0.8.0

Sprint phase: Execution

## E-037 [patch] — Remove the external reduced-precision oracle (Owner: Codex)

- [x] Claim the provider-local test, manifest, ADR, and PM scope on the clean
      default head.
- [x] Replace the `half` dev oracle with an independent IEEE-754 reference
      module while preserving exhaustive and rounding-boundary coverage.
- [x] Remove the workspace and crate `half` declarations and synchronize the
      E-025c records; retain the documented Criterion/ciborium transitive edge.
- [x] Pass format, strict all-target/all-feature Clippy, Nextest, doctests,
      Rustdoc, and the standalone locked lockfile check.
- [ ] Push, merge, and refresh the Atlas provider pointer in a separate
      convergence increment.

## ATLAS-EUNOMIA-NAN-CONTRACT-2026-08-21 — current slice

- [x] Define one real-scalar min/max contract: one NaN is ignored, two NaNs
      remain NaN, `min(-0, +0)` is `-0`, and `max(-0, +0)` is `+0`.
- [x] Implement the contract in the `NumericElement` default and document the
      native primitive-float overrides; keep Complex's explicit ordering.
- [x] Add generic value-semantic tests for all shipped real scalar types and
      `RealField::clamp` NaN-bound cases.
- [x] Synchronize the numeric book and accept ADR 0005.
- [x] Run exact-head format, strict all-target/all-feature Clippy, Nextest
      **138/138**, doctests **9/9**, Rustdoc, and locked package listing at
      final code head `0cf3c7d`. The focused signed-zero regression also passes
      under Nextest.
- [x] Fresh staged-library `mdbook test` and `mdbook build` passed at the
      implementation head `ba51a16`; the later commits change only tests,
      checklist state, and the Unreleased changelog. The static `mdbook build`
      passes at `0cf3c7d`.
- [ ] Re-run local staged-library `mdbook test` against `0cf3c7d` after the
      shared target cache is uncontended and lane-owned artifacts are rebuilt;
      the current retry is invalidated by cached artifacts whose dep-info points
      at `D:/atlas/repos/eunomia` and by missing fresh `rkyv` compiler-artifact
      messages. Hosted fresh-build coverage remains pending.
- [ ] Push the provider branch, open the PR, and collect terminal hosted gates.
- [ ] Merge the provider change, then refresh the Atlas gitlink in a separate
      convergence increment.

## ATLAS-EUNOMIA-NUMPY-CI-2026-08-20 — current slice

- [x] Confirm the optional NumPy boundary is consumed by Hephaestus and
      Kwavers, while Eunomia has no standalone Python packaging surface.
- [x] Add the dedicated Python 3.13 / NumPy 2.5.1 CI job with pinned action
      SHAs and finite 20-minute execution budget.
- [x] Add a runtime dtype contract for `Complex32` (`complex64`, 8 bytes) and
      `Complex64` (`complex128`, 16 bytes); focused Nextest passes 4/4.
- [x] Install `nextest@0.9.140` in the NumPy job after hosted run
      `32412277378` failed because the command was not on `PATH`.
- [x] Confirm the feature compiles locally. The locked overlay gate and strict
      local Clippy remain blocked by the shared `[patch.unused]` lock state and
      GNU/MSVC shared-cache mismatch respectively; no such claim is made.
- [ ] Collect hosted CI at the exact branch head, then synchronize the Atlas
      and provider completion records.

## E-036 [patch] — strict Clippy Rustdoc closure — closed 2026-08-16

- Owner: Codex; claimed 2026-08-16 from the clean provider default `58e5715`.
- Scope: `crates/eunomia/src/types/complex/numpy_element.rs` Rustdoc and this
  provider checklist/gap audit only. No numeric or public API behavior changes.
- Acceptance: standalone format, locked workspace check, warning-denied
  all-target/all-feature Clippy, Nextest, doctests, Rustdoc, and cargo-deny
  pass at the final provider head; the corrected terminology is documented.
- [x] Correct the Clippy `doc_markdown` failure and synchronize the audit
      evidence. The exact default head is now verified by standalone format,
      locked all-target/all-feature check, strict Clippy, Nextest 135/135,
      doctest 9/9, Rustdoc, and cargo-deny advisories/bans/licenses/sources.

## FloatElement sign-preserving roots (cbrt/rsqrt/nth_root) — done

- [x] Add sign-preserving `cbrt` to `FloatElement` — PR #60 (`bba10b6`).
- [x] Add sign-preserving `rsqrt` and `nth_root` to `FloatElement` — PR #63,
      merged `1a52590`.
- [x] Verify the consumer migrations resolve against `1a52590`: CFDrs PR #341
      (`e30704b2`), kwavers PR #364 (`1cb63974`), ritk PR #139 (`ec7e2e4c`).

## E-REL-001 [patch] — crates.io Trusted Publishing

- [x] Add and validate the release workflow, then register `eunomia` against
      `ryancinsight/eunomia/.github/workflows/rust-release.yml` in crates.io.
- [x] Confirm Eunomia `0.8.0` is indexed on crates.io (2026-08-09) and validate
      the clean-provider package gates: metadata, formatting, all six feature
      checks, warning-denied Clippy, 116/116 Nextest, 9/9 doctests, Rustdoc,
      and `cargo package --locked --list`.
- [x] Re-verify 2026-08-12 at the provider worktree head: no-default-feature
      check, warning-denied all-target/all-feature Clippy, Nextest 117/117
      (all features), 9/9 doctests, and strict all-target check pass.
- [x] Run an online `cargo publish --locked --package eunomia --dry-run` from
      exact default head `d252f968`; Cargo packages 73 files, verifies the crate,
      and stops at the expected dry-run upload boundary.
- [x] Add a hosted Rust 1.95.0 all-target/all-feature MSRV gate using the
      committed lockfile and pinned action SHAs; run `31789001841` passes at
      PR head `b6c3d9a`.


## E-034 [minor] — provider relative equality (Owner: Codex `/root`)

- [x] Take over the stale staged provider increment blocking Helios hosted
      compilation without disturbing any unrelated repository state.
- [x] Correct relative scaling to use both operands' absolute magnitudes and
      cover negative `f32`/`f64` values.
- [x] Pass Eunomia format, no-default-feature check, warning-denied Clippy,
      Nextest (94/94), doctest (9/9), warning-denied Rustdoc, and SemVer
      (196/196), plus clean-provider Helios compilation.
- [x] Publish Eunomia at `884d193`, advance Atlas to `a5279bf`, and pass hosted
      Helios run `29882508040` at the pinned consumer head `22bea48`.

## E-025c [major] — retire the production raw-half surface (Owner: Codex)

- [x] Reconcile the live consumer graph: Hermes and Leto use native Eunomia
  reduced-precision types; Apollo's remaining raw-half FFT surface is
  Apollo-owned and does not consume Eunomia's foreign impls.
- [x] Delete the foreign raw-half numeric/cast surface and remove the external
      reduced-precision dependency from the provider graph.
- [x] Update provider tests, Rustdoc, README, changelog, and residual-risk
  records for the breaking 0.6.0 contract.
- [x] Pass format, feature, warning-denied Clippy, Nextest (86/86), doctest
  (5/5), rustdoc, semver, and path-overridden Hermes/Leto/Hephaestus checks.
  Hephaestus also proves its lock must advance from Hermes 0.3/Leto 0.38 to the
  merged native-provider Hermes 0.4/Leto 0.39 defaults.
- [x] Publish Eunomia `0.8.0`; crates.io indexing is confirmed at
      https://crates.io/crates/eunomia and the provider release workflow is
      verified. The exact online dry-run remains tracked under E-REL-001.
- [x] Merge the 0.8.0 provider revision into the remote default and refresh the
      Atlas gitlink/convergence audit; current default is `d252f968` and the
      parent pointer already records that exact head.


## E-021 [arch] — native complex provider cutover

- [x] Audit direct and transitive `num-traits`/`num-complex` ownership in
  Eunomia, Leto, and Hephaestus.
- [x] Record the breaking contract and migration in ADR 0002.
- [x] Remove Eunomia's direct `num-traits` dependency and foreign identity
  implementations.
- [x] Add compile-time complex ABI assertions and value-semantic provider tests.
- [x] Align the optional NumPy/PyO3 boundary with version 0.29.
- [x] Pass format, warning-denied Clippy, Nextest, doctest, rustdoc, feature,
  and semver gates.
- [x] Publish Eunomia first, then update and verify Leto and Hephaestus:
  Eunomia PR #36 (`34d0cc8`), Hephaestus PR #48 (`82bb3a7`), and Leto PR
  #42 (`cf47686`) are merged into their remote defaults.

## E-022 [minor] — native binary16/bfloat16 conversion kernel (Owner: Claude)

Next increment after E-022: E-023 (fold sub-byte formats onto the kernel + pin
conventions). See ADR 0003 / gap_audit §Byte-layout for the full workstream
(E-022…E-030).

- [x] Audit eunomia vs bytemuck/zerocopy/half/`TransmuteFrom`; record ADR 0003 +
  gap_audit findings + backlog E-022…E-030.
- [x] Add `convert::{narrow, widen}` generic const-parameterized IEEE kernel
  (RNE ties-to-even, subnormals, inf/NaN, f32-subnormal).
- [x] Differential-verify bit-exact vs `half`: exhaustive widen (2¹⁶ × 2),
  exhaustive finite round-trip, ~4.2M rounding sweep, pinned ties-to-even.
- [x] Pass fmt, clippy `-D warnings`, nextest (52/52), doctest, rustdoc.
- [x] Commit and merge PR #37 (`6f431f2d`) into `main`.

## E-023 [minor] — canonical sub-byte conversion (Owner: Codex)

- [x] Pin the existing E5M2, E2M1, E4M3, and E3M0 conventions with analytical
  known-value, special-value, round-trip, and ties-to-even tests.
- [x] Generalize the native conversion kernel over the finite-only
  reserved-top-exponent policy without duplicating the arithmetic.
- [x] Replace all four hand-written type conversions and packed conversion
  tables with the canonical kernel.
- [x] Correct sub-byte numeric constants to match their declared layouts.
- [x] Pass format, feature, warning-denied Clippy, Nextest, doctest, rustdoc,
  semver, and downstream Leto/Hephaestus checks.

## ADR governance — generated index refresh

- [x] Confirm ADR 0001–0004 already carry canonical `Status: Accepted`
  headers; no decision content changes are in scope.
- [x] Regenerate `docs/adr/README.md` from the provider ADR headers and verify
  exact generator equality in the Atlas overlay.
- [x] Synchronize the provider backlog and gap audit with the derived-index
  refresh; retain the root Atlas ADR-governance item as the cross-repository
  status owner.
