//! `FloatElement` impls for the primitive float types (libm-backed,
//! native-precision transcendentals).

use crate::traits::{FloatElement, NumericElement};

impl FloatElement for f32 {
    #[inline(always)]
    fn from_f32(val: f32) -> Self {
        val
    }
    #[inline(always)]
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }
}

impl FloatElement for f64 {
    #[inline(always)]
    fn from_f32(val: f32) -> Self {
        val as f64
    }
    #[inline(always)]
    fn from_f64(val: f64) -> Self {
        val
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    // Native double-precision transcendentals (override the f32-routed defaults).
    #[inline]
    fn exp(self) -> Self {
        libm::exp(self)
    }
    #[inline]
    fn ln(self) -> Self {
        libm::log(self)
    }
    #[inline]
    fn sin(self) -> Self {
        libm::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        libm::cos(self)
    }
    #[inline]
    fn acos(self) -> Self {
        libm::acos(self)
    }
    #[inline]
    fn tan(self) -> Self {
        libm::tan(self)
    }
    #[inline]
    fn sinh(self) -> Self {
        libm::sinh(self)
    }
    #[inline]
    fn cosh(self) -> Self {
        libm::cosh(self)
    }
    #[inline]
    fn tanh(self) -> Self {
        libm::tanh(self)
    }
    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2(self, other)
    }
    #[inline]
    fn powf(self, n: Self) -> Self {
        libm::pow(self, n)
    }
    #[inline]
    fn recip(self) -> Self {
        1.0 / self
    }
    #[inline]
    fn floor(self) -> Self {
        libm::floor(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        libm::ceil(self)
    }
    #[inline]
    fn round(self) -> Self {
        libm::round(self)
    }
    #[inline]
    fn trunc(self) -> Self {
        libm::trunc(self)
    }
    #[inline]
    fn signum(self) -> Self {
        if self.is_nan() {
            self
        } else {
            libm::copysign(1.0, self)
        }
    }
    // Native double-precision special functions (override the f32-routed defaults).
    #[inline]
    fn erf(self) -> Self {
        libm::erf(self)
    }
    #[inline]
    fn erfc(self) -> Self {
        libm::erfc(self)
    }
    #[inline]
    fn lgamma(self) -> Self {
        libm::lgamma(self)
    }
}

macro_rules! impl_numeric_element_unsigned {
    ($t:ty, $byte_width:expr, $min_value:expr, $max_value:expr) => {
        impl NumericElement for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const NAN: Self = 0;
            const INFINITY: Self = 0;
            const BYTE_WIDTH: usize = $byte_width;
            const ALL_ONES: Self = !0;
            const SIGN_MASK: Self = 0;
            const MIN_VALUE: Self = $min_value;
            const MAX_VALUE: Self = $max_value;

            #[inline(always)]
            fn abs(self) -> Self {
                self
            }
            #[inline(always)]
            fn scalar_fmadd(self, b: Self, c: Self) -> Self {
                self.wrapping_mul(b).wrapping_add(c)
            }
            #[inline(always)]
            fn sqrt(self) -> Self {
                // Exact integer (floor) square root; no f64 round-trip (the old
                // `(self as f64).sqrt() as Self` lost precision above 2^53, e.g.
                // `u64::MAX`).
                self.isqrt()
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
            /// Native saturating_add replaces the trait's float default `self + rhs`,
            /// which would wrap in release / panic in debug on uint overflow. The
            /// native op saturates to `MAX_VALUE`/`MIN_VALUE`.
            #[inline(always)]
            fn saturating_add(self, rhs: Self) -> Self {
                self.saturating_add(rhs)
            }
            /// Native saturating_mul; see [`saturating_add`] for rationale.
            #[inline(always)]
            fn saturating_mul(self, rhs: Self) -> Self {
                self.saturating_mul(rhs)
            }
            /// Native checked_add returns `None` on uint overflow instead of
            /// silently wrapping (the trait float default returns `Some(self + rhs)`,
            /// which is wrong for integers).
            #[inline(always)]
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }
            /// Native checked_mul; see [`checked_add`] for rationale.
            #[inline(always)]
            fn checked_mul(self, rhs: Self) -> Option<Self> {
                self.checked_mul(rhs)
            }
        }
    };
}

impl_numeric_element_unsigned!(u8, 1, u8::MIN, u8::MAX);
impl_numeric_element_unsigned!(u16, 2, u16::MIN, u16::MAX);
impl_numeric_element_unsigned!(u32, 4, u32::MIN, u32::MAX);
impl_numeric_element_unsigned!(u64, 8, u64::MIN, u64::MAX);
impl_numeric_element_unsigned!(usize, core::mem::size_of::<usize>(), usize::MIN, usize::MAX);
