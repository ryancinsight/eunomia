//! Domain boundaries and the cancellation-avoiding pair.
//!
//! Each function is exercised at the edges of its mathematical domain,
//! where the contract is a specific value, a pole, or a NaN -- not an
//! approximation. `exp_m1`/`ln_1p` additionally assert the accuracy near
//! zero that is their whole reason for existing.

use eunomia::FloatElement;

// ── ADR-0061-required inverse trig / hyperbolic / cancellation-safe ops ──
//
// Special (exact, non-rounded) values below agree bitwise across every route
// because they hit an IEEE-754 special-case or an exact-algorithm branch
// rather than a general rounded approximation; no tolerance is involved.

#[test]
fn asin_domain_edges() {
    // Domain edges: exact at ±1 (agrees bitwise — no rounding involved).
    assert_eq!(FloatElement::asin(1.0f64), core::f64::consts::FRAC_PI_2);
    assert_eq!(FloatElement::asin(-1.0f64), -core::f64::consts::FRAC_PI_2);
    // Outside [-1, 1]: undefined, NaN.
    assert!(FloatElement::asin(1.000_000_1f64).is_nan());
    assert!(FloatElement::asin(-1.000_000_1f64).is_nan());
    // ±0 preserved (odd function).
    assert_eq!(FloatElement::asin(0.0f64).to_bits(), 0.0f64.to_bits());
    assert_eq!(FloatElement::asin(-0.0f64).to_bits(), (-0.0f64).to_bits());
    assert!(FloatElement::asin(f64::NAN).is_nan());
}

#[test]
fn atan_domain_edges() {
    // Defined for all reals; ±∞ maps to ±π/2 exactly (special-value rule).
    assert_eq!(
        FloatElement::atan(f64::INFINITY),
        core::f64::consts::FRAC_PI_2
    );
    assert_eq!(
        FloatElement::atan(f64::NEG_INFINITY),
        -core::f64::consts::FRAC_PI_2
    );
    assert_eq!(FloatElement::atan(0.0f64).to_bits(), 0.0f64.to_bits());
    assert_eq!(FloatElement::atan(-0.0f64).to_bits(), (-0.0f64).to_bits());
    assert!(FloatElement::atan(f64::NAN).is_nan());
}

#[test]
fn acosh_domain_edges() {
    // Domain [1, ∞); below 1 is undefined.
    assert_eq!(FloatElement::acosh(1.0f64), 0.0);
    assert!(FloatElement::acosh(0.999_999f64).is_nan());
    assert!(FloatElement::acosh(0.0f64).is_nan());
    assert!(FloatElement::acosh(-1.0f64).is_nan());
    assert_eq!(FloatElement::acosh(f64::INFINITY), f64::INFINITY);
    assert!(FloatElement::acosh(f64::NAN).is_nan());
}

#[test]
fn asinh_domain_edges() {
    // Defined for all reals; odd function, ±0 and ±∞ preserved exactly.
    assert_eq!(FloatElement::asinh(0.0f64).to_bits(), 0.0f64.to_bits());
    assert_eq!(FloatElement::asinh(-0.0f64).to_bits(), (-0.0f64).to_bits());
    assert_eq!(FloatElement::asinh(f64::INFINITY), f64::INFINITY);
    assert_eq!(FloatElement::asinh(f64::NEG_INFINITY), f64::NEG_INFINITY);
    assert!(FloatElement::asinh(f64::NAN).is_nan());
}

#[test]
fn atanh_domain_edges() {
    // Domain (-1, 1); ±1 is the pole (±∞), outside is undefined (NaN).
    assert_eq!(FloatElement::atanh(1.0f64), f64::INFINITY);
    assert_eq!(FloatElement::atanh(-1.0f64), f64::NEG_INFINITY);
    assert!(FloatElement::atanh(1.5f64).is_nan());
    assert!(FloatElement::atanh(-1.5f64).is_nan());
    assert_eq!(FloatElement::atanh(0.0f64).to_bits(), 0.0f64.to_bits());
    assert_eq!(FloatElement::atanh(-0.0f64).to_bits(), (-0.0f64).to_bits());
    assert!(FloatElement::atanh(f64::NAN).is_nan());
}

#[test]
fn exp2_domain_edges() {
    assert_eq!(FloatElement::exp2(0.0f64), 1.0);
    assert_eq!(FloatElement::exp2(1.0f64), 2.0);
    assert_eq!(FloatElement::exp2(f64::NEG_INFINITY), 0.0);
    assert_eq!(FloatElement::exp2(f64::INFINITY), f64::INFINITY);
    assert!(FloatElement::exp2(f64::NAN).is_nan());
}

#[test]
fn exp_m1_domain_edges() {
    assert_eq!(FloatElement::exp_m1(0.0f64).to_bits(), 0.0f64.to_bits());
    assert_eq!(FloatElement::exp_m1(-0.0f64).to_bits(), (-0.0f64).to_bits());
    assert_eq!(FloatElement::exp_m1(f64::NEG_INFINITY), -1.0);
    assert_eq!(FloatElement::exp_m1(f64::INFINITY), f64::INFINITY);
    assert!(FloatElement::exp_m1(f64::NAN).is_nan());
}

#[test]
fn ln_1p_domain_edges() {
    assert_eq!(FloatElement::ln_1p(0.0f64).to_bits(), 0.0f64.to_bits());
    assert_eq!(FloatElement::ln_1p(-1.0f64), f64::NEG_INFINITY);
    assert!(FloatElement::ln_1p(-1.5f64).is_nan());
    assert!(FloatElement::ln_1p(-2.0f64).is_nan());
    assert_eq!(FloatElement::ln_1p(f64::INFINITY), f64::INFINITY);
    assert!(FloatElement::ln_1p(f64::NAN).is_nan());
}

/// The whole reason `exp_m1`/`ln_1p` exist rather than `exp(x) - 1` /
/// `ln(1.0 + x)`: near zero, the naive forms subtract two nearly-equal
/// values and cancel away most of the significant digits, while the
/// dedicated routines keep full relative precision by never forming the
/// cancelling intermediate.
///
/// At `x = 1e-10`, the true value of both `exp(x) - 1` and `ln(1 + x)` is
/// `x` to within `O(x²) ≈ 1e-20` — far below `f64` precision, so the exact
/// answer is `x` itself at this scale. The naive forms lose ~8 of `f64`'s 16
/// significant decimal digits to cancellation (relative error ~8e-8,
/// consistent with accumulating one `f64` rounding at the `1.0 + x`/`exp(x)`
/// step relative to a result of magnitude `x`); the dedicated routines stay
/// accurate to machine epsilon.
#[test]
fn exp_m1_and_ln_1p_avoid_cancellation() {
    let x = 1e-10_f64;

    let naive_exp_m1 = x.exp() - 1.0;
    let naive_ln_1p = (1.0 + x).ln();
    let exact_exp_m1 = FloatElement::exp_m1(x);
    let exact_ln_1p = FloatElement::ln_1p(x);

    // The naive forms are off by a relative error orders of magnitude larger
    // than f64 epsilon (~2.2e-16) — cancellation is real and measurable.
    let naive_exp_m1_rel_err = ((naive_exp_m1 - x) / x).abs();
    let naive_ln_1p_rel_err = ((naive_ln_1p - x) / x).abs();
    assert!(
        naive_exp_m1_rel_err > 1e-9,
        "naive exp(x)-1 should show cancellation error, got rel err {naive_exp_m1_rel_err:e}"
    );
    assert!(
        naive_ln_1p_rel_err > 1e-9,
        "naive ln(1+x) should show cancellation error, got rel err {naive_ln_1p_rel_err:e}"
    );

    // The dedicated routines keep full relative precision (rel error at f64
    // epsilon scale, not the ~8e-8 the naive forms carry).
    let exact_exp_m1_rel_err = ((exact_exp_m1 - x) / x).abs();
    let exact_ln_1p_rel_err = ((exact_ln_1p - x) / x).abs();
    assert!(
        exact_exp_m1_rel_err < 1e-9,
        "exp_m1 should keep full relative precision, got rel err {exact_exp_m1_rel_err:e}"
    );
    assert!(
        exact_ln_1p_rel_err < 1e-9,
        "ln_1p should keep full relative precision, got rel err {exact_ln_1p_rel_err:e}"
    );
}
