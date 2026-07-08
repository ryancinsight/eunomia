//! Arithmetic operators for [`Complex`] (field-wise Add/Sub/Neg/Rem, the
//! complex product/quotient Mul/Div, and the order-less PartialOrd).

use super::Complex;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub, SubAssign};

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

/// Multiply by a real scalar: `(re + im·i)·s = re·s + (im·s)·i`.
impl<T: Mul<Output = T> + Clone> Mul<T> for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: T) -> Self {
        Self {
            re: self.re * rhs.clone(),
            im: self.im * rhs,
        }
    }
}

/// Divide by a real scalar.
impl<T: Div<Output = T> + Clone> Div<T> for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: T) -> Self {
        Self {
            re: self.re / rhs.clone(),
            im: self.im / rhs,
        }
    }
}

/// Add a real scalar to the real part: `(re + im·i) + s = (re + s) + im·i`.
impl<T: Add<Output = T>> Add<T> for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: T) -> Self {
        Self {
            re: self.re + rhs,
            im: self.im,
        }
    }
}

/// Subtract a real scalar from the real part.
impl<T: Sub<Output = T>> Sub<T> for Complex<T> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: T) -> Self {
        Self {
            re: self.re - rhs,
            im: self.im,
        }
    }
}

/// Compound assignment by a real scalar (`*= s`, `/= s`, `+= s`, `-= s`),
/// forwarding to the by-value scalar operators above (num_complex parity).
impl<T: Mul<Output = T> + Clone + Copy> MulAssign<T> for Complex<T> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs;
    }
}

impl<T: Div<Output = T> + Clone + Copy> DivAssign<T> for Complex<T> {
    #[inline(always)]
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs;
    }
}

impl<T: Add<Output = T> + Copy> AddAssign<T> for Complex<T> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl<T: Sub<Output = T> + Copy> SubAssign<T> for Complex<T> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs;
    }
}

/// Compound-assignment operators, forwarding to the by-value binary impls (the
/// `num_complex::Complex` `*Assign` surface). Each is available exactly when its
/// by-value counterpart is.
impl<T> AddAssign for Complex<T>
where
    Complex<T>: Add<Output = Complex<T>> + Copy,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T> SubAssign for Complex<T>
where
    Complex<T>: Sub<Output = Complex<T>> + Copy,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T> MulAssign for Complex<T>
where
    Complex<T>: Mul<Output = Complex<T>> + Copy,
{
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T> DivAssign for Complex<T>
where
    Complex<T>: Div<Output = Complex<T>> + Copy,
{
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
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

macro_rules! scalar_left_ops {
    ($scalar:ty) => {
        impl Add<Complex<$scalar>> for $scalar {
            type Output = Complex<$scalar>;

            #[inline(always)]
            fn add(self, rhs: Complex<$scalar>) -> Self::Output {
                Complex::new(self + rhs.re, rhs.im)
            }
        }

        impl Sub<Complex<$scalar>> for $scalar {
            type Output = Complex<$scalar>;

            #[inline(always)]
            fn sub(self, rhs: Complex<$scalar>) -> Self::Output {
                Complex::new(self - rhs.re, -rhs.im)
            }
        }

        impl Mul<Complex<$scalar>> for $scalar {
            type Output = Complex<$scalar>;

            #[inline(always)]
            fn mul(self, rhs: Complex<$scalar>) -> Self::Output {
                Complex::new(self * rhs.re, self * rhs.im)
            }
        }

        impl Div<Complex<$scalar>> for $scalar {
            type Output = Complex<$scalar>;

            #[inline(always)]
            fn div(self, rhs: Complex<$scalar>) -> Self::Output {
                Complex::new(self, 0.0) / rhs
            }
        }
    };
}

scalar_left_ops!(f32);
scalar_left_ops!(f64);

#[cfg(test)]
// This test deliberately exercises the by-reference operator forms this module
// adds; `op_ref` would flag them as needless on `Copy` operands, which is the
// exact form under test.
#[allow(clippy::op_ref)]
mod tests {
    use super::Complex;

    #[test]
    fn scalar_real_ops_and_compound_assign() {
        // add/sub a real scalar (real part only)
        assert_eq!(Complex::new(2.0_f64, 3.0) + 5.0, Complex::new(7.0, 3.0));
        assert_eq!(Complex::new(2.0_f64, 3.0) - 1.0, Complex::new(1.0, 3.0));
        // scalar compound assignment
        let mut z = Complex::new(2.0_f64, -4.0);
        z *= 3.0;
        assert_eq!(z, Complex::new(6.0, -12.0));
        z /= 2.0;
        assert_eq!(z, Complex::new(3.0, -6.0));
        z += 1.0;
        assert_eq!(z, Complex::new(4.0, -6.0));
        z -= 4.0;
        assert_eq!(z, Complex::new(0.0, -6.0));
    }

    #[test]
    fn scalar_and_assign_operators() {
        // scalar multiply / divide
        let z = Complex::new(2.0_f64, -4.0);
        assert_eq!(z * 3.0, Complex::new(6.0, -12.0));
        assert_eq!(z / 2.0, Complex::new(1.0, -2.0));
        // compound assignment forwards to the binary op
        let mut a = Complex::new(1.0_f64, 1.0);
        a += Complex::new(2.0, 3.0);
        assert_eq!(a, Complex::new(3.0, 4.0));
        a -= Complex::new(1.0, 1.0);
        assert_eq!(a, Complex::new(2.0, 3.0));
        a *= Complex::new(0.0, 1.0); // multiply by i: (2+3i)·i = -3+2i
        assert_eq!(a, Complex::new(-3.0, 2.0));
        a /= Complex::new(0.0, 1.0); // divide by i: back to 2+3i
        assert_eq!(a, Complex::new(2.0, 3.0));
    }

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

    #[test]
    fn scalar_left_ops_match_right_ops_by_commutativity() {
        let z = Complex::new(2.0_f64, -3.0);
        let s = 4.0_f64;
        // addition and multiplication commute with the existing scalar-right impls
        assert_eq!(s + z, z + s);
        assert_eq!(s * z, z * s);
        // subtraction/division are the scalar treated as a zero-imaginary complex
        assert_eq!(s - z, Complex::new(s, 0.0) - z);
        assert_eq!(s / z, Complex::new(s, 0.0) / z);

        let zf = Complex::new(1.5_f32, 2.5);
        let sf = 3.0_f32;
        assert_eq!(sf + zf, zf + sf);
        assert_eq!(sf * zf, zf * sf);
        assert_eq!(sf - zf, Complex::new(sf, 0.0) - zf);
        assert_eq!(sf / zf, Complex::new(sf, 0.0) / zf);
    }
}
