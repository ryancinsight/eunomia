//! Additive and multiplicative identity constants for [`Complex`].
//!
//! These mirror `num_complex::Complex`'s `Zero`/`One` so generic code that
//! needs the complex zero (buffer initialisation, accumulator seeds) or one can
//! use a `const` instead of a trait-method call.

use super::Complex;
use crate::traits::NumericElement;

impl<T: NumericElement> Complex<T> {
    /// The additive identity `0 + 0i`.
    pub const ZERO: Self = Self {
        re: <T as NumericElement>::ZERO,
        im: <T as NumericElement>::ZERO,
    };

    /// The multiplicative identity `1 + 0i`.
    pub const ONE: Self = Self {
        re: <T as NumericElement>::ONE,
        im: <T as NumericElement>::ZERO,
    };
}

impl<T> num_traits::Zero for Complex<T>
where
    T: NumericElement + num_traits::Zero + core::ops::Add<Output = T> + Copy,
{
    #[inline(always)]
    fn zero() -> Self {
        Self {
            re: T::zero(),
            im: T::zero(),
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }
}

impl<T> num_traits::One for Complex<T>
where
    T: NumericElement
        + num_traits::One
        + num_traits::Zero
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Mul<Output = T>
        + Copy,
{
    #[inline(always)]
    fn one() -> Self {
        Self {
            re: T::one(),
            im: T::zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Complex;

    #[test]
    fn identity_constants() {
        assert_eq!(Complex::<f64>::ZERO, Complex::new(0.0, 0.0));
        assert_eq!(Complex::<f32>::ONE, Complex::new(1.0, 0.0));
        // additive / multiplicative identity behaviour
        let z = Complex::new(3.0_f64, -2.0);
        assert_eq!(z + Complex::ZERO, z);
        assert_eq!(z * Complex::ONE, z);
    }

    #[test]
    fn num_traits_zero_matches_const() {
        use num_traits::Zero;
        assert_eq!(Complex::<f64>::zero(), Complex::<f64>::ZERO);
        assert!(Complex::<f64>::zero().is_zero());
        assert!(!Complex::new(1.0_f64, 0.0).is_zero());
        assert!(!Complex::new(0.0_f64, 1.0).is_zero());
    }

    #[test]
    fn num_traits_one_matches_const() {
        use num_traits::One;
        assert_eq!(Complex::<f32>::one(), Complex::<f32>::ONE);
        let z = Complex::new(5.0_f32, -3.0);
        assert_eq!(z * Complex::one(), z);
    }
}
