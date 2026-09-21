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

                #[inline]
                fn divide_by_f64(self, factor: f64) -> Self {
                    self / <Self as FloatElement>::from_f64(factor)
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

    #[inline]
    fn divide_by_f64(self, factor: f64) -> Self {
        let factor = <f32 as FloatElement>::from_f64(factor);
        Self::new(self.re / factor, self.im / factor)
    }
}

impl UnitScalar for Complex64 {
    #[inline]
    fn scale_by_f64(self, factor: f64) -> Self {
        self.scale(factor)
    }

    #[inline]
    fn divide_by_f64(self, factor: f64) -> Self {
        Self::new(self.re / factor, self.im / factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn division_matches_native<T: FloatElement + UnitScalar>() {
        for value in [T::ZERO, T::ONE, T::from_f64(2.0)] {
            assert_eq!(value.divide_by_f64(2.0), value / T::from_f64(2.0));
        }
    }

    #[test]
    fn unit_division_covers_shipped_real_storage() {
        division_matches_native::<f32>();
        division_matches_native::<f64>();
        division_matches_native::<F16>();
        division_matches_native::<F32>();
        division_matches_native::<F64>();
        division_matches_native::<F4>();
        division_matches_native::<F8>();
        division_matches_native::<Bf4>();
        division_matches_native::<Bf8>();
        division_matches_native::<Bf16>();
    }

    #[test]
    fn complex_unit_scaling_preserves_quadrature() {
        let value = Complex64::new(2.0, -3.0);
        let scaled = value.scale_by_f64(0.5);
        assert_eq!(scaled, Complex64::new(1.0, -1.5));
    }

    #[test]
    fn real_unit_division_avoids_reciprocal_overflow() {
        let coefficient = 1.0e-40_f32;
        assert_eq!(
            coefficient.divide_by_f64(f64::from(coefficient)).to_bits(),
            1.0_f32.to_bits()
        );
        assert_eq!(
            0.0_f32.divide_by_f64(f64::from(coefficient)).to_bits(),
            0.0_f32.to_bits()
        );

        let coefficient = 1.0e-315_f64;
        assert_eq!(
            coefficient.divide_by_f64(coefficient).to_bits(),
            1.0_f64.to_bits()
        );
        assert_eq!(
            0.0_f64.divide_by_f64(coefficient).to_bits(),
            0.0_f64.to_bits()
        );
    }

    #[test]
    fn complex_unit_division_is_componentwise() {
        let coefficient = 1.0e-40_f32;
        let quotient =
            Complex32::new(coefficient, -coefficient).divide_by_f64(f64::from(coefficient));
        assert_eq!(quotient.re.to_bits(), 1.0_f32.to_bits());
        assert_eq!(quotient.im.to_bits(), (-1.0_f32).to_bits());
        let zero = Complex32::new(0.0, 0.0).divide_by_f64(f64::from(coefficient));
        assert_eq!(zero.re.to_bits(), 0.0_f32.to_bits());
        assert_eq!(zero.im.to_bits(), 0.0_f32.to_bits());

        let coefficient = 1.0e-315_f64;
        let quotient = Complex64::new(coefficient, -coefficient).divide_by_f64(coefficient);
        assert_eq!(quotient.re.to_bits(), 1.0_f64.to_bits());
        assert_eq!(quotient.im.to_bits(), (-1.0_f64).to_bits());
        let zero = Complex64::new(0.0, 0.0).divide_by_f64(coefficient);
        assert_eq!(zero.re.to_bits(), 0.0_f64.to_bits());
        assert_eq!(zero.im.to_bits(), 0.0_f64.to_bits());
    }
}
