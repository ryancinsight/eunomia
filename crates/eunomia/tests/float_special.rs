//! Value-semantic contract tests for the `FloatElement` special functions
//! (`erf`, `erfc`, `lgamma`), cross-checked against analytic references rather
//! than asserting mere existence.

use eunomia::FloatElement;

fn close(a: f64, b: f64, tol: f64, label: &str) {
    assert!((a - b).abs() <= tol, "{label}: {a} vs {b} (tol {tol})");
}

#[test]
fn erf_f64_matches_reference() {
    close(0.0f64.erf(), 0.0, 1e-15, "erf(0)");
    close(1.0f64.erf(), 0.842_700_792_949_714_9, 1e-12, "erf(1)");
    // Error function is odd.
    close(0.7f64.erf(), -(-0.7f64).erf(), 1e-15, "erf odd");
    // Saturates to 1 far from the origin.
    close(6.0f64.erf(), 1.0, 1e-12, "erf(6)");
}

#[test]
fn erfc_f64_is_one_minus_erf() {
    close(0.0f64.erfc(), 1.0, 1e-15, "erfc(0)");
    for &x in &[0.3f64, 1.0, 2.5] {
        close(x.erfc(), 1.0 - x.erf(), 1e-12, "erfc == 1 - erf");
    }
    // Complementary tail is small, positive, and finite (no cancellation).
    let tail = 5.0f64.erfc();
    assert!(tail > 0.0 && tail < 1e-10, "erfc(5) tail: {tail}");
}

#[test]
fn lgamma_f64_matches_reference() {
    close(1.0f64.lgamma(), 0.0, 1e-12, "lgamma(1) = ln(0!) = 0");
    close(2.0f64.lgamma(), 0.0, 1e-12, "lgamma(2) = ln(1!) = 0");
    close(5.0f64.lgamma(), 24.0f64.ln(), 1e-12, "lgamma(5) = ln(4!) = ln 24");
    // ln|Γ(1/2)| = ln(√π).
    close(0.5f64.lgamma(), std::f64::consts::PI.sqrt().ln(), 1e-12, "lgamma(1/2) = ln√π");
}

#[test]
fn f32_special_functions_route_through_libm() {
    // f32 uses the trait default (single-precision libm), agreeing with the
    // analytic values within single precision.
    assert!((1.0f32.erf() - 0.842_700_8).abs() < 1e-6, "f32 erf(1)");
    assert!((1.0f32.erfc() - (1.0 - 0.842_700_8)).abs() < 1e-6, "f32 erfc(1)");
    assert!((5.0f32.lgamma() - 24.0f32.ln()).abs() < 1e-4, "f32 lgamma(5)");
}
