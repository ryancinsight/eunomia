//! `FloatElement` impls for the wrapper float types (native f64 via the
//! explicit F64 impl; reduced-precision types route through f32).

use crate::traits::FloatElement;
use crate::types::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8};

macro_rules! impl_float_element {
    ($t:ident, $from_f32:expr, $from_f64:expr, $to_f32:expr) => {
        impl FloatElement for $t {
            #[inline(always)]
            fn from_f32(val: f32) -> Self {
                $from_f32(val)
            }
            #[inline(always)]
            fn from_f64(val: f64) -> Self {
                $from_f64(val)
            }
            #[inline(always)]
            fn to_f32(self) -> f32 {
                $to_f32(self)
            }
        }
    };
}

impl_float_element!(F16, F16::from_f32, F16::from_f64, F16::to_f32);
impl_float_element!(F32, F32, |val| F32(val as f32), |x: F32| x.0);
// F64 wraps native `f64`, so it gets an explicit impl with native
// double-precision transcendentals — the macro's f32-routed default would
// widen-narrow and discard f64 precision. (F32 routes through f32 = native;
// F16/Bf16/F8/F4/Bf8/Bf4 have no hardware transcendentals, so the f32 default is
// their correct reduced-precision path.)
impl FloatElement for F64 {
    #[inline(always)]
    fn from_f32(val: f32) -> Self {
        F64(val as f64)
    }
    #[inline(always)]
    fn from_f64(val: f64) -> Self {
        F64(val)
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.0 as f32
    }
    #[inline]
    fn exp(self) -> Self {
        F64(libm::exp(self.0))
    }
    #[inline]
    fn ln(self) -> Self {
        F64(libm::log(self.0))
    }
    #[inline]
    fn sin(self) -> Self {
        F64(libm::sin(self.0))
    }
    #[inline]
    fn cos(self) -> Self {
        F64(libm::cos(self.0))
    }
    #[inline]
    fn acos(self) -> Self {
        F64(libm::acos(self.0))
    }
    #[inline]
    fn tan(self) -> Self {
        F64(libm::tan(self.0))
    }
    #[inline]
    fn sinh(self) -> Self {
        F64(libm::sinh(self.0))
    }
    #[inline]
    fn cosh(self) -> Self {
        F64(libm::cosh(self.0))
    }
    #[inline]
    fn tanh(self) -> Self {
        F64(libm::tanh(self.0))
    }
    #[inline]
    fn atan2(self, other: Self) -> Self {
        F64(libm::atan2(self.0, other.0))
    }
    #[inline]
    fn powf(self, n: Self) -> Self {
        F64(libm::pow(self.0, n.0))
    }
    #[inline]
    fn recip(self) -> Self {
        F64(1.0 / self.0)
    }
    #[inline]
    fn floor(self) -> Self {
        F64(libm::floor(self.0))
    }
    #[inline]
    fn ceil(self) -> Self {
        F64(libm::ceil(self.0))
    }
    #[inline]
    fn round(self) -> Self {
        F64(libm::round(self.0))
    }
    #[inline]
    fn trunc(self) -> Self {
        F64(libm::trunc(self.0))
    }
    #[inline]
    fn signum(self) -> Self {
        if self.0.is_nan() {
            self
        } else {
            F64(libm::copysign(1.0, self.0))
        }
    }
}
impl_float_element!(Bf16, Bf16::from_f32, Bf16::from_f64, Bf16::to_f32);
impl_float_element!(
    Bf8,
    Bf8::from_f32,
    |val| Bf8::from_f32(val as f32),
    |x: Bf8| x.to_f32()
);
impl_float_element!(
    Bf4,
    Bf4::from_f32,
    |val| Bf4::from_f32(val as f32),
    |x: Bf4| x.to_f32()
);
impl_float_element!(F8, F8::from_f32, |val| F8::from_f32(val as f32), |x: F8| x
    .to_f32());
impl_float_element!(F4, F4::from_f32, |val| F4::from_f32(val as f32), |x: F4| x
    .to_f32());

#[cfg(test)]
mod tests {
    use crate::traits::FloatElement;
    use crate::types::F64;

    #[test]
    fn f64_wrapper_transcendentals_are_native_precision() {
        // Native f64 agrees with std f64 to ~machine epsilon; the f32-routed
        // default would only be accurate to ~1e-7, failing these bounds.
        assert!((F64(1.0).exp().0 - core::f64::consts::E).abs() < 1e-15);
        assert!((F64(0.1).ln().0 - 0.1_f64.ln()).abs() < 1e-15);
        assert!((F64(0.7).sin().0 - 0.7_f64.sin()).abs() < 1e-15);
        assert!((F64(0.25).acos().0 - 0.25_f64.acos()).abs() < 1e-15);
        assert!((F64(2.0).powf(F64(10.0)).0 - 1024.0).abs() < 1e-12);
    }

    #[test]
    fn rounding_and_powi_surface() {
        // UFCS calls the FloatElement impl explicitly (concrete f64 would
        // otherwise resolve to std's inherent methods, not eunomia's).
        assert_eq!(FloatElement::floor(2.7_f64), 2.0);
        assert_eq!(FloatElement::ceil(2.2_f64), 3.0);
        assert_eq!(FloatElement::round(2.5_f64), 3.0);
        assert_eq!(FloatElement::trunc(-2.7_f64), -2.0);
        assert!((FloatElement::acos(0.25_f64) - 0.25_f64.acos()).abs() < 1e-15);
        // signum: ±1 with the sign (num_traits / std semantics), NaN→NaN.
        assert_eq!(FloatElement::signum(-3.5_f64), -1.0);
        assert_eq!(FloatElement::signum(0.0_f64), 1.0);
        // powi: exact integer power, correct for negative base + exponent.
        assert_eq!(FloatElement::powi(2.0_f64, 10), 1024.0);
        assert_eq!(FloatElement::powi(-2.0_f64, 3), -8.0);
        assert!((FloatElement::powi(2.0_f64, -2) - 0.25).abs() < 1e-15);
        // F64 wrapper (native impl).
        assert_eq!(FloatElement::floor(F64(2.7)).0, 2.0);
        assert_eq!(FloatElement::powi(F64(2.0), 10).0, 1024.0);
    }
}
