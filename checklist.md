# Eunomia checklist

Target version: 0.2.0

Sprint phase: Execution

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
- [ ] Publish Eunomia first, then update and verify Leto and Hephaestus.

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
- [ ] Commit + PR (awaiting user approval — working-tree change, not committed).
