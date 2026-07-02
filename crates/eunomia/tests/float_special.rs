//! Value-semantic contract tests for the `FloatElement` special functions
//! (`erf`, `erfc`, `lgamma`), cross-checked against analytic references rather
//! than asserting mere existence.
//!
//! Calls use fully-qualified `FloatElement::…` syntax: std is stabilizing
//! same-named inherent float methods (`unstable_name_collisions`), and method
//! syntax would silently rebind to those, changing which implementation the
//! test verifies. Qualification pins the trait under test.

use eunomia::FloatElement;

fn close(a: f64, b: f64, tol: f64, label: &str) {
    assert!((a - b).abs() <= tol, "{label}: {a} vs {b} (tol {tol})");
}

#[test]
fn erf_f64_matches_reference() {
    close(FloatElement::erf(0.0f64), 0.0, 1e-15, "erf(0)");
    close(
        FloatElement::erf(1.0f64),
        0.842_700_792_949_714_9,
        1e-12,
        "erf(1)",
    );
    // Error function is odd.
    close(
        FloatElement::erf(0.7f64),
        -FloatElement::erf(-0.7f64),
        1e-15,
        "erf odd",
    );
    // Saturates to 1 far from the origin.
    close(FloatElement::erf(6.0f64), 1.0, 1e-12, "erf(6)");
}

#[test]
fn erfc_f64_is_one_minus_erf() {
    close(FloatElement::erfc(0.0f64), 1.0, 1e-15, "erfc(0)");
    for &x in &[0.3f64, 1.0, 2.5] {
        close(
            FloatElement::erfc(x),
            1.0 - FloatElement::erf(x),
            1e-12,
            "erfc == 1 - erf",
        );
    }
    // Complementary tail is small, positive, and finite (no cancellation).
    let tail = FloatElement::erfc(5.0f64);
    assert!(tail > 0.0 && tail < 1e-10, "erfc(5) tail: {tail}");
}

#[test]
fn lgamma_f64_matches_reference() {
    close(
        FloatElement::lgamma(1.0f64),
        0.0,
        1e-12,
        "lgamma(1) = ln(0!) = 0",
    );
    close(
        FloatElement::lgamma(2.0f64),
        0.0,
        1e-12,
        "lgamma(2) = ln(1!) = 0",
    );
    close(
        FloatElement::lgamma(5.0f64),
        24.0f64.ln(),
        1e-12,
        "lgamma(5) = ln(4!) = ln 24",
    );
    // ln|Γ(1/2)| = ln(√π).
    close(
        FloatElement::lgamma(0.5f64),
        std::f64::consts::PI.sqrt().ln(),
        1e-12,
        "lgamma(1/2) = ln√π",
    );
}

#[test]
fn f32_special_functions_route_through_libm() {
    // f32 uses the trait default (single-precision libm), agreeing with the
    // analytic values within single precision.
    assert!(
        (FloatElement::erf(1.0f32) - 0.842_700_8).abs() < 1e-6,
        "f32 erf(1)"
    );
    assert!(
        (FloatElement::erfc(1.0f32) - (1.0 - 0.842_700_8)).abs() < 1e-6,
        "f32 erfc(1)"
    );
    assert!(
        (FloatElement::lgamma(5.0f32) - 24.0f32.ln()).abs() < 1e-4,
        "f32 lgamma(5)"
    );
}
