use crate::convert::SpecialValues;

use super::format::{decode, nan, Class};

pub(in crate::ops::floats::arithmetic) fn neg<P: SpecialValues, const E: u32, const M: u32>(
    bits: u32,
) -> u32 {
    let value = decode::<P, E, M>(bits);
    if value.class == Class::Nan {
        return nan::<P, E, M>(!value.sign);
    }
    (bits & ((1 << (E + M + 1)) - 1)) ^ (1 << (E + M))
}
