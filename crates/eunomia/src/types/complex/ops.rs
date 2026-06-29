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
