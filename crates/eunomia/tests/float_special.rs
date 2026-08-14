//! Value-semantic contract tests for the `FloatElement` special functions
//! (`log10`, `log2`, `erf`, `erfc`, `lgamma`), cross-checked against analytic
//! references rather than asserting mere existence.
//!
//! Calls use fully-qualified `FloatElement::…` syntax: std is stabilizing
//! same-named inherent float methods (`unstable_name_collisions`), and method
//! syntax would silently rebind to those, changing which implementation the
//! test verifies. Qualification pins the trait under test.

use eunomia::{FloatElement, F64};

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
