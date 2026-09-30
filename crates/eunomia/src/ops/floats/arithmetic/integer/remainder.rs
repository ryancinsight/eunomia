//! Exact remainder with Rust `%` semantics: the quotient truncates toward zero,
//! and the result keeps the dividend's sign. The Rust Reference defines this
//! rule at <https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators>.

use crate::convert::SpecialValues;

use super::format::{compare_magnitudes, decode, nan, round_pack, zero, Class, ExactSignificand};

fn power_of_two_mod(exponent: u32, modulus: u32) -> u32 {
    let mut result = 1 % modulus;
    let mut base = 2 % modulus;
    let mut power = exponent;
    while power != 0 {
        if power & 1 != 0 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        power >>= 1;
    }
    result
}

pub(in crate::ops::floats::arithmetic) fn rem<P: SpecialValues, const E: u32, const M: u32>(
    dividend_bits: u32,
    divisor_bits: u32,
) -> u32 {
    let dividend = decode::<P, E, M>(dividend_bits);
    let divisor = decode::<P, E, M>(divisor_bits);
    if dividend.class == Class::Nan
        || divisor.class == Class::Nan
        || dividend.class == Class::Infinity
        || divisor.class == Class::Zero
    {
        return nan::<P, E, M>(false);
    }
    if dividend.class == Class::Zero {
        return zero::<E, M>(dividend.sign);
    }
    if divisor.class == Class::Infinity {
        return round_pack::<P, E, M>(
            dividend.sign,
            ExactSignificand::from_u32_shifted(dividend.significand, 0),
            dividend.exponent,
        );
    }

    if compare_magnitudes(
        dividend.significand,
        dividend.exponent,
        divisor.significand,
        divisor.exponent,
    ) == core::cmp::Ordering::Less
    {
        return round_pack::<P, E, M>(
            dividend.sign,
            ExactSignificand::from_u32_shifted(dividend.significand, 0),
            dividend.exponent,
        );
    }

    let (remainder, exponent) = if dividend.exponent >= divisor.exponent {
        let scale = u32::try_from(dividend.exponent - divisor.exponent)
            .expect("invariant: dividend exponent is larger");
        let modulus = divisor.significand;
        let remainder =
            ((dividend.significand % modulus) * power_of_two_mod(scale, modulus)) % modulus;
        (remainder, divisor.exponent)
    } else {
        let divisor_shift = u32::try_from(divisor.exponent - dividend.exponent)
            .expect("invariant: divisor exponent is larger");
        let divisor_significand = divisor.significand << divisor_shift;
        (
            dividend.significand % divisor_significand,
            dividend.exponent,
        )
    };
    round_pack::<P, E, M>(
        dividend.sign,
        ExactSignificand::from_u32_shifted(remainder, 0),
        exponent,
    )
}
