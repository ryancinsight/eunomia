use crate::convert::SpecialValues;

use super::format::{
    decode, encode_infinity_result, nan, round_pack, zero, Class, ExactSignificand,
};

pub(in crate::ops::floats::arithmetic) fn mul<P: SpecialValues, const E: u32, const M: u32>(
    left_bits: u32,
    right_bits: u32,
) -> u32 {
    let left = decode::<P, E, M>(left_bits);
    let right = decode::<P, E, M>(right_bits);
    if left.class == Class::Nan || right.class == Class::Nan {
        return nan::<P, E, M>(false);
    }

    let sign = left.sign ^ right.sign;
    if (left.class == Class::Infinity && right.class == Class::Zero)
        || (right.class == Class::Infinity && left.class == Class::Zero)
    {
        return nan::<P, E, M>(false);
    }
    if left.class == Class::Infinity || right.class == Class::Infinity {
        return encode_infinity_result::<P, E, M>(sign);
    }
    if left.class == Class::Zero || right.class == Class::Zero {
        return zero::<E, M>(sign);
    }

    let product = u128::from(left.significand) * u128::from(right.significand);
    round_pack::<P, E, M>(
        sign,
        ExactSignificand::from_u128(product),
        left.exponent + right.exponent,
    )
}
