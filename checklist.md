# Eunomia checklist

Target version: 0.6.0

Sprint phase: Execution

## E-034 [minor] — provider relative equality (Owner: Codex `/root`)

- [x] Take over the stale staged provider increment blocking Helios hosted
      compilation without disturbing any unrelated repository state.
- [x] Correct relative scaling to use both operands' absolute magnitudes and
      cover negative `f32`/`f64` values.
- [ ] Pass Eunomia format, feature, warning-denied Clippy, Nextest, doctest,
      Rustdoc, SemVer, and downstream Helios clean-provider gates.
- [ ] Publish Eunomia and advance the exact Atlas and Helios pins.

## E-025c [major] — retire the production raw-half surface (Owner: Codex)

- [x] Reconcile the live consumer graph: Hermes and Leto use native Eunomia
  reduced-precision types; Apollo's remaining raw-half FFT surface is
  Apollo-owned and does not consume Eunomia's foreign impls.
- [x] Delete the foreign raw-half numeric/cast surface and move `half` from the
  production graph to the differential-oracle dev graph.
- [x] Update provider tests, Rustdoc, README, changelog, and residual-risk
  records for the breaking 0.6.0 contract.
- [x] Pass format, feature, warning-denied Clippy, Nextest (86/86), doctest
  (5/5), rustdoc, semver, and path-overridden Hermes/Leto/Hephaestus checks.
  Hephaestus also proves its lock must advance from Hermes 0.3/Leto 0.38 to the
  merged native-provider Hermes 0.4/Leto 0.39 defaults.
- [ ] Publish and merge Eunomia, then refresh the Atlas gitlink and convergence
  audit.

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
