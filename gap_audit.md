# Eunomia gap audit

## Residual risk

- E-021 is in progress only at the downstream delivery boundary. Eunomia no
  longer directly depends on `num-traits`; all-feature builds still contain it
  transitively through the optional external NumPy stack.
- Hephaestus still exposes `num_complex::Complex<f32>` in production eigenvalue
  buffer APIs and copies those values into `numpy::Complex32` at the Python
  boundary.
- Leto production APIs already use `eunomia::Complex`; any remaining
  `num-complex` graph edge must be classified as a differential-test or Python
  interop dependency before removal.

## Evidence

- Compile-time assertions pin both complex layouts and the selected NumPy
  `Element` implementations.
- `cargo check` passes with no default features and all features.
- Warning-denied all-target/all-feature Clippy passes.
- Nextest passes 45/45; the provider-contract suite covers ABI round-trips,
  native/generic identities, and NumPy trait identity.
- Doctests and warning-denied rustdoc pass. `cargo semver-checks` reports no
  detected incompatibility against `origin/main`; the release remains 0.2.0
  because removing foreign trait implementations is a documented pre-1.0
  breaking contract.
