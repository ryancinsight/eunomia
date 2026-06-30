//! Arithmetic operators for [`Complex`] (field-wise Add/Sub/Neg/Rem, the
//! complex product/quotient Mul/Div, and the order-less PartialOrd).

use super::Complex;
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }
}

impl<T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone> Mul for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        Self {
            re: self.re.clone() * other.re.clone() - self.im.clone() * other.im.clone(),
            im: self.re * other.im + self.im * other.re,
        }
    }
}

impl<T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Clone> Div
    for Complex<T>
{
    type Output = Self;
    #[inline(always)]
    fn div(self, other: Self) -> Self {
        let denom = other.re.clone() * other.re.clone() + other.im.clone() * other.im.clone();
        Self {
            re: (self.re.clone() * other.re.clone() + self.im.clone() * other.im.clone())
                / denom.clone(),
            im: (self.im * other.re.clone() - self.re * other.im) / denom,
        }
    }
}

impl<T: Neg<Output = T>> Neg for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// Reference-operand forwarding for the binary arithmetic operators, mirroring
/// `num_complex` (and `std`'s `forward_ref_binop!`): `&a · &b`, `a · &b`, and
/// `&a · b` forward to the by-value impl above. A declarative macro is used here
/// for the same reason `std` does — twelve otherwise-identical impls (4 ops × 3
/// reference shapes) would be pure duplication. Each forwards by cloning behind
/// the reference, so it is exactly the by-value cost on `Copy` scalars.
macro_rules! forward_ref_binop {
    ($imp:ident, $method:ident) => {
        impl<T> $imp<&Complex<T>> for &Complex<T>
        where
            Complex<T>: $imp<Output = Complex<T>> + Clone,
        {
            type Output = Complex<T>;
            #[inline(always)]
            fn $method(self, other: &Complex<T>) -> Complex<T> {
                (*self).clone().$method((*other).clone())
            }
        }

        impl<T> $imp<&Complex<T>> for Complex<T>
        where
            Complex<T>: $imp<Output = Complex<T>> + Clone,
        {
            type Output = Complex<T>;
            #[inline(always)]
            fn $method(self, other: &Complex<T>) -> Complex<T> {
                self.$method((*other).clone())
            }
        }

        impl<T> $imp<Complex<T>> for &Complex<T>
        where
            Complex<T>: $imp<Output = Complex<T>> + Clone,
        {
            type Output = Complex<T>;
            #[inline(always)]
            fn $method(self, other: Complex<T>) -> Complex<T> {
                (*self).clone().$method(other)
            }
        }
    };
}

forward_ref_binop!(Add, add);
forward_ref_binop!(Sub, sub);
forward_ref_binop!(Mul, mul);
forward_ref_binop!(Div, div);

/// Component-wise remainder.
///
/// Complex numbers have no canonical `%`; this applies `Rem` field-wise. It
/// exists so `Complex<T>` can satisfy generic scalar-element trait bounds that
/// require `Rem` (e.g. a numeric `Scalar` supertrait) where the operation is
/// never semantically exercised on complex values.
impl<T: Rem<Output = T>> Rem for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn rem(self, other: Self) -> Self {
        Self {
            re: self.re % other.re,
            im: self.im % other.im,
        }
    }
}

/// Complex numbers admit no total order. `partial_cmp` therefore always returns
/// `None`; the impl exists only so `Complex<T>` can satisfy a `PartialOrd`
/// trait bound required of generic scalar elements — it never claims an
/// ordering. Equality is the derived `PartialEq`.
impl<T: PartialEq> PartialOrd for Complex<T> {
    #[inline(always)]
    fn partial_cmp(&self, _other: &Self) -> Option<core::cmp::Ordering> {
        None
    }
}

#[cfg(test)]
// This test deliberately exercises the by-reference operator forms this module
// adds; `op_ref` would flag them as needless on `Copy` operands, which is the
// exact form under test.
#[allow(clippy::op_ref)]
mod tests {
    use super::Complex;

    #[test]
    fn reference_operators_match_by_value() {
        let a = Complex::new(2.0_f64, 3.0);
        let b = Complex::new(-1.0_f64, 4.0);
        assert_eq!(&a + &b, a + b);
        assert_eq!(&a - &b, a - b);
        assert_eq!(&a * &b, a * b);
        assert_eq!(&a / &b, a / b);
        // mixed reference/value shapes
        assert_eq!(a + &b, a + b);
        assert_eq!(&a * b, a * b);
    }
}
