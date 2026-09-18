//! Value-semantic contract tests for the `FloatElement` special functions
//! (`log10`, `log2`, `erf`, `erfc`, `lgamma`, plus the ADR-0061-required
//! `asin`/`atan`/`acosh`/`asinh`/`atanh`/`exp2`/`exp_m1`/`ln_1p`),
//! cross-checked against analytic references rather than asserting mere
//! existence.
//!
//! Calls use fully-qualified `FloatElement::…` syntax: std is stabilizing
//! same-named inherent float methods (`unstable_name_collisions`), and method
//! syntax would silently rebind to those, changing which implementation the
//! test verifies. Qualification pins the trait under test.
//!
//! The ADR-0061 additions test two genuinely different routes, each with its
//! own correct oracle (never std's float methods, which are an independent
//! implementation with no derivable error bound relative to `libm`, and whose
//! *input* conditioning near a pole — `atanh` approaching ±1 — further
//! amplifies any inter-implementation disagreement to hundreds of ulp):
//! - `f64`/`F64` call `libm`'s `f64` entry point directly, so the correct
//!   assertion is bitwise equality against that exact same call (wiring, not
//!   accuracy) — see `new_transcendentals_f64_and_f64_wrapper_wire_to_libm_bitwise`.
//! - `f32` routes through `libm`'s independent `f32` algorithm, so it is
//!   compared against `libm`'s `f64` result rounded to `f32` (an oracle with
//!   zero input error, since every `f32` widens to `f64` exactly), within a
//!   bound derived from `libm`'s own documented per-function error where the
//!   source states one — see
//!   `new_transcendentals_f32_match_libm_f64_oracle_within_documented_bound`.

use eunomia::{FloatElement, F64};

fn close(a: f64, b: f64, tol: f64, label: &str) {
    assert!((a - b).abs() <= tol, "{label}: {a} vs {b} (tol {tol})");
}

/// Monotonic integer key for IEEE-754 floats (Bruce Dawson's ordering trick):
/// maps the bit pattern into a totally ordered `i64` so `|key(a) - key(b)|` is
/// the exact ULP distance between `a` and `b`, including across the zero
/// crossing and differing signs.
///
/// Used only for the `f32` route below: the `f64`/`F64` route asserts plain
/// bitwise equality against `libm` directly (the same call the impl makes),
/// which needs no ULP measure at all.
fn ulp_key32(x: f32) -> i64 {
    let bits = x.to_bits() as i32 as i64;
    if bits < 0 {
        0x8000_0000_i64 - bits
    } else {
        bits
    }
}
fn ulp_diff32(a: f32, b: f32) -> i64 {
    (ulp_key32(a) - ulp_key32(b)).abs()
}

/// Assert `a` and `b` agree within `ulp` units in the last place, treating
/// NaN-vs-NaN as agreement (NaN payload/sign is unspecified by IEEE 754 and
/// libm implementations are free to differ there). `ulp` must be a bound
/// derived at the call site (see the `f32` route test below), never fit to
/// an observed failure.
fn close_ulp32(a: f32, b: f32, ulp: i64, label: &str) {
    if a.is_nan() && b.is_nan() {
        return;
    }
    let d = ulp_diff32(a, b);
    assert!(d <= ulp, "{label}: {a} vs {b} ({d} ulp > bound {ulp})");
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

/// Precision-contract bound for the `F64` wrapper against the primitive `f64`
/// path.
///
/// Derivation: `F64` is `#[repr(transparent)]` over `f64`, so its documented
/// contract is *native* double precision — every operation must be evaluated in
/// `f64`, never widen-narrowed through `f32`. For a result of magnitude `O(1)`,
/// one `f64` ulp is `ε₆₄ = 2⁻⁵² ≈ 2.22e-16`; a native path that dispatches the
/// same `libm` entry point as the primitive impl is correctly rounded to the
/// identical bit pattern, so the true difference is exactly 0. The bound below
/// is set one decimal order above a single `f64` ulp — tight enough to reject
/// any `f32`-routed body, loose enough to survive a future re-routing to a
/// different but still-`f64`-accurate implementation.
///
/// Rejection power: an `f32` round trip carries relative error `ε₃₂ = 2⁻²³ ≈
/// 1.19e-7`, so on an `O(1)` result it lands near `1e-8`–`1e-7` — roughly eight
/// decimal orders outside this bound. `1e-15` therefore separates the two
/// implementations unambiguously rather than merely admitting the correct one.
const F64_NATIVE_TOL: f64 = 1e-15;

/// The five special functions the `F64` wrapper must evaluate natively.
///
/// `F64`'s whole contract is `f64` precision, but its `FloatElement` impl
/// inherits any method it does not override from the trait's `f32`-routed
/// defaults (`Self::from_f32(libm::<op>f(self.to_f32()))`). That default is
/// correct for `F16`/`Bf16`, which have no hardware transcendentals, and a HARD
/// precision-contract violation for `F64`, which does. This test pins each of
/// the five against the primitive `f64` impl — the same trait, the same
/// operation, the type whose native routing is already established — so a
/// dropped override fails here instead of silently discarding ~9 decimal
/// digits in every downstream generic algorithm instantiated at `F64`.
#[test]
fn f64_wrapper_special_functions_are_native_precision() {
    // Operands chosen so every result is O(1): the absolute bound above is then
    // equivalent to a relative one, and no assertion is weakened by scale.
    for &x in &[2.0f64, 10.0, 1000.0, 0.5] {
        close(
            FloatElement::log10(F64(x)).0,
            FloatElement::log10(x),
            F64_NATIVE_TOL,
            "F64 log10",
        );
        close(
            FloatElement::log2(F64(x)).0,
            FloatElement::log2(x),
            F64_NATIVE_TOL,
            "F64 log2",
        );
        close(
            FloatElement::lgamma(F64(x)).0,
            FloatElement::lgamma(x),
            F64_NATIVE_TOL,
            "F64 lgamma",
        );
    }
    for &x in &[0.3f64, 0.7, 1.5, 2.5] {
        close(
            FloatElement::erf(F64(x)).0,
            FloatElement::erf(x),
            F64_NATIVE_TOL,
            "F64 erf",
        );
        close(
            FloatElement::erfc(F64(x)).0,
            FloatElement::erfc(x),
            F64_NATIVE_TOL,
            "F64 erfc",
        );
    }
}

/// Independent analytic cross-check: agreeing with the primitive `f64` impl
/// only proves the two routes match, so anchor `F64` to closed-form values the
/// crate does not compute — `log₁₀(1000) = 3`, `log₂(1024) = 10`,
/// `lgamma(5) = ln 4!`, `erf(0) = 0`, `erfc(0) = 1` — at full `f64` precision.
#[test]
fn f64_wrapper_special_functions_match_analytic_references() {
    close(
        FloatElement::log10(F64(1000.0)).0,
        3.0,
        F64_NATIVE_TOL,
        "log10(1000) = 3",
    );
    close(
        FloatElement::log2(F64(1024.0)).0,
        10.0,
        F64_NATIVE_TOL,
        "log2(1024) = 10",
    );
    close(
        FloatElement::lgamma(F64(5.0)).0,
        24.0f64.ln(),
        1e-14,
        "lgamma(5) = ln 4!",
    );
    close(FloatElement::erf(F64(0.0)).0, 0.0, F64_NATIVE_TOL, "erf(0)");
    close(
        FloatElement::erfc(F64(0.0)).0,
        1.0,
        F64_NATIVE_TOL,
        "erfc(0)",
    );
    // erfc is the complement of erf; the identity holds to f64 precision only
    // if both are evaluated natively.
    for &x in &[0.3f64, 1.0, 2.5] {
        close(
            FloatElement::erfc(F64(x)).0,
            1.0 - FloatElement::erf(F64(x)).0,
            1e-14,
            "F64 erfc == 1 - erf",
        );
    }
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
