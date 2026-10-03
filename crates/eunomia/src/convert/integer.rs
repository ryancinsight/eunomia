//! Float-to-integer conversions: the crate's home for turning a float into an
//! integer, the one direction std has no conversion trait for.
//!
//! [`IntegerTarget`] names the integer types a float converts into. Its
//! [`from_rounded`](IntegerTarget::from_rounded) and
//! [`from_floor`](IntegerTarget::from_floor) round in the float element's own
//! precision, then convert the integral result through
//! [`from_truncated`](IntegerTarget::from_truncated), the one saturating
//! conversion. Every shipped float format widens exactly into `f64`, so the
//! integral value reaches the target unchanged.
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "this module is the crate's home for float-to-integer conversion; \
              Rust's float-to-integer `as` truncates toward zero, saturates at \
              the target's range and maps NaN to 0, as the trait documents"
)]

use crate::traits::{private, FloatElement};

/// An integer type a float converts into, saturating at its range.
///
/// Every conversion saturates: `+∞` and values above `MAX` give `MAX`, `-∞`
/// and values below `MIN` give `MIN`, and NaN gives `0`.
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

macro_rules! impl_integer_target {
    ($($t:ty),+) => {$(
        impl IntegerTarget for $t {
            #[inline]
            fn from_truncated(x: f64) -> Self {
                x as $t
            }
        }
    )+};
}

impl_integer_target!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

#[cfg(test)]
mod tests;
