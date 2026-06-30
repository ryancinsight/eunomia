//! Iterator reductions for [`Complex`] — `Sum` and `Product`, mirroring
//! `num_complex::Complex` so `iter.sum()`/`iter.product()` work over complex
//! values (FFT accumulation, polynomial evaluation).

use super::Complex;
use crate::traits::NumericElement;
use core::iter::{Product, Sum};

impl<T: NumericElement> Sum for Complex<T> {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, z| acc + z)
    }
}

impl<'a, T: NumericElement> Sum<&'a Complex<T>> for Complex<T> {
    #[inline]
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, z| acc + *z)
    }
}

impl<T: NumericElement> Product for Complex<T> {
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |acc, z| acc * z)
    }
}

impl<'a, T: NumericElement> Product<&'a Complex<T>> for Complex<T> {
    #[inline]
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |acc, z| acc * *z)
    }
}

#[cfg(test)]
mod tests {
    use super::Complex;

    #[test]
    fn sum_and_product_over_iterators() {
        let v = [
            Complex::new(1.0_f64, 1.0),
            Complex::new(2.0, -1.0),
            Complex::new(0.0, 3.0),
        ];
        assert_eq!(v.iter().copied().sum::<Complex<f64>>(), Complex::new(3.0, 3.0));
        assert_eq!(v.iter().sum::<Complex<f64>>(), Complex::new(3.0, 3.0));
        // product: (1+i)(2-i) = 3+i; (3+i)(3i) = -3+9i
        assert_eq!(v.iter().copied().product::<Complex<f64>>(), Complex::new(-3.0, 9.0));
    }
}
