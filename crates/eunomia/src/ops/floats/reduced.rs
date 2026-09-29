//! One-rounding arithmetic for the IEEE reduced-precision types.
//!
//! Results use round-to-nearest, ties-to-even as specified by `SoftFloat` §6.1:
//! <https://www.jhauser.us/arithmetic/SoftFloat-3/doc/SoftFloat.html>.
use crate::types::{Bf16, F16};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Class {
    Zero,
    Finite,
    Infinity,
    Nan,
}

#[derive(Clone, Copy)]
struct Decoded {
    sign: bool,
    class: Class,
    significand: u64,
    exponent: i32,
}

fn decode<const E: u32, const M: u32>(bits: u16) -> Decoded {
    let bits = u32::from(bits);
    let sign = bits >> (E + M) != 0;
    let encoded_exponent = (bits >> M) & ((1 << E) - 1);
    let fraction = bits & ((1 << M) - 1);
    let bias = (1_i32 << (E - 1)) - 1;
    let (class, significand, exponent) = if encoded_exponent == (1 << E) - 1 {
        (
            if fraction == 0 {
                Class::Infinity
            } else {
                Class::Nan
            },
            0,
            0,
        )
    } else if encoded_exponent == 0 {
        if fraction == 0 {
            (Class::Zero, 0, 0)
        } else {
            (
                Class::Finite,
                u64::from(fraction),
                1 - bias - i32::try_from(M).expect("invariant: mantissa width fits i32"),
            )
        }
    } else {
        (
            Class::Finite,
            u64::from((1 << M) | fraction),
            i32::try_from(encoded_exponent).expect("invariant: exponent fits i32")
                - bias
                - i32::try_from(M).expect("invariant: mantissa width fits i32"),
        )
    };
    Decoded {
        sign,
        class,
        significand,
        exponent,
    }
}

fn shift_right_jam(value: u64, shift: u32) -> u64 {
    if shift == 0 {
        value
    } else if shift >= u64::BITS {
        u64::from(value != 0)
    } else {
        (value >> shift) | u64::from(value & ((1 << shift) - 1) != 0)
    }
}

fn round_right_even(value: u64, shift: u32) -> u64 {
    if shift == 0 {
        return value;
    }
    if shift > u64::BITS {
        return 0;
    }
    if shift == u64::BITS {
        return u64::from(value > (1 << (u64::BITS - 1)));
    }
    let kept = value >> shift;
    let remainder = value & ((1 << shift) - 1);
    let halfway = 1 << (shift - 1);
    kept + u64::from(remainder > halfway || (remainder == halfway && kept & 1 != 0))
}

fn pack<const E: u32, const M: u32>(sign: bool, magnitude: u64, exponent: i32) -> u16 {
    let sign_bits = u32::from(sign) << (E + M);
    if magnitude == 0 {
        return u16::try_from(sign_bits).expect("invariant: reduced encoding fits u16");
    }
    let bias = (1_i32 << (E - 1)) - 1;
    let bit_len = u64::BITS - magnitude.leading_zeros();
    let top_exponent = exponent + i32::try_from(bit_len).expect("invariant: width fits i32") - 1;
    let minimum_normal = 1 - bias;
    let precision = M + 1;
    let encoded = if top_exponent >= minimum_normal {
        let drop = bit_len.saturating_sub(precision);
        let mut significand = if drop == 0 {
            magnitude << (precision - bit_len)
        } else {
            round_right_even(magnitude, drop)
        };
        let mut result_exponent = top_exponent;
        if significand == 1 << precision {
            significand >>= 1;
            result_exponent += 1;
        }
        let maximum_normal = ((1_i32 << E) - 2) - bias;
        if result_exponent > maximum_normal {
            sign_bits | (((1 << E) - 1) << M)
        } else {
            let field = u32::try_from(result_exponent + bias)
                .expect("invariant: result exponent is normal");
            sign_bits
                | (field << M)
                | (u32::try_from(significand).expect("invariant: significand fits u32")
                    & ((1 << M) - 1))
        }
    } else {
        let minimum_subnormal =
            minimum_normal - i32::try_from(M).expect("invariant: mantissa width fits i32");
        let shift = minimum_subnormal - exponent;
        let rounded = if shift > 0 {
            round_right_even(
                magnitude,
                u32::try_from(shift).expect("invariant: shift fits u32"),
            )
        } else {
            magnitude << shift.unsigned_abs()
        };
        if rounded >= 1 << M {
            sign_bits | (1 << M)
        } else {
            sign_bits | u32::try_from(rounded).expect("invariant: subnormal fits u32")
        }
    };
    u16::try_from(encoded).expect("invariant: reduced encoding fits u16")
}

fn nan<const E: u32, const M: u32>() -> u16 {
    u16::try_from((((1 << E) - 1) << M) | (1 << (M - 1))).expect("invariant: reduced NaN fits u16")
}

fn add<const E: u32, const M: u32>(left_bits: u16, mut right_bits: u16, subtract: bool) -> u16 {
    let sign_mask = 1 << (E + M);
    if subtract {
        right_bits ^= u16::try_from(sign_mask).expect("invariant: sign bit fits u16");
    }
    let left = decode::<E, M>(left_bits);
    let right = decode::<E, M>(right_bits);
    if left.class == Class::Nan || right.class == Class::Nan {
        return nan::<E, M>();
    }
    if left.class == Class::Infinity || right.class == Class::Infinity {
        if left.class == Class::Infinity
            && right.class == Class::Infinity
            && left.sign != right.sign
        {
            return nan::<E, M>();
        }
        return if left.class == Class::Infinity {
            left_bits
        } else {
            right_bits
        };
    }
    if left.class == Class::Zero && right.class == Class::Zero {
        return u16::try_from(u32::from(left.sign && right.sign) << (E + M))
            .expect("invariant: zero fits u16");
    }
    if left.class == Class::Zero {
        return right_bits;
    }
    if right.class == Class::Zero {
        return left_bits;
    }
    let magnitude_mask = u16::try_from(sign_mask - 1).expect("invariant: mask fits u16");
    let (larger, smaller) = if left_bits & magnitude_mask >= right_bits & magnitude_mask {
        (left, right)
    } else {
        (right, left)
    };
    let shift = u32::try_from(larger.exponent - smaller.exponent)
        .expect("invariant: magnitude orders exponent");
    let larger_extended = larger.significand << 3;
    let smaller_extended = shift_right_jam(smaller.significand << 3, shift);
    let magnitude = if larger.sign == smaller.sign {
        larger_extended + smaller_extended
    } else {
        larger_extended - smaller_extended
    };
    pack::<E, M>(
        larger.sign && magnitude != 0,
        magnitude,
        larger.exponent - 3,
    )
}

fn multiply<const E: u32, const M: u32>(left_bits: u16, right_bits: u16) -> u16 {
    let left = decode::<E, M>(left_bits);
    let right = decode::<E, M>(right_bits);
    if left.class == Class::Nan || right.class == Class::Nan {
        return nan::<E, M>();
    }
    let sign = left.sign ^ right.sign;
    if (left.class == Class::Infinity && right.class == Class::Zero)
        || (right.class == Class::Infinity && left.class == Class::Zero)
    {
        return nan::<E, M>();
    }
    if left.class == Class::Infinity || right.class == Class::Infinity {
        return u16::try_from((u32::from(sign) << (E + M)) | (((1 << E) - 1) << M))
            .expect("invariant: infinity fits u16");
    }
    if left.class == Class::Zero || right.class == Class::Zero {
        return u16::try_from(u32::from(sign) << (E + M)).expect("invariant: zero fits u16");
    }
    pack::<E, M>(
        sign,
        left.significand * right.significand,
        left.exponent + right.exponent,
    )
}

macro_rules! impl_reduced_arithmetic {
    ($ty:ident, $exponent:literal, $mantissa:literal) => {
        impl core::ops::Add for $ty {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self(add::<$exponent, $mantissa>(self.0, rhs.0, false))
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
                Self(add::<$exponent, $mantissa>(self.0, rhs.0, true))
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
                Self(multiply::<$exponent, $mantissa>(self.0, rhs.0))
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
                Self::from_f32(self.to_f32() / rhs.to_f32())
            }
        }
        impl core::ops::Rem for $ty {
            type Output = Self;
            #[inline]
            fn rem(self, rhs: Self) -> Self {
                Self::from_f32(self.to_f32() % rhs.to_f32())
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
                Self(self.0 ^ 0x8000)
            }
        }
    };
}

impl_reduced_arithmetic!(F16, 5, 10);
impl_reduced_arithmetic!(Bf16, 8, 7);

#[cfg(test)]
mod tests {
    use super::{Bf16, F16};

    struct Vectors {
        sign: u16,
        one: u16,
        half_ulp: u16,
        minimum_normal: u16,
        largest_subnormal: u16,
        maximum: u16,
        infinity: u16,
        nan: u16,
        half: u16,
        two: u16,
    }

    macro_rules! check {
        ($ty:ty, $v:expr) => {{
            let v = $v;
            let value = <$ty>::from_bits;
            assert_eq!((value(0) + value(v.sign)).to_bits(), 0);
            assert_eq!((value(v.sign) + value(v.sign)).to_bits(), v.sign);
            assert_eq!((value(1) + value(1)).to_bits(), 2);
            assert_eq!(
                (value(v.minimum_normal) - value(v.largest_subnormal)).to_bits(),
                1
            );
            assert_eq!((value(v.one) + value(v.half_ulp)).to_bits(), v.one);
            assert_eq!((value(v.one + 1) + value(v.half_ulp)).to_bits(), v.one + 2);
            assert_eq!((value(v.sign | v.one) + value(v.one)).to_bits(), 0);
            assert_eq!((value(v.maximum) + value(v.maximum)).to_bits(), v.infinity);
            assert_eq!(
                (value(v.minimum_normal) * value(v.half)).to_bits(),
                v.minimum_normal / 2
            );
            assert_eq!((value(v.maximum) * value(v.two)).to_bits(), v.infinity);
            assert_eq!((value(v.sign) * value(v.two)).to_bits(), v.sign);
            assert_eq!((value(v.infinity) * value(0)).to_bits(), v.nan);
            assert_eq!((value(v.nan) + value(v.one)).to_bits(), v.nan);
            assert_eq!((value(v.infinity) - value(v.infinity)).to_bits(), v.nan);

            let mut assigned = value(v.one);
            assigned += value(v.one);
            assert_eq!(assigned.to_bits(), v.two);
            assigned -= value(v.one);
            assert_eq!(assigned.to_bits(), v.one);
            assigned *= value(v.half);
            assert_eq!(assigned.to_bits(), v.half);
        }};
    }

    #[test]
    fn binary16_operators_round_once_in_format() {
        check!(
            F16,
            Vectors {
                sign: 0x8000,
                one: 0x3c00,
                half_ulp: 0x1000,
                minimum_normal: 0x0400,
                largest_subnormal: 0x03ff,
                maximum: 0x7bff,
                infinity: 0x7c00,
                nan: 0x7e00,
                half: 0x3800,
                two: 0x4000,
            }
        );
    }

    #[test]
    fn bfloat16_operators_round_once_in_format() {
        check!(
            Bf16,
            Vectors {
                sign: 0x8000,
                one: 0x3f80,
                half_ulp: 0x3b80,
                minimum_normal: 0x0080,
                largest_subnormal: 0x007f,
                maximum: 0x7f7f,
                infinity: 0x7f80,
                nan: 0x7fc0,
                half: 0x3f00,
                two: 0x4000,
            }
        );
    }
}
