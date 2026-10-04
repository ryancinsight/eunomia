//! Float-to-integer conversions: the crate's home for turning a float into an
//! integer, the one direction std has no conversion trait for.
//!
//! [`IntegerTarget`] names the integer types a float converts into. Its
//! [`from_rounded`](IntegerTarget::from_rounded),
//! [`try_from_rounded`](IntegerTarget::try_from_rounded), and
//! [`from_floor`](IntegerTarget::from_floor) round in the float element's own
//! precision. The infallible methods then convert through
//! [`from_truncated`](IntegerTarget::from_truncated), the one saturating
//! conversion, while the checked method rejects non-finite and out-of-range
//! results. Every shipped float format widens exactly into `f64`, so the
//! rounded integral value reaches the target unchanged.
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "this module is the crate's home for float-to-integer conversion; \
              Rust's float-to-integer `as` truncates toward zero, saturates at \
              the target's range and maps NaN to 0, as the trait documents"
)]

use core::fmt;

use crate::traits::{private, FloatElement};

/// Why a checked float-to-integer conversion failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IntegerConversionError {
    /// The input was NaN.
    NotANumber,
    /// The input was positive or negative infinity.
    Infinite,
    /// The rounded finite value is outside the destination integer's range.
    OutOfRange,
}

impl fmt::Display for IntegerConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotANumber => "NaN cannot be represented as an integer",
            Self::Infinite => "infinity cannot be represented as an integer",
            Self::OutOfRange => "rounded value is outside the destination integer range",
        })
    }
}

impl core::error::Error for IntegerConversionError {}

/// An integer type a float converts into.
///
/// The infallible conversions saturate: `+∞` and values above `MAX` give
/// `MAX`, `-∞` and values below `MIN` give `MIN`, and NaN gives `0`.
/// [`try_from_rounded`](Self::try_from_rounded) instead reports these cases.
///
/// Sealed: implemented for the primitive integers `i8` to `i64`, `isize`,
/// `u8` to `u64`, and `usize`.
pub trait IntegerTarget: private::Sealed + Copy {
    /// Converts `x`, truncating toward zero and saturating at `Self`'s range.
    ///
    /// ```
    /// use eunomia::convert::IntegerTarget;
    ///
    /// assert_eq!(u8::from_truncated(255.9), 255);
    /// assert_eq!(u8::from_truncated(256.0), 255);
    /// assert_eq!(u8::from_truncated(-0.9), 0);
    /// assert_eq!(i8::from_truncated(f64::NEG_INFINITY), i8::MIN);
    /// assert_eq!(usize::from_truncated(f64::NAN), 0);
    /// ```
    fn from_truncated(x: f64) -> Self;

    /// Rounds `x` to the nearest integer, ties to even, and converts it,
    /// saturating at `Self`'s range.
    ///
    /// The rounding runs in `T`
    /// ([`round_ties_even`](FloatElement::round_ties_even)); the integral
    /// result widens exactly into `f64` for every shipped format.
    ///
    /// ```
    /// use eunomia::convert::IntegerTarget;
    /// use eunomia::{FloatElement, F16};
    ///
    /// assert_eq!(u8::from_rounded(2.5_f32), 2);
    /// assert_eq!(u8::from_rounded(F16::from_f64(255.5)), 255);
    /// assert_eq!(u32::from_rounded(-0.5_f64), 0);
    /// assert_eq!(usize::from_rounded(f64::NAN), 0);
    /// ```
    #[inline]
    fn from_rounded<T: FloatElement>(x: T) -> Self {
        Self::from_truncated(x.round_ties_even().to_f64())
    }

    /// Rounds `x` to the nearest integer, ties to even, and converts it only
    /// when that rounded value is representable by `Self`.
    ///
    /// Rounding runs in `T` before its exact widening into `f64`. NaN returns
    /// [`IntegerConversionError::NotANumber`], either infinity returns
    /// [`IntegerConversionError::Infinite`], and a finite rounded value below
    /// `MIN` or above `MAX` returns [`IntegerConversionError::OutOfRange`].
    ///
    /// ```
    /// use eunomia::convert::{IntegerConversionError, IntegerTarget};
    ///
    /// assert_eq!(u8::try_from_rounded(2.5_f32), Ok(2));
    /// assert_eq!(u8::try_from_rounded(255.0_f32), Ok(255));
    /// assert_eq!(
    ///     u8::try_from_rounded(255.5_f32),
    ///     Err(IntegerConversionError::OutOfRange)
    /// );
    /// assert_eq!(
    ///     i16::try_from_rounded(f64::NAN),
    ///     Err(IntegerConversionError::NotANumber)
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`IntegerConversionError`] when `x` is non-finite or its
    /// rounded value is outside `Self`'s range.
    fn try_from_rounded<T: FloatElement>(x: T) -> Result<Self, IntegerConversionError>;

    /// Rounds `x` toward `-∞` and converts the result, saturating at `Self`'s
    /// range.
    ///
    /// The rounding runs in `T` ([`floor`](FloatElement::floor)); the
    /// integral result widens exactly into `f64` for every shipped format.
    ///
    /// ```
    /// use eunomia::convert::IntegerTarget;
    ///
    /// assert_eq!(usize::from_floor(2.9_f32), 2);
    /// assert_eq!(i32::from_floor(-0.5_f64), -1);
    /// assert_eq!(usize::from_floor(-0.5_f64), 0);
    /// assert_eq!(usize::from_floor(f64::INFINITY), usize::MAX);
    /// ```
    #[inline]
    fn from_floor<T: FloatElement>(x: T) -> Self {
        Self::from_truncated(x.floor().to_f64())
    }
}

#[inline]
fn upper_exclusive(magnitude_bits: u32) -> f64 {
    const BINARY64_EXPONENT_BIAS: u64 = 1023;
    const BINARY64_FRACTION_BITS: u32 = 52;

    let exponent = u64::from(magnitude_bits) + BINARY64_EXPONENT_BIAS;
    f64::from_bits(exponent << BINARY64_FRACTION_BITS)
}

macro_rules! magnitude_bits {
    (signed, $t:ty) => {
        <$t>::BITS - 1
    };
    (unsigned, $t:ty) => {
        <$t>::BITS
    };
}

macro_rules! lower_inclusive {
    (signed, $upper:ident) => {
        -$upper
    };
    (unsigned, $upper:ident) => {
        0.0
    };
}

macro_rules! impl_integer_target {
    ($signedness:ident; $($t:ty),+) => {$(
        impl IntegerTarget for $t {
            #[inline]
            fn from_truncated(x: f64) -> Self {
                x as $t
            }

            #[inline]
            fn try_from_rounded<T: FloatElement>(
                x: T,
            ) -> Result<Self, IntegerConversionError> {
                let rounded = x.round_ties_even();
                if rounded.is_nan() {
                    return Err(IntegerConversionError::NotANumber);
                }
                if !rounded.is_finite() {
                    return Err(IntegerConversionError::Infinite);
                }

                let value = rounded.to_f64();
                let upper = upper_exclusive(magnitude_bits!($signedness, $t));
                let lower = lower_inclusive!($signedness, upper);
                if value < lower || value >= upper {
                    Err(IntegerConversionError::OutOfRange)
                } else {
                    Ok(value as $t)
                }
            }
        }
    )+};
}

impl_integer_target!(signed; i8, i16, i32, i64, isize);
impl_integer_target!(unsigned; u8, u16, u32, u64, usize);

#[cfg(test)]
mod tests;
