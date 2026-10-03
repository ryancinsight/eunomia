//! Integer-to-float conversions: the crate's home for converting a count or
//! a signed integer into a float format. (`CastFrom`'s generic casts retire
//! under the stack's `CastFrom` retirement item.)
//!
//! Std has no lossless trait for these (`f64: From<usize>` does not exist,
//! since `usize` can exceed `f64`'s 53-bit significand), so the casts live
//! here, each under the contract its caller relies on: exact while the integer
//! fits the format's significand, rounded to nearest, ties to even, above it.
//!
//! The concrete forms are the inherent methods on [`F32`] and [`F64`] (`const
//! fn` except the count reciprocal); the generic forms are
//! [`FloatElement::from_count`], [`FloatElement::from_count_reciprocal`],
//! [`FloatElement::from_integer`] and [`TryFromCount::try_from_count`].
#![expect(
    clippy::cast_precision_loss,
    reason = "this module is the crate's home for integer-to-float conversion; \
              each cast rounds to nearest, ties to even, as the methods document"
)]

use crate::convert::odd_truncated;
use crate::traits::{CountRangeError, FloatElement, NumericElement, TryFromCount};
use crate::types::{Complex, F32, F64, I16, I32, I8};

impl F32 {
    /// Converts the count `n`, rounding to nearest, ties to even.
    ///
    /// Exact for `n <= 2^24`.
    ///
    /// ```
    /// use eunomia::F32;
    ///
    /// const LEN: F32 = F32::from_count(1 << 24);
    /// assert_eq!(LEN.0, 16_777_216.0);
    /// assert_eq!(F32::from_count((1 << 24) + 1).0, 16_777_216.0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_count(n: usize) -> Self {
        Self(n as f32)
    }

    /// Converts the signed integer `k`, rounding to nearest, ties to even.
    ///
    /// Exact for `|k| <= 2^24`.
    #[inline]
    #[must_use]
    pub const fn from_integer(k: i64) -> Self {
        Self(k as f32)
    }

    /// Converts the reciprocal `1/n` of the count `n`, rounding to nearest,
    /// ties to even.
    ///
    /// Correctly rounded for every count `n >= 1`: the quotient is formed in
    /// integer arithmetic and rounded once (see `reciprocal_significand` in
    /// the conversion module). `n = 0` yields the IEEE quotient `1/0`, `+inf`.
    ///
    /// ```
    /// use eunomia::F32;
    ///
    /// assert_eq!(F32::from_count_reciprocal(4).0, 0.25);
    /// assert_eq!(F32::from_count_reciprocal(0).0, f32::INFINITY);
    /// ```
    #[inline]
    #[must_use]
    pub fn from_count_reciprocal(n: usize) -> Self {
        Self(reciprocal_to_24_bits(n, Rounding::NearestEven))
    }
}

impl F64 {
    /// Converts the count `n`, rounding to nearest, ties to even.
    ///
    /// Exact for `n <= 2^53`.
    ///
    /// ```
    /// use eunomia::F64;
    ///
    /// const LEN: F64 = F64::from_count(1 << 53);
    /// assert_eq!(LEN.0, 9_007_199_254_740_992.0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_count(n: usize) -> Self {
        Self(n as f64)
    }

    /// Converts the signed integer `k`, rounding to nearest, ties to even.
    ///
    /// Exact for `|k| <= 2^53`.
    #[inline]
    #[must_use]
    pub const fn from_integer(k: i64) -> Self {
        Self(k as f64)
    }

    /// Converts the reciprocal `1/n` of the count `n`, rounding to nearest,
    /// ties to even.
    ///
    /// Correctly rounded for every count `n >= 1`, including the counts past
    /// `2^53` that `f64` cannot hold, which `1.0 / n` would round twice.
    /// `n = 0` yields `+inf`.
    ///
    /// ```
    /// use eunomia::F64;
    ///
    /// assert_eq!(F64::from_count_reciprocal(8).0, 0.125);
    /// assert_eq!(F64::from_count_reciprocal(3).0, 1.0 / 3.0);
    /// ```
    #[inline]
    #[must_use]
    pub fn from_count_reciprocal(n: usize) -> Self {
        Self(
            match reciprocal_significand(n, f64::MANTISSA_DIGITS, Rounding::NearestEven) {
                // `q <= 2^53` is exact, and `2^-s` with `s <= 116` is a normal
                // `f64`, so the product is the rounded quotient itself.
                Some((q, s)) => {
                    q as f64 * f64::from_bits(u64::from(F64_EXPONENT_BIAS - s) << F64_FRACTION_BITS)
                }
                None => f64::INFINITY,
            },
        )
    }
}

/// `f32` exponent bias.
const F32_EXPONENT_BIAS: u32 = 127;
/// `f32` fraction field width.
const F32_FRACTION_BITS: u32 = 23;
/// `f64` exponent bias.
const F64_EXPONENT_BIAS: u32 = 1023;
/// `f64` fraction field width.
const F64_FRACTION_BITS: u32 = 52;

/// How [`reciprocal_significand`] rounds an inexact quotient.
#[derive(Clone, Copy)]
enum Rounding {
    /// Round to nearest; a tie cannot occur (see [`reciprocal_significand`]).
    NearestEven,
    /// Truncate and set the last kept bit when the quotient is inexact.
    Odd,
}

/// The reciprocal of a count `n >= 1` as `(q, s)`: `q * 2^-s` is `1/n`
/// rounded to `precision` significant bits as `rounding` directs. `None` for
/// `n = 0`.
///
/// With `n` in `[2^(w-1), 2^w)`, `s = w - 1 + precision` places `2^s / n` in
/// `(2^(precision-1), 2^precision]`, so `q = floor(2^s / n)` carries
/// `precision` bits; it reaches `2^precision` only for a power-of-two `n`,
/// whose reciprocal is exact. The remainder `2^s mod n` is zero exactly when
/// `1/n` has `precision` bits. A rounding tie cannot occur: `1/n` on a
/// midpoint would need `n * (2q + 1) = 2^(s+1)`, and `2q + 1 > 1` is odd.
/// `s <= 63 + 53` keeps `2^s` inside `u128` for every count and both
/// precisions this module uses.
fn reciprocal_significand(n: usize, precision: u32, rounding: Rounding) -> Option<(u128, u32)> {
    let divisor = n as u128;
    if divisor == 0 {
        return None;
    }
    let width = u128::BITS - divisor.leading_zeros();
    let scale = width - 1 + precision;
    let numerator = 1_u128 << scale;
    let quotient = numerator / divisor;
    let remainder = numerator % divisor;
    let rounded = if remainder == 0 {
        quotient
    } else {
        match rounding {
            Rounding::NearestEven => quotient + u128::from(2 * remainder > divisor),
            Rounding::Odd => quotient | 1,
        }
    };
    Some((rounded, scale))
}

/// `1/n` rounded to 24 bits as `rounding` directs, as an exact `f32`: `q`
/// is at most `2^24` and `2^-s` with `s <= 87` is a normal `f32`. `n = 0`
/// yields `+inf`.
fn reciprocal_to_24_bits(n: usize, rounding: Rounding) -> f32 {
    match reciprocal_significand(n, f32::MANTISSA_DIGITS, rounding) {
        Some((q, s)) => q as f32 * f32::from_bits((F32_EXPONENT_BIAS - s) << F32_FRACTION_BITS),
        None => f32::INFINITY,
    }
}

/// `1/n` rounded to odd at 24 bits: the reciprocal counterpart of
/// [`odd_rounded_count`], under the same single-rounding argument
/// ([`odd_rounded_magnitude`]). The result is a normal `f32` (at least
/// `2^-64`), so a narrower format's subnormal grid, whose points and midpoints
/// are even multiples of the result's quantum, keeps that argument too.
#[inline]
pub(crate) fn odd_rounded_count_reciprocal(n: usize) -> f32 {
    reciprocal_to_24_bits(n, Rounding::Odd)
}

/// Rounds the magnitude `m` to 24 significant bits with round-to-odd: the
/// leading 24 bits are kept and the last kept bit is set when any discarded
/// bit is nonzero. The result is exact in `f32`.
///
/// Narrowing it with round-to-nearest-even into a format of precision
/// `p <= 22` equals rounding `m` directly. Every `p`-bit value and every
/// midpoint between two of them has at most `p + 1` significant bits, so it is
/// a 24-bit value whose last bit is zero. When `m` is not a 24-bit value the
/// result is odd, so it is none of those points. Both `m` and the result lie
/// strictly inside the open interval between the two consecutive *even*
/// 24-bit values around `m`; that interval holds no `p`-bit value and no
/// midpoint, so both round to the same neighbour. A plain
/// round-to-nearest intermediate fails here: it can land exactly on a
/// midpoint `m` lies above, and the tie then goes the wrong way.
#[inline]
pub(crate) fn odd_rounded_magnitude(m: u64) -> f32 {
    let width = u64::BITS - m.leading_zeros();
    let kept = match width.checked_sub(f32::MANTISSA_DIGITS) {
        Some(shift) if shift > 0 => odd_truncated(m, shift) << shift,
        _ => m,
    };
    kept as f32
}

/// [`odd_rounded_magnitude`] of a count.
#[inline]
pub(crate) fn odd_rounded_count(n: usize) -> f32 {
    odd_rounded_magnitude(n as u64)
}

impl<T: FloatElement> TryFromCount for T {
    #[inline]
    fn try_from_count(n: usize) -> Result<Self, CountRangeError> {
        Ok(Self::from_count(n))
    }
}

/// A count embeds as the real part: `n + 0i`, with the real part's contract.
impl<T: NumericElement> TryFromCount for Complex<T> {
    #[inline]
    fn try_from_count(n: usize) -> Result<Self, CountRangeError> {
        T::try_from_count(n).map(|re| Self::new(re, T::ZERO))
    }
}

macro_rules! impl_try_from_count_int {
    ($($t:ty),+) => {$(
        impl TryFromCount for $t {
            #[inline]
            fn try_from_count(n: usize) -> Result<Self, CountRangeError> {
                Self::try_from(n).map_err(|_| CountRangeError::new::<Self>(n))
            }
        }
    )+};
}

impl_try_from_count_int!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

macro_rules! impl_try_from_count_int_wrapper {
    ($($wrapper:ident($inner:ty)),+) => {$(
        impl TryFromCount for $wrapper {
            #[inline]
            fn try_from_count(n: usize) -> Result<Self, CountRangeError> {
                <$inner>::try_from(n)
                    .map($wrapper)
                    .map_err(|_| CountRangeError::new::<Self>(n))
            }
        }
    )+};
}

impl_try_from_count_int_wrapper!(I8(i8), I16(i16), I32(i32));

#[cfg(test)]
mod tests;
