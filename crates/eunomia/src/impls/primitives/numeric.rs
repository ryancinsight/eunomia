//! `NumericElement` impls for primitive floats and signed/unsigned integers.

use crate::traits::{private, CastFrom, NumericElement};
use crate::types::Complex;

impl NumericElement for f32 {
    const ZERO: Self = 0.0_f32;
    const ONE: Self = 1.0_f32;
    const NAN: Self = f32::NAN;
    const INFINITY: Self = f32::INFINITY;
    const BYTE_WIDTH: usize = 4;
    const ALL_ONES: Self = f32::from_bits(0xFFFF_FFFF);
    const SIGN_MASK: Self = f32::from_bits(0x8000_0000);
    const MIN_VALUE: Self = f32::NEG_INFINITY;
    const MAX_VALUE: Self = f32::INFINITY;

    #[inline(always)]
    fn abs(self) -> Self {
        #[cfg(feature = "std")]
        {
            self.abs()
        }
        #[cfg(not(feature = "std"))]
        {
            f32::from_bits(self.to_bits() & 0x7FFF_FFFF)
        }
    }
    #[inline(always)]
    fn scalar_fmadd(self, b: Self, c: Self) -> Self {
        #[cfg(feature = "std")]
        {
            self.mul_add(b, c)
        }
        #[cfg(not(feature = "std"))]
        {
            libm::fmaf(self, b, c)
        }
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")]
        {
            self.sqrt()
        }
        #[cfg(not(feature = "std"))]
        {
            libm::sqrtf(self)
        }
    }
    #[inline(always)]
    fn is_finite(self) -> bool {
        self.is_finite()
    }
    #[inline(always)]
    fn is_nan(self) -> bool {
        self.is_nan()
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self::from_bits(self.to_bits() & rhs.to_bits())
    }
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self::from_bits(self.to_bits() | rhs.to_bits())
    }
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self::from_bits(self.to_bits() ^ rhs.to_bits())
    }
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.to_bits().count_ones()
    }
    /// Use native `f32::min` which correctly handles NaN propagation.
    #[inline(always)]
    fn min_scalar(self, other: Self) -> Self {
        self.min(other)
    }
    /// Use native `f32::max` which correctly handles NaN propagation.
    #[inline(always)]
    fn max_scalar(self, other: Self) -> Self {
        self.max(other)
    }
}

impl NumericElement for f64 {
    const ZERO: Self = 0.0_f64;
    const ONE: Self = 1.0_f64;
    const NAN: Self = f64::NAN;
    const INFINITY: Self = f64::INFINITY;
    const BYTE_WIDTH: usize = 8;
    const ALL_ONES: Self = f64::from_bits(0xFFFF_FFFF_FFFF_FFFF);
    const SIGN_MASK: Self = f64::from_bits(0x8000_0000_0000_0000);
    const MIN_VALUE: Self = f64::NEG_INFINITY;
    const MAX_VALUE: Self = f64::INFINITY;

    #[inline(always)]
    fn abs(self) -> Self {
        #[cfg(feature = "std")]
        {
            self.abs()
        }
        #[cfg(not(feature = "std"))]
        {
            f64::from_bits(self.to_bits() & 0x7FFF_FFFF_FFFF_FFFF)
        }
    }
    #[inline(always)]
    fn scalar_fmadd(self, b: Self, c: Self) -> Self {
        #[cfg(feature = "std")]
        {
            self.mul_add(b, c)
        }
        #[cfg(not(feature = "std"))]
        {
            libm::fma(self, b, c)
        }
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")]
        {
            self.sqrt()
        }
        #[cfg(not(feature = "std"))]
        {
            libm::sqrt(self)
        }
    }
    #[inline(always)]
    fn is_finite(self) -> bool {
        self.is_finite()
    }
    #[inline(always)]
    fn is_nan(self) -> bool {
        self.is_nan()
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self
    }
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self::from_bits(self.to_bits() & rhs.to_bits())
    }
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self::from_bits(self.to_bits() | rhs.to_bits())
    }
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self::from_bits(self.to_bits() ^ rhs.to_bits())
    }
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.to_bits().count_ones()
    }
    /// Use native `f64::min` which correctly handles NaN propagation.
    #[inline(always)]
    fn min_scalar(self, other: Self) -> Self {
        self.min(other)
    }
    /// Use native `f64::max` which correctly handles NaN propagation.
    #[inline(always)]
    fn max_scalar(self, other: Self) -> Self {
        self.max(other)
    }
}

/// Shared `NumericElement` body for the built-in signed integer types. Differs
/// from [`impl_numeric_element_unsigned`] only in `ALL_ONES` (-1), `SIGN_MASK`
/// (`T::MIN`), and `abs`. `min_scalar`/`max_scalar` use the `PartialOrd`-based
/// trait defaults.
macro_rules! impl_numeric_element_signed {
    ($t:ty, $byte_width:expr) => {
        impl NumericElement for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const NAN: Self = 0;
            const INFINITY: Self = 0;
            const BYTE_WIDTH: usize = $byte_width;
            const ALL_ONES: Self = -1;
            const SIGN_MASK: Self = <$t>::MIN;
            const MIN_VALUE: Self = <$t>::MIN;
            const MAX_VALUE: Self = <$t>::MAX;

            #[inline(always)]
            fn abs(self) -> Self {
                self.abs()
            }
            #[inline(always)]
            fn scalar_fmadd(self, b: Self, c: Self) -> Self {
                self.wrapping_mul(b).wrapping_add(c)
            }
            #[inline(always)]
            fn sqrt(self) -> Self {
                // Exact integer (floor) square root. The previous
                // `(self as f64).sqrt() as Self` rounded operands above 2^53 to f64
                // *before* taking the root, losing precision (e.g. `i64::MAX`).
                // `isqrt` is exact. Negative inputs have no real root and integers
                // have no NaN to signal it, so they return 0 (documented contract).
                if self < 0 {
                    0
                } else {
                    self.isqrt()
                }
            }
            #[inline(always)]
            fn is_finite(self) -> bool {
                true
            }
            #[inline(always)]
            fn is_nan(self) -> bool {
                false
            }
            #[inline(always)]
            fn to_f64(self) -> f64 {
                self as f64
            }
            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                self & rhs
            }
            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                self | rhs
            }
            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                self ^ rhs
            }
            #[inline(always)]
            fn count_ones(self) -> u32 {
                self.count_ones()
            }
            #[inline(always)]
            fn saturating_add(self, rhs: Self) -> Self {
                self.saturating_add(rhs)
            }
            #[inline(always)]
            fn saturating_mul(self, rhs: Self) -> Self {
                self.saturating_mul(rhs)
            }
            #[inline(always)]
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }
            #[inline(always)]
            fn checked_mul(self, rhs: Self) -> Option<Self> {
                self.checked_mul(rhs)
            }
        }
    };
}

impl_numeric_element_signed!(i8, 1);
impl_numeric_element_signed!(i16, 2);
impl_numeric_element_signed!(i32, 4);
impl_numeric_element_signed!(i64, 8);
impl_numeric_element_signed!(isize, core::mem::size_of::<isize>());

impl<T> NumericElement for Complex<T>
where
    T: NumericElement + CastFrom<i32> + core::ops::Neg<Output = T>,
{
    const ZERO: Self = Self::new(<T as NumericElement>::ZERO, <T as NumericElement>::ZERO);
    const ONE: Self = Self::new(<T as NumericElement>::ONE, <T as NumericElement>::ZERO);
    const NAN: Self = Self::new(<T as NumericElement>::NAN, <T as NumericElement>::ZERO);
    const INFINITY: Self = Self::new(<T as NumericElement>::INFINITY, <T as NumericElement>::ZERO);
    const BYTE_WIDTH: usize = 2 * <T as NumericElement>::BYTE_WIDTH;
    const ALL_ONES: Self = Self::new(
        <T as NumericElement>::ALL_ONES,
        <T as NumericElement>::ALL_ONES,
    );
    const SIGN_MASK: Self = Self::new(
        <T as NumericElement>::SIGN_MASK,
        <T as NumericElement>::ZERO,
    );
    const MIN_VALUE: Self = Self::new(
        <T as NumericElement>::MIN_VALUE,
        <T as NumericElement>::ZERO,
    );
    const MAX_VALUE: Self = Self::new(
        <T as NumericElement>::MAX_VALUE,
        <T as NumericElement>::ZERO,
    );

    #[inline(always)]
    fn abs(self) -> Self {
        Self::new(
            (self.re * self.re + self.im * self.im).sqrt(),
            <T as NumericElement>::ZERO,
        )
    }

    #[inline(always)]
    fn scalar_fmadd(self, b: Self, c: Self) -> Self {
        self * b + c
    }

    #[inline(always)]
    fn sqrt(self) -> Self {
        let mag2 = self.re * self.re + self.im * self.im;
        let half =
            <T as NumericElement>::ONE / (<T as NumericElement>::ONE + <T as NumericElement>::ONE);
        let u = ((mag2 + self.re) * half).sqrt();
        let mut v = ((mag2 - self.re) * half).sqrt();
        if self.im < <T as NumericElement>::ZERO {
            v = -v;
        }
        Self::new(u, v)
    }

    #[inline(always)]
    fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    #[inline(always)]
    fn is_nan(self) -> bool {
        self.re.is_nan() || self.im.is_nan()
    }

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self.re.to_f64()
    }

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.re.bitand(rhs.re), self.im.bitand(rhs.im))
    }

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.re.bitor(rhs.re), self.im.bitor(rhs.im))
    }

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.re.bitxor(rhs.re), self.im.bitxor(rhs.im))
    }

    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.re.count_ones() + self.im.count_ones()
    }

    #[inline(always)]
    fn min_scalar(self, other: Self) -> Self {
        if self.re < other.re || (self.re == other.re && self.im <= other.im) {
            self
        } else {
            other
        }
    }

    #[inline(always)]
    fn max_scalar(self, other: Self) -> Self {
        if self.re > other.re || (self.re == other.re && self.im >= other.im) {
            self
        } else {
            other
        }
    }
}

impl<T> CastFrom<i32> for Complex<T>
where
    T: NumericElement,
{
    #[inline(always)]
    fn cast_from(val: i32) -> Self {
        Self::new(T::cast_from(val), <T as NumericElement>::ZERO)
    }
}

impl<T> private::Sealed for Complex<T> where T: private::Sealed {}
