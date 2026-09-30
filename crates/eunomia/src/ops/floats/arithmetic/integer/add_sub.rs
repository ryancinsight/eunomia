use crate::convert::SpecialValues;

use super::format::{decode, round_pack, Class, ExactSignificand};

trait RightSign {
    const INVERT: bool;
}

struct SameSign;

impl RightSign for SameSign {
    const INVERT: bool = false;
}

struct OppositeSign;

impl RightSign for OppositeSign {
    const INVERT: bool = true;
}

fn add_sub<P: SpecialValues, S: RightSign, const E: u32, const M: u32>(
    left_bits: u32,
    right_bits: u32,
) -> u32 {
    let left = decode::<P, E, M>(left_bits);
    let mut right = decode::<P, E, M>(right_bits);
    if S::INVERT {
        right.sign = !right.sign;
    }

    if left.class == Class::Nan || right.class == Class::Nan {
        return super::format::nan::<P, E, M>(false);
    }
    if left.class == Class::Infinity && right.class == Class::Infinity {
        return if left.sign == right.sign {
            super::format::encode_infinity_result::<P, E, M>(left.sign)
        } else {
            super::format::nan::<P, E, M>(false)
        };
    }
    if left.class == Class::Infinity {
        return super::format::encode_infinity_result::<P, E, M>(left.sign);
    }
    if right.class == Class::Infinity {
        return super::format::encode_infinity_result::<P, E, M>(right.sign);
    }
    if left.class == Class::Zero && right.class == Class::Zero {
        return super::format::zero::<E, M>(left.sign && right.sign);
    }
    if left.class == Class::Zero {
        return round_pack::<P, E, M>(
            right.sign,
            ExactSignificand::from_u32_shifted(right.significand, 0),
            right.exponent,
        );
    }
    if right.class == Class::Zero {
        return round_pack::<P, E, M>(
            left.sign,
            ExactSignificand::from_u32_shifted(left.significand, 0),
            left.exponent,
        );
    }

    let common_exponent = left.exponent.min(right.exponent);
    let left_magnitude = ExactSignificand::from_u32_shifted(
        left.significand,
        u32::try_from(left.exponent - common_exponent)
            .expect("invariant: common exponent does not exceed left exponent"),
    );
    let right_magnitude = ExactSignificand::from_u32_shifted(
        right.significand,
        u32::try_from(right.exponent - common_exponent)
            .expect("invariant: common exponent does not exceed right exponent"),
    );
    let (sign, magnitude) = if left.sign == right.sign {
        (left.sign, left_magnitude.add(right_magnitude))
    } else {
        match left_magnitude.cmp(&right_magnitude) {
            core::cmp::Ordering::Greater => (left.sign, left_magnitude.subtract(right_magnitude)),
            core::cmp::Ordering::Less => (right.sign, right_magnitude.subtract(left_magnitude)),
            core::cmp::Ordering::Equal => (false, ExactSignificand::ZERO),
        }
    };
    round_pack::<P, E, M>(sign, magnitude, common_exponent)
}

pub(in crate::ops::floats::arithmetic) fn add<P: SpecialValues, const E: u32, const M: u32>(
    left: u32,
    right: u32,
) -> u32 {
    add_sub::<P, SameSign, E, M>(left, right)
}

pub(in crate::ops::floats::arithmetic) fn sub<P: SpecialValues, const E: u32, const M: u32>(
    left: u32,
    right: u32,
) -> u32 {
    add_sub::<P, OppositeSign, E, M>(left, right)
}
