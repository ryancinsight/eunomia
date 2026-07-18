# Eunomia gap audit

## Residual risk

- E-021 is in progress. Eunomia still directly depends on `num-traits` for
  foreign `Zero`/`One` implementations.
- Hephaestus still exposes `num_complex::Complex<f32>` in production eigenvalue
  buffer APIs and copies those values into `numpy::Complex32` at the Python
  boundary.
- Leto production APIs already use `eunomia::Complex`; any remaining
  `num-complex` graph edge must be classified as a differential-test or Python
  interop dependency before removal.

## Evidence

- Source scan and `cargo tree -i` on 2026-07-18 establish the dependency and
  public-API ownership above. This is source and dependency-graph evidence, not
  yet compile or runtime verification.
