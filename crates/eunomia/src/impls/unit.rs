//! [`UnitScalar`] implementations for Eunomia's shipped storage types.

use crate::traits::{FloatElement, UnitScalar};
use crate::types::{Bf16, Bf4, Bf8, Complex32, Complex64, F16, F32, F4, F64, F8};

macro_rules! impl_real_unit_scalar {
    ($($scalar:ty),+ $(,)?) => {
        $(
            impl UnitScalar for $scalar {
                #[inline]
                fn scale_by_f64(self, factor: f64) -> Self {
                    self * <Self as FloatElement>::from_f64(factor)
                }
            }
        )+
    };
}

impl_real_unit_scalar!(f32, f64, F16, F32, F64, F4, F8, Bf4, Bf8, Bf16);

impl UnitScalar for Complex32 {
    #[inline]
    fn scale_by_f64(self, factor: f64) -> Self {
        self.scale(<f32 as FloatElement>::from_f64(factor))
    }
}

impl UnitScalar for Complex64 {
    #[inline]
    fn scale_by_f64(self, factor: f64) -> Self {
        self.scale(factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_unit_scaling_preserves_quadrature() {
        let value = Complex64::new(2.0, -3.0);
        let scaled = value.scale_by_f64(0.5);
        assert_eq!(scaled, Complex64::new(1.0, -1.5));
    }
}
