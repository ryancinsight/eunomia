use crate::{
    convert::{Finite, Ieee, SpecialValues},
    types::{Bf16, Bf4, Bf8, F16, F4, F8},
};

use super::integer;

macro_rules! impl_reduced_arithmetic {
    ($ty:ident, $storage:ty, $policy:ty, $exponent:literal, $mantissa:literal) => {
        impl core::ops::Add for $ty {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self(
                    <$storage>::try_from(integer::add::<$policy, $exponent, $mantissa>(
                        u32::from(self.0),
                        u32::from(rhs.0),
                    ))
                    .expect("invariant: encoded format fits storage"),
                )
            }
        }

        impl core::ops::AddAssign for $ty {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl core::ops::Sub for $ty {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self(
                    <$storage>::try_from(integer::sub::<$policy, $exponent, $mantissa>(
                        u32::from(self.0),
                        u32::from(rhs.0),
                    ))
                    .expect("invariant: encoded format fits storage"),
                )
            }
        }

        impl core::ops::SubAssign for $ty {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl core::ops::Mul for $ty {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self {
                Self(
                    <$storage>::try_from(integer::mul::<$policy, $exponent, $mantissa>(
                        u32::from(self.0),
                        u32::from(rhs.0),
                    ))
                    .expect("invariant: encoded format fits storage"),
                )
            }
        }

        impl core::ops::MulAssign for $ty {
            #[inline]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl core::ops::Div for $ty {
            type Output = Self;

            #[inline]
            fn div(self, rhs: Self) -> Self {
                Self(
                    <$storage>::try_from(integer::div::<$policy, $exponent, $mantissa>(
                        u32::from(self.0),
                        u32::from(rhs.0),
                    ))
                    .expect("invariant: encoded format fits storage"),
                )
            }
        }

        impl core::ops::DivAssign for $ty {
            #[inline]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        impl core::ops::Rem for $ty {
            type Output = Self;

            #[inline]
            fn rem(self, rhs: Self) -> Self {
                Self(
                    <$storage>::try_from(integer::rem::<$policy, $exponent, $mantissa>(
                        u32::from(self.0),
                        u32::from(rhs.0),
                    ))
                    .expect("invariant: encoded format fits storage"),
                )
            }
        }

        impl core::ops::RemAssign for $ty {
            #[inline]
            fn rem_assign(&mut self, rhs: Self) {
                *self = *self % rhs;
            }
        }

        impl core::ops::Neg for $ty {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                Self(
                    <$storage>::try_from(integer::neg::<$policy, $exponent, $mantissa>(u32::from(
                        self.0,
                    )))
                    .expect("invariant: encoded format fits storage"),
                )
            }
        }
    };
}

impl_reduced_arithmetic!(F16, u16, Ieee, 5, 10);
impl_reduced_arithmetic!(Bf16, u16, Ieee, 8, 7);
impl_reduced_arithmetic!(Bf8, u8, Ieee, 5, 2);
impl_reduced_arithmetic!(Bf4, u8, Finite, 2, 1);
impl_reduced_arithmetic!(F8, u8, Finite, 4, 3);
impl_reduced_arithmetic!(F4, u8, Finite, 3, 0);

const _: () = assert!(<Ieee as SpecialValues>::HAS_INFINITY);
const _: () = assert!(!<Finite as SpecialValues>::HAS_INFINITY);
