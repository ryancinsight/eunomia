use crate::convert::SpecialValues;

use super::format::{
    decode, encode_infinity_result, nan, round_pack, zero, Class, ExactSignificand,
};

fn floor_log2_ratio(numerator: u32, denominator: u32) -> i32 {
    let numerator_bits = i32::try_from(numerator.ilog2()).expect("invariant: significand is small");
    let denominator_bits =
        i32::try_from(denominator.ilog2()).expect("invariant: significand is small");
    let mut exponent = numerator_bits - denominator_bits;
    let below_power = if exponent >= 0 {
        u64::from(numerator)
            < (u64::from(denominator)
                << u32::try_from(exponent).expect("invariant: nonnegative ratio exponent fits u32"))
    } else {
        (u64::from(numerator) << exponent.unsigned_abs()) < u64::from(denominator)
    };
    if below_power {
        exponent -= 1;
    }
    exponent
}

fn rounded_quotient(numerator: u32, denominator: u32, shift: i32) -> u128 {
    let numerator = u128::from(numerator);
    let denominator = u128::from(denominator);
    let (numerator, denominator) = if shift >= 0 {
        (
            numerator
                << u32::try_from(shift).expect("invariant: nonnegative quotient shift fits u32"),
            denominator,
        )
    } else {
        let denominator_shift = shift.unsigned_abs();
        let numerator_bits = numerator.ilog2() + 1;
        let denominator_bits = denominator.ilog2() + 1;
        if denominator_bits + denominator_shift > numerator_bits + 1 {
            return 0;
        }
        (numerator, denominator << denominator_shift)
    };
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let twice_remainder = remainder * 2;
    if twice_remainder > denominator || (twice_remainder == denominator && quotient & 1 != 0) {
        quotient + 1
    } else {
        quotient
    }
}

pub(in crate::ops::floats::arithmetic) fn div<P: SpecialValues, const E: u32, const M: u32>(
    numerator_bits: u32,
    denominator_bits: u32,
) -> u32 {
    let numerator = decode::<P, E, M>(numerator_bits);
    let denominator = decode::<P, E, M>(denominator_bits);
    if numerator.class == Class::Nan || denominator.class == Class::Nan {
        return nan::<P, E, M>(false);
    }

    let sign = numerator.sign ^ denominator.sign;
    if numerator.class == Class::Infinity && denominator.class == Class::Infinity {
        return nan::<P, E, M>(false);
    }
    if numerator.class == Class::Infinity {
        return encode_infinity_result::<P, E, M>(sign);
    }
    if denominator.class == Class::Infinity {
        return zero::<E, M>(sign);
    }
    if numerator.class == Class::Zero && denominator.class == Class::Zero {
        return nan::<P, E, M>(false);
    }
    if denominator.class == Class::Zero {
        return encode_infinity_result::<P, E, M>(sign);
    }
    if numerator.class == Class::Zero {
        return zero::<E, M>(sign);
    }

    let ratio_exponent = floor_log2_ratio(numerator.significand, denominator.significand);
    let top_exponent = numerator.exponent - denominator.exponent + ratio_exponent;
    let bias = i32::try_from(1_u32 << (E - 1)).expect("invariant: exponent bias fits i32") - 1;
    let minimum_normal = 1 - bias;
    let mantissa_width = i32::try_from(M).expect("invariant: mantissa width fits i32");
    let quantum_exponent = if top_exponent >= minimum_normal {
        top_exponent - mantissa_width
    } else {
        minimum_normal - mantissa_width
    };
    let scale = numerator.exponent - denominator.exponent - quantum_exponent;
    let significand = rounded_quotient(numerator.significand, denominator.significand, scale);
    round_pack::<P, E, M>(
        sign,
        ExactSignificand::from_u128(significand),
        quantum_exponent,
    )
}
