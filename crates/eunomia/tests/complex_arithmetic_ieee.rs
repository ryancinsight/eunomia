//! IEEE-754 robustness of the complex arithmetic operators.
//!
//! The complex quotient and magnitude are the two places where the textbook
//! closed forms square a component and can therefore overflow (or flush to
//! zero) *before* the result does. Each test below pins a concrete case whose
//! exact result is representable, alongside the ordinary-operand behaviour the
//! optimisation must not change.

use eunomia::{Complex, NumericElement};

/// `|a - b| <= rel * |b|`, with an absolute floor for an exact zero expected.
fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs().max(f64::MIN_POSITIVE)
}

fn close32(a: f32, b: f32, rel: f32) -> bool {
    (a - b).abs() <= rel * b.abs().max(f32::MIN_POSITIVE)
}

#[test]
fn division_avoids_denominator_overflow() {
    // (1 + i) / (1e200 + 1e200 i) = 1e-200 exactly (the denominator is
    // 1e200·(1 + i)). The naive `c² + d²` denominator computes 1e400 = ∞ and
    // returns 0 + 0i.
    let quotient = Complex::new(1.0_f64, 1.0) / Complex::new(1.0e200, 1.0e200);
    assert!(close(quotient.re, 1.0e-200, 1e-12), "re = {}", quotient.re);
    assert_eq!(quotient.im, 0.0, "im = {}", quotient.im);

    // f32 counterpart: 1e20² = 1e40 overflows f32 (max ≈ 3.4e38).
    let quotient32 = Complex::new(1.0_f32, 1.0) / Complex::new(1.0e20, 1.0e20);
    assert!(
        close32(quotient32.re, 1.0e-20, 1e-6),
        "re = {}",
        quotient32.re
    );
    assert_eq!(quotient32.im, 0.0, "im = {}", quotient32.im);
}

#[test]
fn division_avoids_denominator_underflow() {
    // (1 + i) / (1e-200 + 1e-200 i) = 1e200 exactly. The naive denominator
    // computes 1e-400 = 0 and returns ∞ + NaNi.
    let quotient = Complex::new(1.0_f64, 1.0) / Complex::new(1.0e-200, 1.0e-200);
    assert!(close(quotient.re, 1.0e200, 1e-12), "re = {}", quotient.re);
    assert_eq!(quotient.im, 0.0, "im = {}", quotient.im);
    assert!(quotient.re.is_finite());

    let quotient32 = Complex::new(1.0_f32, 1.0) / Complex::new(1.0e-20, 1.0e-20);
    assert!(
        close32(quotient32.re, 1.0e20, 1e-6),
        "re = {}",
        quotient32.re
    );
    assert_eq!(quotient32.im, 0.0, "im = {}", quotient32.im);
    assert!(quotient32.re.is_finite());
}

#[test]
fn division_matches_the_closed_form_for_ordinary_operands() {
    // (1 + 2i) / (3 - i) = (1 + 7i) / 10.
    let quotient = Complex::new(1.0_f64, 2.0) / Complex::new(3.0, -1.0);
    assert!(close(quotient.re, 0.1, 1e-12), "re = {}", quotient.re);
    assert!(close(quotient.im, 0.7, 1e-12), "im = {}", quotient.im);

    // Divide by i then multiply by i round-trips (exercises both branches'
    // ordinary-operand path).
    let z = Complex::new(2.0_f64, 3.0);
    let i = Complex::new(0.0_f64, 1.0);
    let round_trip = z * i / i;
    assert!(close(round_trip.re, z.re, 1e-12));
    assert!(close(round_trip.im, z.im, 1e-12));
}

#[test]
fn magnitude_does_not_overflow() {
    // |1e200 + 1e200 i| = √2·1e200; the naive re² + im² computes 1e400 = ∞.
    let magnitude = Complex::new(1.0e200_f64, 1.0e200).norm();
    assert!(
        close(magnitude, core::f64::consts::SQRT_2 * 1.0e200, 1e-12),
        "norm = {magnitude}"
    );
    assert!(magnitude.is_finite());

    // The `NumericElement::abs` path must agree with `Complex::norm`.
    let abs = <Complex<f64> as NumericElement>::abs(Complex::new(1.0e200, 1.0e200)).re;
    assert!(close(abs, core::f64::consts::SQRT_2 * 1.0e200, 1e-12), "abs = {abs}");

    // f32 counterpart: 1e20² = 1e40 overflows f32.
    let magnitude32 = Complex::new(1.0e20_f32, 1.0e20).norm();
    assert!(
        close32(magnitude32, core::f32::consts::SQRT_2 * 1.0e20, 1e-6),
        "norm = {magnitude32}"
    );
}

#[test]
fn magnitude_preserves_the_ordinary_and_zero_cases() {
    // Exact ordinary magnitude is unchanged.
    assert_eq!(Complex::new(3.0_f64, 4.0).norm(), 5.0);
    assert_eq!(Complex::new(0.0_f64, 0.0).norm(), 0.0);
    assert_eq!(Complex::new(-0.0_f64, 0.0).norm(), 0.0);
    // A single-component magnitude is the component itself.
    assert_eq!(Complex::new(7.0_f64, 0.0).norm(), 7.0);
    assert_eq!(Complex::new(0.0_f64, -9.0).norm(), 9.0);
    // NaN stays NaN rather than collapsing to a finite magnitude.
    assert!(Complex::new(f64::NAN, 1.0).norm().is_nan());
}

#[test]
fn numeric_element_sqrt_is_the_principal_root() {
    // √(3 + 4i) = 2 + i. The previous body used re² + im² (= 25) where the
    // magnitude |z| (= 5) is required, returning a value of magnitude 5.
    let root = <Complex<f64> as NumericElement>::sqrt(Complex::new(3.0, 4.0));
    assert!(close(root.re, 2.0, 1e-12), "re = {}", root.re);
    assert!(close(root.im, 1.0, 1e-12), "im = {}", root.im);

    // √(2i) = 1 + i.
    let root = <Complex<f64> as NumericElement>::sqrt(Complex::new(0.0, 2.0));
    assert!(close(root.re, 1.0, 1e-12), "re = {}", root.re);
    assert!(close(root.im, 1.0, 1e-12), "im = {}", root.im);

    // Squaring the principal root returns the input (magnitude check).
    let squared = root * root;
    assert!(close(squared.re, 0.0, 1e-12), "re = {}", squared.re);
    assert!(close(squared.im, 2.0, 1e-12), "im = {}", squared.im);

    // The inherent (polar) surface and the trait surface agree.
    let inherent = Complex::new(3.0_f64, 4.0).sqrt();
    let trait_root = <Complex<f64> as NumericElement>::sqrt(Complex::new(3.0, 4.0));
    assert!(close(inherent.re, trait_root.re, 1e-12));
    assert!(close(inherent.im, trait_root.im, 1e-12));
}
