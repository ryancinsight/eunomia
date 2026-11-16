//! Integer powers by repeated squaring: the kernel behind
//! [`FloatElement::nth_power`](super::FloatElement::nth_power) and
//! [`FloatElement::powi`](super::FloatElement::powi).

use super::{FloatElement, NumericElement};

/// `base^exp` for a `u32` exponent, by squaring in `T`.
///
/// Forms at most `2·⌊log₂ exp⌋ + 1` products for `exp ≥ 1` and none for
/// `exp = 0`, which gives one.
///
/// # Rounding error
///
/// Let `n = exp ≥ 1` and `u` be the unit roundoff of `T`: round to nearest
/// makes each product the exact one times `1 + δ` with `|δ| ≤ u`. Assume no
/// product overflows or is subnormal. The square `s_i` of the base `s_0 = x`
/// carries `2^i − 1` factors `1 + δ` counted with multiplicity (`s_0` none,
/// `s_i = fl(s_{i−1}²)` carries `2(2^(i−1) − 1) + 1`). The accumulator
/// multiplies in the `k` squares the set bits of `n` select, and its first
/// product `1·s` is exact, so the factors total `Σ(2^i − 1) + k − 1 = n − 1`
/// and the result is `xⁿ·Π(1 + δ_j)` over `n − 1` factors. Expanding the
/// product, `|Π(1 + δ_j) − 1| ≤ Π(1 + |δ_j|) − 1 ≤ (1 + u)^(n−1) − 1`, which
/// holds for every `n`. For `m = n − 1` with `m·u < 1`,
/// `(1 + u)^m ≤ (1 − u)^−m ≤ 1 / (1 − m·u)`, the first step because
/// `(1 + u)(1 − u) ≤ 1` and the second by Bernoulli's inequality
/// `(1 − u)^m ≥ 1 − m·u`, so the bound is at most `γ_m = m·u / (1 − m·u)`. For
/// `m·u ≥ 1` (`n > 2^24` for `f32`) `γ_m` is undefined and only the first
/// bound holds.
#[inline(always)]
pub(super) fn nth_power<T: NumericElement>(mut base: T, mut exp: u32) -> T {
    let mut acc = T::ONE;
    while exp > 0 {
        if exp & 1 == 1 {
            acc *= base;
        }
        exp >>= 1;
        // A square is formed only while a higher bit remains: the one after
        // the last bit would never be read.
        if exp > 0 {
            base *= base;
        }
    }
    acc
}

/// `base^n` for a signed exponent: the positive power, or for a negative
/// exponent its reciprocal, with the underflow-preserving downward
/// recomputation when the upward power overflows, and the rounded exact power
/// when it saturates a finite-only format.
///
/// # Error
///
/// `n = 0` is exact. Let `e = |n| ≥ 1`, `u` the unit roundoff of `T`, and every
/// product and the result be normal (no overflow or underflow, the result
/// included: a subnormal quotient carries a larger rounding error). For
/// `n > 0` the bound is that of [`nth_power`]. For `n < 0` the reciprocal of
/// the upward power `x^e·Π`, a product of `e − 1` factors `1 + δ_j` with
/// `Π` in `[(1 − u)^(e−1), (1 + u)^(e−1)]`, is `x^−e·(1 + δ₀) / Π`, so the
/// relative error is at most `(1 + u)(1 − u)^−(e−1) − 1 ≤ (1 − u)^−e − 1`
/// (`(1 + u)(1 − u) ≤ 1`; the other side is smaller), which is at most
/// `e·u / (1 − e·u)` for `e·u < 1` (Bernoulli, as for the kernel bound).
///
/// # Saturation
///
/// A finite-only format (`Bf4`, `F8`, `F4`: its
/// [`INFINITY`](NumericElement::INFINITY) is finite) clamps an overflowing
/// product to its largest finite value `M`, so an upward power equal to `M` may
/// stand for any exact power from about `M` up and `1 / M` is not its
/// quotient: `F8` `powi(2, -9)` has upward power `512`, clamped to `240`, and
/// `1 / 240` rounds to `2^-8` where `2^-9` is correct. Recomputing as
/// `nth_power(1 / x, |n|)` rounds `1 / x` early and turns `Bf4`
/// `powi(1.5, -3)` (correct `0.5`) into `0`. The path instead forms the power
/// `P̂` in [`Accumulator`](FloatElement::Accumulator), inverts it there and
/// rounds into `T` once, by the `from_f64` contract.
///
/// Take `|x| > 1` (only such a power reaches `M`) and `u = 2^-24`, the roundoff
/// of the `f32` accumulator of all three formats. The accumulator quotient is
/// within the `n < 0` bound above, and its conversion is the correctly rounded
/// `|x|^−e` unless a rounding midpoint of `T` lies inside that interval. A
/// power of two is exact in `f32`; that no midpoint is reached for any other
/// base is finite arithmetic, which `tests/float_element/powi_saturation.rs`
/// settles with an exact integer oracle for every base and `e ≤ 128`.
///
/// Beyond `e = 128` the result is a signed zero. `F8`'s least `|x| > 1` is
/// `9/8` and its least positive value `2^-9` (the others have larger ones), and
/// the quotient rounds to zero once `P̂ > 2^10`. For `128 < e < 2^23`,
/// `(e − 1)·u < 1/2` gives `P̂ ≥ |x|^e / 2 > 2^20`, since `(9/8)^128 > 2^21`.
/// For `e ≥ 2^23`, `P̂` is at least the 23rd square of the base, which is
/// `(9/8)^(2^23)` within a factor of two and overflows `f32`: `P̂ = ∞`, and
/// `1 / ∞ = 0` takes the sign of `x^n`.
#[inline(always)]
pub(super) fn powi<T: FloatElement>(base: T, n: i32) -> T {
    // `unsigned_abs`, never `-n`: `i32::MIN` has no positive `i32`
    // negation (`-n` panics in debug, wraps to `MIN` and returns `ONE` in
    // release). `u32` holds 2^31 exactly.
    let exp = n.unsigned_abs();
    let acc = nth_power(base, exp);
    if n >= 0 {
        return acc;
    }
    // No `is_infinite` on `NumericElement` (deliberately: infinities are not
    // portable across all element types); infinite ⟺ neither finite nor NaN.
    if !acc.is_finite() && !acc.is_nan() {
        // Cold path: the upward power overflowed, so the true quotient
        // underflows — recompute working downward to preserve IEEE gradual
        // underflow instead of flushing to +0 (`(-2)^-130` is the subnormal
        // 2^-130). Inverting first unconditionally would instead round the
        // reciprocal early and amplify that error |n|-fold (measured: 8 ulp
        // off std at |n| = 10).
        nth_power(T::ONE / base, exp)
    } else if saturates(acc) {
        let wide = nth_power(base.to_accumulator(), exp);
        T::from_accumulator(<T::Accumulator as NumericElement>::ONE / wide)
    } else {
        T::ONE / acc
    }
}

/// Whether `power` is the clamped overflow of a finite-only format: the format
/// has no infinity (its [`INFINITY`](NumericElement::INFINITY) is finite) and
/// `power` has the largest finite magnitude.
#[inline(always)]
fn saturates<T: FloatElement>(power: T) -> bool {
    T::INFINITY.is_finite() && power.abs() == T::MAX_VALUE
}
