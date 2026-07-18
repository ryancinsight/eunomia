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
