//! Integer powers by repeated squaring behind
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
pub(super) fn nth_power<T: NumericElement>(base: T, exp: u32) -> T {
    square_and_multiply(base, exp, T::ONE, |left, right| left * right)
}

/// The squaring schedule of [`nth_power`] over any associative product, so that the exponent-carrying
/// power of [`powi`] forms the same products.
#[inline(always)]
fn square_and_multiply<M: Copy>(
    mut base: M,
    mut exp: u32,
    one: M,
    multiply: impl Fn(M, M) -> M,
) -> M {
    let mut acc = one;
    while exp > 0 {
        if exp & 1 == 1 {
            acc = multiply(acc, base);
        }
        exp >>= 1;
        // A square is formed only while a higher bit remains: the one after
        // the last bit would never be read.
        if exp > 0 {
            base = multiply(base, base);
        }
    }
    acc
}

/// `base^n` for a signed exponent: the positive power, or for a negative exponent its reciprocal,
/// with the underflow-preserving downward recomputation when the upward power overflows, the
/// rounded exact power when the upward power saturates a finite-only format, and the
/// exponent-carrying power when the upward power is subnormal.
///
/// # Error
///
/// `n = 0` is exact. Let `e = |n| ≥ 1`, `u` the unit roundoff of `T`, `s` its least positive
/// value, `y = x^−e` and `B_e = (1 − u)^−e − 1`, at most `e·u / (1 − e·u)` for `e·u < 1`. For
/// `n > 0` the bound is that of [`nth_power`], while no product is subnormal or, on a finite-only
/// format, clamped to the largest finite value `M`, which is normal.
///
/// For `n < 0` the upward power `P̂` is `x^e·Π`, with `Π` a product of `e − 1` factors `1 + δ_j`
/// in `[(1 − u)^(e−1), (1 + u)^(e−1)]`, so `|1/P̂ − y| ≤ ((1 − u)^−(e−1) − 1)·|y|` while every
/// product is normal: always for `|x| > 1`, and for `|x| < 1` once `P̂` is above the least normal
/// value, as every product is at least `|P̂|`. The result `ŷ` is one of:
///
/// * The quotient `fl(1/P̂)`, when `P̂` is finite, unclamped and above the least normal. Its
///   rounding adds at most `u` relative where the quotient is normal, so `|ŷ − y| ≤ B_e·|y|` (as
///   `(1 + u)(1 − u)^−(e−1) ≤ (1 − u)^−e`), and at most `s/2` where it is subnormal, however small
///   `|y|`: `|ŷ − y| ≤ B_e·|y| + s/2` (`F8` `powi(224, -1)` is `2^-8` for `4.464e-3`, `0.125·|y|`).
/// * The exponent-carrying power, when `P̂` is at most the least normal: a subnormal `P̂` has lost
///   relative precision that the reciprocal carries into the result (`Bf8` `powi(7·2^-8, -3)`:
///   `P̂ = 2^-16` for `343·2^-24`, and `1/P̂` overflows for `y = 48913`). With `x = m·2^k` and `m`
///   in `[1, 2)`, the schedule of [`nth_power`] runs on pairs `(m, k)` whose products renormalize
///   by one exact halving, and `ŷ = scale(fl(1/m), −k)`. Products of magnitudes in `[1, 2)` are
///   normal, so the factors are the same `e − 1`, `fl(1/m)` adds the one of `B_e`, and the scaling
///   is exact unless the result is subnormal (`s/2` more) or overflows: `|ŷ − y| ≤ B_e·|y| + s/2`
///   again. The exponent saturates at `±2^31`, past every range.
///
/// Both bounds assume no overflow. The value before it is within `B_e` of `y`, so `ŷ` is finite
/// and within the bound when `|y|(1 + B_e)` is below `M + ulp(M)/2`, infinite (on a finite-only
/// format, `M`) when `|y|(1 − B_e)` reaches it, and either between (within the bound unless it
/// is that overflow value).
/// An overflowing upward power takes the downward recomputation (products of `1/x`, rounded `e`
/// times, possibly subnormal) and a saturated one the exact path below; neither is covered.
///
/// # Saturation
///
/// A finite-only format (`Bf4`, `F8`, `F4`: its [`INFINITY`](NumericElement::INFINITY) is finite)
/// clamps an overflowing product to its largest finite value `M`, so an upward power equal to `M`
/// may stand for any exact power from about `M` up. For `n > 0` the clamp is the result (`F8`
/// `powi(1.625, 11)` is `240`; the exact `208.6` rounds to `208`). For `n < 0`, `1 / M` is not
/// the quotient: `F8` `powi(2, -9)` has upward power `512`, clamped to `240`, and `1 / 240`
/// rounds to `2^-8`, not `2^-9`. The path forms the power `P̂` in
/// [`Accumulator`](FloatElement::Accumulator), inverts it there and rounds into `T` once (the
/// `from_f64` contract); inverting `x` first turns `Bf4` `powi(1.5, -3)` into `0` for `0.5`.
///
/// Take `|x| > 1` and `u = 2^-24`, the roundoff of the `f32` accumulator of all three formats.
/// While `P̂ < 2^126` every accumulator product and the quotient are normal and unclamped, so the
/// quotient meets the `n < 0` bound and converts to the correctly rounded `|x|^−e` unless a
/// midpoint of `T` lies inside that interval; a power of two is exact there, so it rounds
/// correctly (to even at a tie). That no midpoint does is finite arithmetic, settled by the exact
/// integer oracle in `tests/float_element/powi_saturation.rs` for every base and `e ≤ 128`, in
/// both regimes. From `P̂ ≥ 2^126`, or on overflow, the quotient is at most `2^-126` and converts
/// to zero, correctly: for `e < 2^23`, `(e − 1)·u < 1/2` keeps `|x|^e` above `2^125`, so the exact
/// `|x|^−e` lies below `2^-125`, far below half the least positive value of `T` (at least `2^-10`).
///
/// Beyond `e = 128` the result is a signed zero: `F8`'s least `|x| > 1` is `9/8` and its least
/// positive value `2^-9` (the others are larger), the quotient rounds to zero once `P̂ > 2^10`,
/// and `P̂ ≥ |x|^e / 2 > 2^20` for `128 < e < 2^23`, since `(9/8)^128 > 2^21`. For `e ≥ 2^23`,
/// `P̂` is at least the 23rd square of the base, `(9/8)^(2^23)` within a factor of two, which
/// overflows `f32`: `P̂ = ∞` and `1 / ∞ = 0` takes the sign of `x^n`.
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
        let quotient = T::ONE / acc;
        if upward_power_may_be_subnormal(quotient) {
            if let Some(exponent) = base.binary_exponent() {
                return carried_reciprocal_power(base, exponent, exp);
            }
        }
        quotient
    }
}

/// Whether the quotient `1/P̂` of the upward power `P̂` reaches the top two binades of `T`, which
/// every subnormal `P̂` does, and so does `P̂ = 0` (`1/0` is infinite or the largest finite value).
///
/// With least normal exponent `1 - emax`, `P̂` below `2^(1-emax)` gives a quotient of at least
/// `2^(emax-1)` (rounding is monotone and `2^(emax-1)` is representable), and four times it
/// overflows or clamps (`MAX + ulp/2 < 2^(emax+1)`). Every shipped format has that exponent range.
/// A `P̂` at the least normal also passes,
/// and `F4` passes `P̂ = 1/2` (its fourfold `M` clamps), harmlessly: the carried power meets the
/// same bound. NaN is neither infinite nor clamped.
#[inline(always)]
fn upward_power_may_be_subnormal<T: FloatElement>(quotient: T) -> bool {
    let doubled = quotient + quotient;
    let quadrupled = doubled + doubled;
    (!quadrupled.is_finite() && !quadrupled.is_nan()) || saturates(quadrupled)
}

/// A power as `significand·2^exponent` with the significand of magnitude in `[1, 2)`, so that
/// squaring leaves the range of `T` only when the power itself does.
#[derive(Clone, Copy)]
struct Carried<T> {
    significand: T,
    exponent: i32,
}

impl<T: FloatElement> Carried<T> {
    /// The product, renormalized by one exact halving when the significands reach `2`.
    #[inline(always)]
    fn multiply(self, other: Self) -> Self {
        let product = self.significand * other.significand;
        // Magnitudes in `[1, 2)` multiply to `[1, 4)`: finite, nonzero, binary exponent 0 or 1.
        let carry = product
            .binary_exponent()
            .expect("invariant: a product of significands in [1, 2) is finite and nonzero");
        Self {
            significand: product.scale_binary(-carry),
            exponent: self
                .exponent
                .saturating_add(other.exponent)
                .saturating_add(carry),
        }
    }
}

/// `base^-exp` for a finite nonzero `base` of binary exponent `base_exponent`, formed as a
/// [`Carried`] power so that no intermediate is subnormal: the reciprocal of the significand
/// scaled by the negated exponent.
#[inline(always)]
fn carried_reciprocal_power<T: FloatElement>(base: T, base_exponent: i32, exp: u32) -> T {
    let first = Carried {
        significand: base.scale_binary(-base_exponent),
        exponent: base_exponent,
    };
    let one = Carried {
        significand: T::ONE,
        exponent: 0,
    };
    let power = square_and_multiply(first, exp, one, Carried::multiply);
    (T::ONE / power.significand).scale_binary(power.exponent.saturating_neg())
}

/// Whether `power` is the clamped overflow of a finite-only format: the format
/// has no infinity (its [`INFINITY`](NumericElement::INFINITY) is finite) and
/// `power` has the largest finite magnitude.
#[inline(always)]
fn saturates<T: FloatElement>(power: T) -> bool {
    T::INFINITY.is_finite() && power.abs() == T::MAX_VALUE
}
