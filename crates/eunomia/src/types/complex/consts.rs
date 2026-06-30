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
}
