//! Integer-to-float conversions: the crate's home for converting a count or
//! a signed integer into a float format. (`CastFrom`'s generic casts retire
//! under the stack's `CastFrom` retirement item.)
//!
//! Std has no lossless trait for these (`f64: From<usize>` does not exist,
//! since `usize` can exceed `f64`'s 53-bit significand), so the casts live
//! here, each under the contract its caller relies on: exact while the integer
//! fits the format's significand, rounded to nearest, ties to even, above it.
//!
//! The `const fn` forms are the inherent methods on [`F32`] and [`F64`]; the
//! generic forms are [`FloatElement::from_count`],
//! [`FloatElement::from_integer`] and [`TryFromCount::try_from_count`].
#![expect(
    clippy::cast_precision_loss,
    reason = "this module is the crate's home for integer-to-float conversion; \
              each cast rounds to nearest, ties to even, as the methods document"
)]

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
        Some(shift) if shift > 0 => {
            let discarded = m & ((1_u64 << shift) - 1);
            ((m >> shift) | u64::from(discarded != 0)) << shift
        }
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
