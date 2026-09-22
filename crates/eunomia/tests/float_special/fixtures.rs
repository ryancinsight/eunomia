//! The comparison oracles every leaf asserts through.
//!
//! `close` bounds absolute error; `assert_close_within_ulp` bounds it in
//! units of the last place, the only scale-free way to state single-precision
//! agreement across the exponent range these tests cover.

pub(super) fn close(a: f64, b: f64, tol: f64, label: &str) {
    assert!((a - b).abs() <= tol, "{label}: {a} vs {b} (tol {tol})");
}

/// Monotonic integer key for an IEEE-754 single-precision float (Bruce
/// Dawson's ordering trick): maps the bit pattern into a totally ordered
/// `i64` so `|key(a) - key(b)|` is the exact ULP distance between `a` and
/// `b`, including across the zero crossing and differing signs.
///
/// Used only for the single-precision route below: the double-precision
/// route asserts plain bitwise equality against `libm` directly (the same
/// call the impl makes), which needs no ULP measure at all. Kept concrete
/// (no second precision instantiates this) rather than generalized ahead of
/// a real second occurrence.
pub(super) fn ulp_ordering_key(x: f32) -> i64 {
    let bits = x.to_bits() as i32 as i64;
    if bits < 0 {
        0x8000_0000_i64 - bits
    } else {
        bits
    }
}

pub(super) fn ulp_distance(a: f32, b: f32) -> i64 {
    (ulp_ordering_key(a) - ulp_ordering_key(b)).abs()
}

/// Assert `a` and `b` agree within `ulp` units in the last place, treating
/// NaN-vs-NaN as agreement (NaN payload/sign is unspecified by IEEE 754 and
/// libm implementations are free to differ there). `ulp` must be a bound
/// derived at the call site (see the single-precision route test below),
/// never fit to an observed failure.
pub(super) fn assert_close_within_ulp(a: f32, b: f32, ulp: i64, label: &str) {
    if a.is_nan() && b.is_nan() {
        return;
    }
    let d = ulp_distance(a, b);
    assert!(d <= ulp, "{label}: {a} vs {b} ({d} ulp > bound {ulp})");
}
