//! The ADR-0061 additions, each against its own correct oracle.
//!
//! `f64` and `F64` call `libm`'s f64 entry point, so the assertion is
//! bitwise equality against that same call -- wiring, not accuracy.
//! `f32` routes through `libm`'s independent f32 algorithm and is
//! compared against its f64 result rounded to f32, within `libm`'s own
//! documented error bound.

use eunomia::{FloatElement, F64};

use super::fixtures::close_ulp32;

/// `f64` route: `libm` **is** the implementation (the trait's `f64` override
/// and the `F64` wrapper's override both call the double-precision `libm`
/// entry point directly, with no intervening rounding), so the only correct
/// assertion is wiring, not accuracy — plain bitwise equality against the
/// exact same `libm` call the impl makes. This is not a comparison against
/// an independent oracle and carries no tolerance: if it ever needed one,
/// that would mean the impl is not actually calling `libm` as documented.
///
/// A dropped `f64`/`F64` override falls back to the trait's `f32`-routed
/// default (`Self::from_f32(libm::<op>f(self.to_f32()))`), which this test
/// catches: an `f32` widen-narrow round trip on an `O(1)` input differs from
/// the native `libm::<op>(x)` result by roughly `ε₃₂ ≈ 1.2e-7` relative,
/// which is billions of `f64` ulps — never bitwise-equal by accident.
#[test]
fn new_transcendentals_f64_and_f64_wrapper_wire_to_libm_bitwise() {
    for &x in &[0.3, 0.7, -0.5, 0.9] {
        assert_eq!(FloatElement::asin(x), libm::asin(x), "f64 asin");
        assert_eq!(FloatElement::asin(F64(x)).0, libm::asin(x), "F64 asin");
        assert_eq!(FloatElement::atanh(x), libm::atanh(x), "f64 atanh");
        assert_eq!(FloatElement::atanh(F64(x)).0, libm::atanh(x), "F64 atanh");
    }
    // acosh's domain is [1, ∞).
    for &x in &[1.5, 2.0, 10.0] {
        assert_eq!(FloatElement::acosh(x), libm::acosh(x), "f64 acosh");
        assert_eq!(FloatElement::acosh(F64(x)).0, libm::acosh(x), "F64 acosh");
    }
    for &x in &[0.3, 1.5, 2.0, 10.0] {
        assert_eq!(FloatElement::atan(x), libm::atan(x), "f64 atan");
        assert_eq!(FloatElement::atan(F64(x)).0, libm::atan(x), "F64 atan");
        assert_eq!(FloatElement::asinh(x), libm::asinh(x), "f64 asinh");
        assert_eq!(FloatElement::asinh(F64(x)).0, libm::asinh(x), "F64 asinh");
        assert_eq!(FloatElement::exp2(x), libm::exp2(x), "f64 exp2");
        assert_eq!(FloatElement::exp2(F64(x)).0, libm::exp2(x), "F64 exp2");
        assert_eq!(FloatElement::exp_m1(x), libm::expm1(x), "f64 exp_m1");
        assert_eq!(FloatElement::exp_m1(F64(x)).0, libm::expm1(x), "F64 exp_m1");
        assert_eq!(FloatElement::ln_1p(x), libm::log1p(x), "f64 ln_1p");
        assert_eq!(FloatElement::ln_1p(F64(x)).0, libm::log1p(x), "F64 ln_1p");
    }
}

/// Closed-form points independent of the bitwise-wiring check above: each
/// value below is either an exact mathematical identity landing on an
/// exactly representable `f64` (`exp2(10) = 2^10 = 1024`, no rounding
/// possible for an integer power of two in range) or a boundary the
/// algorithm special-cases directly rather than approximates (`atan(1) =
/// π/4` is the standard reduction-range edge in every arctangent
/// implementation of this family, evaluated as a table lookup / exact
/// argument-reduction case, not the general rational-polynomial path).
#[test]
fn new_transcendentals_f64_analytic_reference_points() {
    assert_eq!(FloatElement::atan(1.0f64), core::f64::consts::FRAC_PI_4);
    assert_eq!(FloatElement::exp2(10.0f64), 1024.0);
    assert_eq!(FloatElement::exp_m1(0.0f64), 0.0);
    assert_eq!(FloatElement::ln_1p(0.0f64), 0.0);
}

/// `f32` route: the trait's default routes through `libm`'s single-precision
/// entry point (e.g. `libm::asinf`), a genuinely different algorithm/codepath
/// from the `f64` entry point, so this is a real accuracy check needing a
/// derived bound — never `libm`'s own `f64`-vs-`f32` self-consistency, which
/// is what "wiring" would test.
///
/// Oracle: `libm::<op>(x as f64) as f32` — the `f64` route (verified exact
/// above) rounded down to `f32`. Every `f32` input is exactly representable
/// in `f64` (widening is lossless), so this oracle carries zero input
/// error — the only source of disagreement is the `f32` algorithm's own
/// rounding, which is exactly what each bound below measures.
///
/// Bound = `documented_ulp + 0.5` (the 0.5 covers the oracle's own
/// round-to-`f32` step), floored to the largest integer ULP count the bound
/// admits (an integer ULP distance `d` satisfies `d <= bound` iff `d <=
/// floor(bound)`). Where the source documents no bound, the assumed bound is
/// "1 ulp (correctly rounded) + 0.5 ulp (oracle rounding) = 2 ulp" per the
/// reviewed methodology, marked below as an assumption rather than a
/// citation.
#[test]
fn new_transcendentals_f32_match_libm_f64_oracle_within_documented_bound() {
    // Source paths cited below are relative to `libm-0.2.16/src/math/` in
    // the locked crate's registry checkout. Every oracle call casts the SAME
    // `f32` binding to `f64` (`f64::from(x)`) rather than writing a second
    // decimal literal: two independently-rounded literals for a value like
    // `1.05` are two different real numbers (the f32 and f64 roundings of
    // "1.05" differ in their last bits), which reintroduces the input
    // conditioning this oracle exists to eliminate.

    // asinf.rs: no documented ulp comment found in this file (checked the
    // whole file). Assumed bound: 2 ulp (1 correctly-rounded + 0.5 oracle).
    let x: f32 = 0.5;
    close_ulp32(
        FloatElement::asin(x),
        libm::asin(f64::from(x)) as f32,
        2,
        "f32 asin (undocumented bound, assumed 2 ulp)",
    );

    // atanf.rs: no documented ulp comment found in this file. Assumed bound:
    // 2 ulp (1 correctly-rounded + 0.5 oracle).
    let x: f32 = 1.5;
    close_ulp32(
        FloatElement::atan(x),
        libm::atan(f64::from(x)) as f32,
        2,
        "f32 atan (undocumented bound, assumed 2 ulp)",
    );

    // acoshf.rs:17 — "/* up to 2ulp error in [1,1.125] */" (the branch for
    // |x| < 2). Bound = floor(2.0 + 0.5) = 2. Test point kept inside the
    // documented sub-range.
    let x: f32 = 1.05;
    close_ulp32(
        FloatElement::acosh(x),
        libm::acosh(f64::from(x)) as f32,
        2,
        "f32 acosh (acoshf.rs:17, documented 2ulp in [1,1.125])",
    );

    // asinhf.rs:26 — "/* |x| >= 0x1p-12, up to 1.6ulp error in
    // [0.125,0.5] */". Bound = floor(1.6 + 0.5) = 2. Test point kept inside
    // the documented sub-range.
    let x: f32 = 0.3;
    close_ulp32(
        FloatElement::asinh(x),
        libm::asinh(f64::from(x)) as f32,
        2,
        "f32 asinh (asinhf.rs:26, documented 1.6ulp in [0.125,0.5])",
    );

    // atanhf.rs:24 — "/* |x| < 0.5, up to 1.7ulp error */". Bound =
    // floor(1.7 + 0.5) = 2. Test point kept inside the documented branch.
    let x: f32 = 0.3;
    close_ulp32(
        FloatElement::atanh(x),
        libm::atanh(f64::from(x)) as f32,
        2,
        "f32 atanh (atanhf.rs:24, documented 1.7ulp for |x|<0.5)",
    );

    // exp2f.rs:50 — "// Accuracy: Peak error < 0.501 ulp; ...". This is a
    // whole-function claim (not branch-restricted), so any representative
    // point applies. Bound = floor(0.501 + 0.5) = 1.
    let x: f32 = 1.5;
    close_ulp32(
        FloatElement::exp2(x),
        libm::exp2(f64::from(x)) as f32,
        1,
        "f32 exp2 (exp2f.rs:50, documented peak <0.501ulp)",
    );

    // expm1f.rs: no numeric ulp bound documented (only a prose accuracy
    // note). Assumed bound: 2 ulp (1 correctly-rounded + 0.5 oracle).
    let x: f32 = 1.0;
    close_ulp32(
        FloatElement::exp_m1(x),
        libm::expm1(f64::from(x)) as f32,
        2,
        "f32 exp_m1 (undocumented bound, assumed 2 ulp)",
    );

    // log1pf.rs: no documented ulp comment found in this file (the internal
    // polynomial's own error term is not a stated final-result ulp bound).
    // Assumed bound: 2 ulp (1 correctly-rounded + 0.5 oracle).
    let x: f32 = 1.0;
    close_ulp32(
        FloatElement::ln_1p(x),
        libm::log1p(f64::from(x)) as f32,
        2,
        "f32 ln_1p (undocumented bound, assumed 2 ulp)",
    );
}
