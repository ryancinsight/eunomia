//! `NumericElement` impls for primitive floats and signed/unsigned integers.

use crate::traits::{private, CastFrom, NumericElement};
use crate::types::Complex;

/// Shared `NumericElement` body for the primitive float types. The two widths
/// differ only in their constants, the `no_std` `abs` bit mask, the `libm` fused
/// multiply-add / square-root entry points, and the `to_f64` widening (identity
/// for `f64`); the bitwise ops, `count_ones`, and the native `min`/`max`
/// overrides are width-independent.
macro_rules! impl_numeric_element_float {
    (
        $t:ty,
        $byte_width:expr,
        $all_ones:expr,
        $sign_mask:expr,
        $abs_mask:expr,
        $fma:path,
        $sqrt:path,
        $to_f64:expr
    ) => {
        impl NumericElement for $t {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const NAN: Self = <$t>::NAN;
            const INFINITY: Self = <$t>::INFINITY;
            const BYTE_WIDTH: usize = $byte_width;
            const ALL_ONES: Self = $all_ones;
            const SIGN_MASK: Self = $sign_mask;
            const MIN_VALUE: Self = <$t>::NEG_INFINITY;
            const MAX_VALUE: Self = <$t>::INFINITY;

            #[inline(always)]
            fn abs(self) -> Self {
                #[cfg(feature = "std")]
                {
                    self.abs()
                }
                #[cfg(not(feature = "std"))]
                {
                    <$t>::from_bits(self.to_bits() & $abs_mask)
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
                    $fma(self, b, c)
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
                    $sqrt(self)
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
                $to_f64(self)
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
            /// Use the native float `min`, which matches the shared NaN and
            /// signed-zero contract.
            #[inline(always)]
            fn min_scalar(self, other: Self) -> Self {
                self.min(other)
            }
            /// Use the native float `max`, which matches the shared NaN and
            /// signed-zero contract.
            #[inline(always)]
            fn max_scalar(self, other: Self) -> Self {
                self.max(other)
            }
        }
    };
}

impl_numeric_element_float!(
    f32,
    4,
    f32::from_bits(0xFFFF_FFFF),
    f32::from_bits(0x8000_0000),
    0x7FFF_FFFF,
    libm::fmaf,
    libm::sqrtf,
    |x: f32| x as f64
);
impl_numeric_element_float!(
    f64,
    8,
    f64::from_bits(0xFFFF_FFFF_FFFF_FFFF),
    f64::from_bits(0x8000_0000_0000_0000),
    0x7FFF_FFFF_FFFF_FFFF,
    libm::fma,
    libm::sqrt,
    |x: f64| x
);

/// Shared `NumericElement` body for the built-in signed integer types. Differs
/// from [`impl_numeric_element_unsigned`] only in `ALL_ONES` (-1), `SIGN_MASK`
/// (`T::MIN`), and `abs`. `min_scalar`/`max_scalar` use the special-value-aware
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
            /// Native `wrapping_add` replaces the trait's float default
            /// `self + rhs`, which has no wraparound semantics for integers.
            #[inline(always)]
            fn wrapping_add(self, rhs: Self) -> Self {
                self.wrapping_add(rhs)
            }
            /// Native `wrapping_sub`; see [`Self::wrapping_add`] for rationale.
            #[inline(always)]
            fn wrapping_sub(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }
            /// Native `wrapping_mul`; see [`Self::wrapping_add`] for rationale.
            #[inline(always)]
            fn wrapping_mul(self, rhs: Self) -> Self {
                self.wrapping_mul(rhs)
            }
            /// Native `checked_div` returns `None` on a zero divisor or on
            /// `MIN / -1` (the one signed division whose mathematical result
            /// overflows the type), instead of the trait float default
            /// `Some(self / rhs)`, which would panic on either.
            #[inline(always)]
            fn checked_div(self, rhs: Self) -> Option<Self> {
                self.checked_div(rhs)
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

    /// Magnitude `√(re² + im²)` computed without squaring into overflow.
    ///
    /// The naive `(re² + im²).sqrt()` overflowed to `∞` for any component above
    /// `√MAX` even when the magnitude itself is representable; the scaled form
    /// `hi·√(1 + (lo/hi)²)` overflows only when the magnitude is. Measured on
    /// f64, `Complex::new(1e200, 1e200).abs()` was `∞` against the true
    /// `√2·1e200 ≈ 1.414e200`.
    #[inline(always)]
    fn abs(self) -> Self {
        let (re, im) = (self.re.abs(), self.im.abs());
        let (hi, lo) = if re >= im { (re, im) } else { (im, re) };
        if hi == <T as NumericElement>::ZERO {
            return Self::new(<T as NumericElement>::ZERO, <T as NumericElement>::ZERO);
        }
        let r = lo / hi;
        Self::new(
            hi * (<T as NumericElement>::ONE + r * r).sqrt(),
            <T as NumericElement>::ZERO,
        )
    }

    #[inline(always)]
    fn scalar_fmadd(self, b: Self, c: Self) -> Self {
        self * b + c
    }

    /// Principal complex square root.
    ///
    /// `√(re + im·i) = √((|z| + re)/2) ± √((|z| − re)/2)·i`, with the sign of
    /// the imaginary part carried through. The identity is stated in terms of
    /// the magnitude `|z|`, not its square: the previous body substituted
    /// `re² + im²` for `|z|`, so it returned a value of magnitude `|z|` rather
    /// than `√|z|`. Measured on f64, `NumericElement::sqrt(3 + 4i)` returned
    /// `≈ 3.7417 + 3.3166i` (magnitude 5) instead of the correct `2 + i`; the
    /// equivalent `Complex::<f64>::sqrt` surface was already correct.
    #[inline(always)]
    fn sqrt(self) -> Self {
        let magnitude = <Self as NumericElement>::abs(self).re;
        let half =
            <T as NumericElement>::ONE / (<T as NumericElement>::ONE + <T as NumericElement>::ONE);
        let u = (magnitude * half + self.re * half).sqrt();
        let v_squared = magnitude * half - self.re * half;
        // `|z| ≥ re` for every input, so `v_squared` is non-negative in exact
        // arithmetic; clamping a rounding-induced negative keeps it from
        // producing a spurious `NaN`.
        let v = if v_squared < <T as NumericElement>::ZERO {
            <T as NumericElement>::ZERO
        } else {
            v_squared.sqrt()
        };
        if self.im < <T as NumericElement>::ZERO {
            Self::new(u, -v)
        } else {
            Self::new(u, v)
        }
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
