use crate::convert::SpecialValues;

use super::format::{decode, nan, round_pack, Class, ExactSignificand};

pub(in crate::ops::floats::arithmetic) fn neg<P: SpecialValues, const E: u32, const M: u32>(
    bits: u32,
) -> u32 {
    let value = decode::<P, E, M>(bits);
    if value.class == Class::Nan {
        return nan::<P, E, M>(!value.sign);
    }
    (bits & ((1 << (E + M + 1)) - 1)) ^ (1 << (E + M))
}

/// Largest integer not greater than the value, computed in the target format.
///
/// Whole values (exponent `>= 0`, or a significand multiple of the fraction
/// scale) are identities, as are infinities and zeros; NaN canonicalizes with
/// the family payload. A positive fraction drops to its integer part; a
/// negative fraction rounds the magnitude up (`floor(-1.5) = -2`). The
/// result magnitude is at most `2^(M + 1)`, exactly representable, so
/// [`round_pack`] encodes without rounding.
pub(crate) fn floor<P: SpecialValues, const E: u32, const M: u32>(bits: u32) -> u32 {
    let value = decode::<P, E, M>(bits);
    if value.class == Class::Nan {
        return nan::<P, E, M>(value.sign);
    }
    if value.class != Class::Finite {
        // Infinities and zeros are their own floor.
        return bits;
    }
    if value.exponent >= 0 {
        return bits;
    }
    let shift = value.exponent.unsigned_abs();
    let (quotient, has_fraction) = if shift >= u32::BITS {
        (0, value.significand != 0)
    } else {
        (
            value.significand >> shift,
            value.significand & (u32::MAX >> (u32::BITS - shift)) != 0,
        )
    };
    if !has_fraction {
        return bits;
    }
    let magnitude = if value.sign { quotient + 1 } else { quotient };
    round_pack::<P, E, M>(value.sign, ExactSignificand::from_shifted(magnitude, 0), 0)
}
