# Eunomia checklist

Target version: 0.5.0

Sprint phase: Execution

## E-025b [minor] — native reduced-precision bit-pattern contract (Owner: Codex)

- [x] Audit Hermes' raw-half boundary and identify the missing provider-owned
  representation contract.
- [ ] Add const `F16`/`Bf16` bit-pattern constructors and accessors with
  value-semantic signed-zero and NaN-payload tests.
- [ ] Pass format, feature, warning-denied Clippy, Nextest, doctest, rustdoc,
  semver, and path-patched Hermes checks.
- [ ] Publish Eunomia first, then migrate the bounded Hermes scalar/intrinsic
  scope and continue dependency-ordered into Leto.

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
