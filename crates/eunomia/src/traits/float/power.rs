//! Integer powers by repeated squaring: the kernel behind
//! [`FloatElement::powi`](super::FloatElement::powi).

use super::NumericElement;

/// `base^exp` for a `u32` exponent, by squaring in `T`.
///
/// Forms at most `2·⌊log₂ exp⌋ + 1` products for `exp ≥ 1` and none for
/// `exp = 0`, which gives one.
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
/// recomputation when the upward power overflows.
#[inline(always)]
pub(super) fn powi<T: NumericElement>(base: T, n: i32) -> T {
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
    } else {
        T::ONE / acc
    }
}
