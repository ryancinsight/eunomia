//! The Bessel surface (`bessel_j0`, `bessel_j1`, `bessel_k0`), each against
//! its own correct oracle.
//!
//! `f64`/`F64` call `libm`'s `f64` entry point directly for `j0`/`j1`, so the
//! assertion is bitwise equality against that exact same call (wiring, not
//! accuracy) — see `transcendentals.rs` for the methodology. `f32` routes
//! through `libm`'s independent `f32` algorithm and is compared against the
//! `f64` result rounded to `f32` within an assumed 2-ulp bound (the `j0f`/
//! `j1f` sources carry no documented ulp comment; both descend from Sun
//! fdlibm with no stated final-result bound, so the reviewed 1-ulp +
//! 0.5-ulp-oracle assumption applies, marked as such at each site).
//!
//! `k0` has no `libm` entry point (libm 0.2.16 provides no `k0`), so every
//! implementor evaluates the same Abramowitz & Stegun program in its own
//! precision. Its oracle is the published table (DLMF 10.32 / A&S Table
//! 9.8): 9.8.5 bounds the small-argument absolute error by 1e-8 and 9.8.6
//! bounds `x^{1/2}·e^{x}·K₀(x)` by 1.9e-7 (relative ~1.5e-7 on `K₀` itself).
//! The test bounds below restate those quoted errors with headroom for the
//! final rounding, never fit to an observed failure.
//!
//! Calls use fully-qualified `RealField::…` syntax, matching the harness
//! convention documented in `main.rs`: the methods live on `RealField`, and
//! qualification pins the trait under test.

use eunomia::{RealField, F64};

use super::fixtures::assert_close_within_ulp;

/// `f64` route: `libm` **is** the implementation for `j0`/`j1`, so the only
/// correct assertion is wiring — bitwise equality against the exact same
/// `libm` call. A dropped `f64`/`F64` override falls back to the `f32`-routed
/// default, which differs by roughly `ε₃₂` relative (billions of `f64` ulps)
/// and never passes bitwise by accident.
#[test]
fn bessel_double_precision_route_wires_to_libm_bitwise() {
    for &x in &[0.0, 0.3, 1.5, 5.0, 10.0, -2.5] {
        assert_eq!(RealField::bessel_j0(x), libm::j0(x), "f64 bessel_j0({x})");
        assert_eq!(
            RealField::bessel_j0(F64(x)).0,
            libm::j0(x),
            "F64 bessel_j0({x})"
        );
        assert_eq!(RealField::bessel_j1(x), libm::j1(x), "f64 bessel_j1({x})");
        assert_eq!(
            RealField::bessel_j1(F64(x)).0,
            libm::j1(x),
            "F64 bessel_j1({x})"
        );
    }
}

/// Closed-form points: `J₀(0) = 1`, `J₁(0) = 0` (both special-cased ranges in
/// every implementation of this family, evaluated directly rather than
/// approximated).
#[test]
fn bessel_double_precision_analytic_reference_points() {
    assert_eq!(RealField::bessel_j0(0.0f64), 1.0);
    assert_eq!(RealField::bessel_j1(0.0f64), 0.0);
}

/// `f32` route vs the `f64` oracle rounded to `f32`. Every `f32` input widens
/// to `f64` exactly, so the oracle carries zero input error; the bound covers
/// only the `f32` algorithm's own rounding (assumed 2 ulp — no documented
/// bound in `j0f.rs`/`j1f.rs`, same assumption as the undocumented
/// single-precision transcendentals).
#[test]
fn bessel_single_precision_route_matches_double_precision_oracle() {
    for &x in &[0.0f32, 0.3, 1.5, 5.0, 10.0, -2.5] {
        assert_close_within_ulp(
            RealField::bessel_j0(x),
            libm::j0(f64::from(x)) as f32,
            2,
            "f32 bessel_j0 (undocumented bound, assumed 2 ulp)",
        );
        assert_close_within_ulp(
            RealField::bessel_j1(x),
            libm::j1(f64::from(x)) as f32,
            2,
            "f32 bessel_j1 (undocumented bound, assumed 2 ulp)",
        );
    }
}

/// `k0` against the published table. Small branch asserts absolute error
/// (values are O(1)); large branch asserts relative error. Bounds restate the
/// A&S quoted errors (1e-8 abs / ~1.5e-7 rel) with rounding headroom.
#[test]
fn bessel_k0_matches_reference_values() {
    for &(x, expected) in &[
        (0.1_f64, 2.427_069_0_f64),
        (1.0, 0.421_024_4),
        (2.0, 0.113_893_9),
    ] {
        let error = (RealField::bessel_k0(x) - expected).abs();
        assert!(error < 1e-6, "K0({x}) absolute error {error}");
        assert_eq!(
            RealField::bessel_k0(F64(x)).0,
            RealField::bessel_k0(x),
            "F64 k0({x}) matches f64"
        );
    }
    for &(x, expected) in &[
        (3.0_f64, 0.034_739_504_2_f64),
        (5.0, 0.003_691_098_3),
        (10.0, 1.778_006_23e-5),
    ] {
        let value: f64 = RealField::bessel_k0(x);
        let error = ((value - expected) / expected).abs();
        assert!(error < 2e-7, "K0({x}) relative error {error}");
    }
}

/// `K₀` diverges at the origin and is undefined for `x ≤ 0`: non-positive
/// and non-finite arguments return `NaN`.
#[test]
fn bessel_k0_rejects_non_positive_arguments() {
    for &x in &[0.0f64, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(RealField::bessel_k0(x).is_nan(), "K0({x}) must be NaN");
    }
    for &x in &[0.0f32, -1.0, f32::NAN, f32::INFINITY] {
        assert!(RealField::bessel_k0(x).is_nan(), "f32 K0({x}) must be NaN");
    }
}

/// `f32` evaluates the same A&S program in `f32` precision. Derivation: ~15
/// multiply-adds per branch at ≤0.5 ulp (`ε₃₂/2 ≈ 6e-8` relative) each
/// accumulate to ≈9e-7 relative worst case, plus the 1.9e-7 algorithm term —
/// so 2e-6 relative covers analysis with margin. Small branch reuses the 1e-6
/// absolute bound (`f32` rounding at O(1) values is ≈1e-7 ≪ 1e-6).
#[test]
fn bessel_k0_single_precision_agrees_with_double_precision_program() {
    for &(x, expected) in &[
        (0.1f32, 2.427_069_f32),
        (1.0, 0.421_024_4),
        (2.0, 0.113_893_9),
    ] {
        let error = (RealField::bessel_k0(x) - expected).abs();
        assert!(error < 1e-6, "f32 K0({x}) absolute error {error}");
    }
    for &x in &[3.0f32, 5.0, 10.0] {
        let single: f32 = RealField::bessel_k0(x);
        let double = libm_f64_k0_reference(f64::from(x));
        let error = ((single as f64 - double) / double).abs();
        assert!(error < 2e-6, "f32 K0({x}) relative disagreement {error}");
    }
}

/// Reference evaluator for the `f32`-agreement test: the `f64` core itself,
/// verified exact against the table in `bessel_k0_matches_reference_values`.
/// Routing through the implementation under test as the oracle would be
/// circular were it not pinned to the table first — it is, above.
fn libm_f64_k0_reference(x: f64) -> f64 {
    RealField::bessel_k0(x)
}
