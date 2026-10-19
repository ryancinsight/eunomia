//! Native float construction: the crate's home for converting a count,
//! signed integer or floating-point input into a native float format.
//!
//! Std has no lossless trait for these (`f64: From<usize>` and
//! `f32: From<f64>` do not exist), so the casts live
//! here under their documented contracts. Integer construction is exact while
//! the integer fits the significand; inexact construction rounds to nearest,
//! ties to even. Floating-point construction also preserves signed zero and
//! follows the native format's overflow, underflow and NaN rules.
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
    /// Converts a binary64 value into binary32, rounding to nearest, ties to even.
    ///
    /// Every input is admitted. Signed zero is preserved; finite overflow yields
    /// signed infinity, and underflow rounds into subnormals or signed zero.
    /// NaNs remain NaN. Their payload and sign follow Rust's native conversion
    /// contract and are not guaranteed to agree between const and runtime use.
    ///
    /// This is the const construction form of the primitive and wrapper
    /// [`FloatElement::from_f64`] implementations. It performs the same native
    /// conversion rather than introducing a software rounding kernel.
    ///
    /// ```
    /// use eunomia::F32;
    ///
    /// const MIDPOINT: F32 = F32::from_float(f64::from_bits(0x3ff0_0000_1000_0000));
    /// const NEGATIVE_ZERO: F32 = F32::from_float(-0.0);
    /// assert_eq!(MIDPOINT.0.to_bits(), 1.0_f32.to_bits());
    /// assert_eq!(NEGATIVE_ZERO.0.to_bits(), 0x8000_0000);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_float(value: f64) -> Self {
        Self(value as f32)
    }

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

#[cfg(test)]
mod native_float_tests {
    use super::F32;
    use crate::FloatElement;

    // Adjacent binary64 encodings straddle each exact binary32 midpoint.
    const BOUNDARIES: &[(u64, u32)] = &[
        (0, 0),
        (1, 0),
        (0x0010_0000_0000_0000, 0),
        (0x368f_ffff_ffff_ffff, 0),
        (0x3690_0000_0000_0000, 0), // 2^-150 ties to even zero.
        (0x3690_0000_0000_0001, 1),
        (0x36a0_0000_0000_0000, 1), // Smallest binary32 subnormal.
        (0x380f_ffff_dfff_ffff, 0x007f_ffff),
        (0x380f_ffff_e000_0000, 0x0080_0000), // Tie to smallest normal.
        (0x380f_ffff_e000_0001, 0x0080_0000),
        (0x3810_0000_0000_0000, 0x0080_0000),
        (0x3ff0_0000_0fff_ffff, 0x3f80_0000),
        (0x3ff0_0000_1000_0000, 0x3f80_0000), // Even lower neighbour.
        (0x3ff0_0000_1000_0001, 0x3f80_0001),
        (0x3ff0_0000_2fff_ffff, 0x3f80_0001),
        (0x3ff0_0000_3000_0000, 0x3f80_0002), // Even upper neighbour.
        (0x3ff0_0000_3000_0001, 0x3f80_0002),
        (0x47ef_ffff_e000_0000, 0x7f7f_ffff),
        (0x47ef_ffff_efff_ffff, 0x7f7f_ffff),
        (0x47ef_ffff_f000_0000, 0x7f80_0000), // Overflow midpoint.
        (0x47ef_ffff_f000_0001, 0x7f80_0000),
        (0x7fef_ffff_ffff_ffff, 0x7f80_0000),
        (0x7ff0_0000_0000_0000, 0x7f80_0000),
    ];

    const fn boundary_values() -> [[F32; 2]; BOUNDARIES.len()] {
        let mut output = [[F32(0.0); 2]; BOUNDARIES.len()];
        let mut index = 0;
        while index < BOUNDARIES.len() {
            output[index][0] = F32::from_float(f64::from_bits(BOUNDARIES[index].0));
            output[index][1] = F32::from_float(f64::from_bits(BOUNDARIES[index].0 | (1 << 63)));
            index += 1;
        }
        output
    }

    const CONSTANT_BOUNDARIES: [[F32; 2]; BOUNDARIES.len()] = boundary_values();

    // Runtime native cast is the previous primitive/wrapper conversion body.
    // Keep it independently callable rather than constant-folding the oracle.
    #[inline(never)]
    fn previous_runtime_conversion(value: f64) -> f32 {
        core::hint::black_box(value) as f32
    }

    #[test]
    fn constant_narrowing_preserves_analytic_boundaries() {
        for (values, (bits, expected)) in CONSTANT_BOUNDARIES.iter().zip(BOUNDARIES) {
            for (sign, value) in values.iter().enumerate() {
                let expected = expected | if sign == 0 { 0 } else { 1 << 31 };
                let input = f64::from_bits(bits | if sign == 0 { 0 } else { 1 << 63 });
                assert_eq!(value.0.to_bits(), expected);
                assert_eq!(previous_runtime_conversion(input).to_bits(), expected);
                assert_eq!(<f32 as FloatElement>::from_f64(input).to_bits(), expected);
                assert_eq!(<F32 as FloatElement>::from_f64(input).0.to_bits(), expected);
            }
        }
    }

    const MANTISSAS: [u64; 12] = [
        0,
        1,
        (1 << 28) - 1,
        1 << 28,
        (1 << 28) + 1,
        (1 << 29) - 1,
        1 << 29,
        (1 << 29) + 1,
        (1 << 51) - 1,
        1 << 51,
        (1 << 51) + 1,
        (1 << 52) - 1,
    ];
    const GRID_LEN: usize = 2 * 2048 * MANTISSAS.len();

    // This large grid exists only in static storage, never a runtime stack.
    static CONSTANT_GRID: [(u64, F32); GRID_LEN] = {
        let mut output = [(0, F32(0.0)); GRID_LEN];
        let mut index = 0;
        let mut exponent = 0_u64;
        while exponent < 2048 {
            let mut sign = 0_u64;
            while sign < 2 {
                let mut fraction = 0;
                while fraction < MANTISSAS.len() {
                    let bits = (sign << 63) | (exponent << 52) | MANTISSAS[fraction];
                    output[index] = (bits, F32::from_float(f64::from_bits(bits)));
                    index += 1;
                    fraction += 1;
                }
                sign += 1;
            }
            exponent += 1;
        }
        output
    };

    #[test]
    fn constant_narrowing_matches_runtime_across_every_exponent_and_sign() {
        for &(bits, value) in &CONSTANT_GRID {
            let input = f64::from_bits(bits);
            let expected = previous_runtime_conversion(input);
            let primitive = <f32 as FloatElement>::from_f64(input);
            let wrapper = <F32 as FloatElement>::from_f64(input).0;
            if expected.is_nan() {
                // Rust does not promise a payload/sign across narrowing or CTFE.
                assert!(value.0.is_nan());
                assert!(primitive.is_nan());
                assert!(wrapper.is_nan());
            } else {
                assert_eq!(value.0.to_bits(), expected.to_bits(), "{bits:#018x}");
                assert_eq!(primitive.to_bits(), expected.to_bits(), "{bits:#018x}");
                assert_eq!(wrapper.to_bits(), expected.to_bits(), "{bits:#018x}");
            }
        }
    }
}
