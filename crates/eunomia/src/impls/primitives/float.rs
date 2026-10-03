//! `FloatElement` impls for the primitive float types (libm-backed,
//! native-precision transcendentals).

use crate::impls::native_f64::impl_float_element_native_f64;
use crate::traits::{FloatElement, NumericElement};
use crate::types::F32;

impl FloatElement for f32 {
    // Identity: `f32` accumulation already holds `ε₃₂ ≈ 1.2e-7`, and widening to
    // `f64` would cost bandwidth and SIMD lanes rather than fix a defect. A
    // caller wanting double-precision accumulation instantiates at `f64`.
    type Accumulator = Self;

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
    // The provided bodies round to odd for formats narrower than `f32`.
    #[inline]
    fn from_count(n: usize) -> Self {
        F32::from_count(n).0
    }
    #[inline]
    fn from_integer(k: i64) -> Self {
        F32::from_integer(k).0
    }
    #[inline]
    fn from_count_reciprocal(n: usize) -> Self {
        F32::from_count_reciprocal(n).0
    }
}

// The primitive `f64` and the `F64` wrapper are one native-`f64` body, emitted
// from `impl_float_element_native_f64!` (see `impls::native_f64`).
impl_float_element_native_f64!(f64, |x: f64| x, |v: f64| v);

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
            /// Native `saturating_add` replaces the trait's float default `self + rhs`,
            /// which would wrap in release / panic in debug on uint overflow. The
            /// native op saturates to `MAX_VALUE`/`MIN_VALUE`.
            #[inline(always)]
            fn saturating_add(self, rhs: Self) -> Self {
                self.saturating_add(rhs)
            }
            /// Native `saturating_mul`; see [`Self::saturating_add`] for rationale.
            #[inline(always)]
            fn saturating_mul(self, rhs: Self) -> Self {
                self.saturating_mul(rhs)
            }
            /// Native `checked_add` returns `None` on uint overflow instead of
            /// silently wrapping (the trait float default returns `Some(self + rhs)`,
            /// which is wrong for integers).
            #[inline(always)]
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }
            /// Native `checked_mul`; see [`Self::checked_add`] for rationale.
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
            /// Native `checked_div` returns `None` on division by zero
            /// instead of the trait float default `Some(self / rhs)`, which
            /// would panic for unsigned integers.
            #[inline(always)]
            fn checked_div(self, rhs: Self) -> Option<Self> {
                self.checked_div(rhs)
            }
        }
    };
}

impl_numeric_element_unsigned!(u8, 1, u8::MIN, u8::MAX);
impl_numeric_element_unsigned!(u16, 2, u16::MIN, u16::MAX);
impl_numeric_element_unsigned!(u32, 4, u32::MIN, u32::MAX);
impl_numeric_element_unsigned!(u64, 8, u64::MIN, u64::MAX);
impl_numeric_element_unsigned!(usize, core::mem::size_of::<usize>(), usize::MIN, usize::MAX);
