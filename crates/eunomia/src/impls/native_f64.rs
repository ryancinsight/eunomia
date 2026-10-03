//! Shared native-`f64` `FloatElement` body, used by both the `f64` primitive
//! ([`impls::primitives::float`]) and the [`F64`](crate::F64) wrapper
//! ([`impls::wrappers::float`]).
//!
//! [`impls::primitives::float`]: crate::impls::primitives
//! [`impls::wrappers::float`]: crate::impls::wrappers

/// Emits the native-`f64` [`FloatElement`](crate::traits::FloatElement) body
/// once for both the `f64` primitive ([`impls::primitives::float`]) and the
/// [`F64`](crate::F64) wrapper ([`impls::wrappers::float`]). The two are a
/// verbatim transcription differing only by `.0` threading, and the wrapper's
/// copy exists solely because the f32-routed `impl_float_element!` default would
/// silently narrow it.
///
/// `$unwrap` projects the receiver to its inner `f64`; `$wrap` re-wraps an
/// `f64` result back into the target type.
///
/// [`impls::primitives::float`]: crate::impls::primitives
/// [`impls::wrappers::float`]: crate::impls::wrappers
macro_rules! impl_float_element_native_f64 {
    ($t:ty, $unwrap:expr, $wrap:expr) => {
        impl crate::traits::FloatElement for $t {
            // Identity: nothing in the crate is wider than `f64`, so the
            // accumulator is the type itself.
            type Accumulator = Self;

            #[inline(always)]
            fn from_f32(val: f32) -> Self {
                $wrap(val as f64)
            }
            #[inline(always)]
            fn from_f64(val: f64) -> Self {
                $wrap(val)
            }
            #[inline(always)]
            fn to_f32(self) -> f32 {
                $unwrap(self) as f32
            }
            #[inline]
            fn from_count(n: usize) -> Self {
                $wrap(crate::F64::from_count(n).0)
            }
            #[inline]
            fn from_integer(k: i64) -> Self {
                $wrap(crate::F64::from_integer(k).0)
            }
            #[inline]
            fn from_count_reciprocal(n: usize) -> Self {
                $wrap(crate::F64::from_count_reciprocal(n).0)
            }
            #[inline]
            fn binary_exponent(self) -> Option<i32> {
                let value = $unwrap(self);
                if value.is_finite() && value != 0.0 {
                    Some(libm::ilogb(value))
                } else {
                    None
                }
            }
            #[inline]
            fn scale_binary(self, exponent: i32) -> Self {
                $wrap(libm::scalbn($unwrap(self), exponent))
            }
            // Native double-precision transcendentals (the f32-routed defaults
            // would widen-narrow and discard f64 precision).
            #[inline]
            fn exp(self) -> Self {
                $wrap(libm::exp($unwrap(self)))
            }
            #[inline]
            fn exp2(self) -> Self {
                $wrap(libm::exp2($unwrap(self)))
            }
            #[inline]
            fn exp_m1(self) -> Self {
                $wrap(libm::expm1($unwrap(self)))
            }
            #[inline]
            fn ln(self) -> Self {
                $wrap(libm::log($unwrap(self)))
            }
            #[inline]
            fn ln_1p(self) -> Self {
                $wrap(libm::log1p($unwrap(self)))
            }
            #[inline]
            fn sin(self) -> Self {
                $wrap(libm::sin($unwrap(self)))
            }
            #[inline]
            fn asin(self) -> Self {
                $wrap(libm::asin($unwrap(self)))
            }
            #[inline]
            fn cos(self) -> Self {
                $wrap(libm::cos($unwrap(self)))
            }
            #[inline]
            fn acos(self) -> Self {
                $wrap(libm::acos($unwrap(self)))
            }
            #[inline]
            fn tan(self) -> Self {
                $wrap(libm::tan($unwrap(self)))
            }
            #[inline]
            fn atan(self) -> Self {
                $wrap(libm::atan($unwrap(self)))
            }
            #[inline]
            fn sinh(self) -> Self {
                $wrap(libm::sinh($unwrap(self)))
            }
            #[inline]
            fn asinh(self) -> Self {
                $wrap(libm::asinh($unwrap(self)))
            }
            #[inline]
            fn cosh(self) -> Self {
                $wrap(libm::cosh($unwrap(self)))
            }
            #[inline]
            fn acosh(self) -> Self {
                $wrap(libm::acosh($unwrap(self)))
            }
            #[inline]
            fn tanh(self) -> Self {
                $wrap(libm::tanh($unwrap(self)))
            }
            #[inline]
            fn atanh(self) -> Self {
                $wrap(libm::atanh($unwrap(self)))
            }
            #[inline]
            fn atan2(self, other: Self) -> Self {
                $wrap(libm::atan2($unwrap(self), $unwrap(other)))
            }
            #[inline]
            fn powf(self, n: Self) -> Self {
                $wrap(libm::pow($unwrap(self), $unwrap(n)))
            }
            #[inline]
            fn cbrt(self) -> Self {
                $wrap(libm::cbrt($unwrap(self)))
            }
            #[inline]
            fn recip(self) -> Self {
                $wrap(1.0 / $unwrap(self))
            }
            #[inline]
            fn nth_root(self, n: u32) -> Self {
                if n == 0 {
                    return $wrap(f64::NAN);
                }
                let value = $unwrap(self);
                let root = if n % 2 == 1 {
                    libm::copysign(libm::pow(libm::fabs(value), 1.0 / n as f64), value)
                } else {
                    libm::pow(value, 1.0 / n as f64)
                };
                $wrap(root)
            }
            #[inline]
            fn floor(self) -> Self {
                $wrap(libm::floor($unwrap(self)))
            }
            #[inline]
            fn ceil(self) -> Self {
                $wrap(libm::ceil($unwrap(self)))
            }
            #[inline]
            fn round(self) -> Self {
                $wrap(libm::round($unwrap(self)))
            }
            #[inline]
            fn round_ties_even(self) -> Self {
                $wrap(libm::roundeven($unwrap(self)))
            }
            #[inline]
            fn trunc(self) -> Self {
                $wrap(libm::trunc($unwrap(self)))
            }
            #[inline]
            fn signum(self) -> Self {
                let value = $unwrap(self);
                if value.is_nan() {
                    self
                } else {
                    $wrap(libm::copysign(1.0, value))
                }
            }
            // Native double-precision special functions.
            #[inline]
            fn log10(self) -> Self {
                $wrap(libm::log10($unwrap(self)))
            }
            #[inline]
            fn log2(self) -> Self {
                $wrap(libm::log2($unwrap(self)))
            }
            #[inline]
            fn erf(self) -> Self {
                $wrap(libm::erf($unwrap(self)))
            }
            #[inline]
            fn erfc(self) -> Self {
                $wrap(libm::erfc($unwrap(self)))
            }
            #[inline]
            fn lgamma(self) -> Self {
                $wrap(libm::lgamma($unwrap(self)))
            }
        }
    };
}

pub(crate) use impl_float_element_native_f64;
