//! Exact bit decoding and one-rounding encoding for reduced binary formats.
//!
//! A finite value is `(-1)^sign * significand * 2^exponent`. Addition aligns
//! those integer significands exactly; multiplication uses their exact
//! product; division rounds an integer quotient. `round_pack` performs the
//! sole destination rounding using nearest, ties-to-even, the mode specified
//! by Berkeley `SoftFloat` §6.1:
//! <https://www.jhauser.us/arithmetic/SoftFloat-3/doc/SoftFloat.html>.
//!
//! Bfloat16 spans base exponents -133 through 120, a gap of 253 bits, and has
//! at most 8 significand bits. An exact aligned sum therefore needs at most
//! 261 bits. Five u64 limbs provide 320 stack bits; the other shipped formats
//! have smaller exponent gaps. Products have at most 22 significant bits.

use crate::convert::SpecialValues;

const LIMBS: usize = 5;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Class {
    Zero,
    Finite,
    Infinity,
    Nan,
}

#[derive(Clone, Copy)]
pub(super) struct Decoded {
    pub(super) sign: bool,
    pub(super) class: Class,
    pub(super) significand: u32,
    pub(super) exponent: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExactSignificand([u64; LIMBS]);

impl ExactSignificand {
    pub(super) const ZERO: Self = Self([0; LIMBS]);

    pub(super) fn from_u32_shifted(value: u32, shift: u32) -> Self {
        if value == 0 {
            return Self::ZERO;
        }
        let word =
            usize::try_from(shift / u64::BITS).expect("invariant: format exponent gap fits usize");
        let bit = shift % u64::BITS;
        let value = u64::from(value);
        let mut words = [0; LIMBS];
        let low = words
            .get_mut(word)
            .expect("invariant: exact format sum fits five limbs");
        *low = value << bit;
        if bit != 0 && word + 1 < LIMBS {
            words[word + 1] = value >> (u64::BITS - bit);
        }
        Self(words)
    }

    pub(super) fn from_u128(value: u128) -> Self {
        let mut words = [0; LIMBS];
        words[0] = u64::try_from(value & u128::from(u64::MAX))
            .expect("invariant: masked low limb fits u64");
        words[1] = u64::try_from(value >> u64::BITS).expect("invariant: high u128 limb fits u64");
        Self(words)
    }

    pub(super) fn bit_len(self) -> u32 {
        self.0
            .iter()
            .rposition(|word| *word != 0)
            .map_or(0, |index| {
                u32::try_from(index).expect("invariant: limb index fits u32") * u64::BITS
                    + (u64::BITS - self.0[index].leading_zeros())
            })
    }

    fn bit(self, index: u32) -> bool {
        let word = usize::try_from(index / u64::BITS).expect("invariant: bit index fits usize");
        self.0
            .get(word)
            .is_some_and(|value| *value & (1 << (index % u64::BITS)) != 0)
    }

    fn any_below(self, exclusive: u32) -> bool {
        let whole = usize::try_from(exclusive / u64::BITS)
            .expect("invariant: rounded shift fits usize")
            .min(LIMBS);
        if self.0.iter().take(whole).any(|word| *word != 0) {
            return true;
        }
        let partial = exclusive % u64::BITS;
        whole < LIMBS && partial != 0 && self.0[whole] & ((1 << partial) - 1) != 0
    }

    fn shifted_low(self, shift: u32) -> u64 {
        let word = usize::try_from(shift / u64::BITS).expect("invariant: rounded shift fits usize");
        if word >= LIMBS {
            return 0;
        }
        let bit = shift % u64::BITS;
        if bit == 0 {
            self.0[word]
        } else {
            let upper = if word + 1 < LIMBS {
                self.0[word + 1] << (u64::BITS - bit)
            } else {
                0
            };
            (self.0[word] >> bit) | upper
        }
    }

    pub(super) fn rounded_shift_right(self, shift: u32) -> u64 {
        let kept = self.shifted_low(shift);
        if shift == 0 {
            return kept;
        }
        let guard = self.bit(shift - 1);
        let sticky = self.any_below(shift - 1);
        if guard && (sticky || kept & 1 != 0) {
            kept + 1
        } else {
            kept
        }
    }

    pub(super) fn add(self, rhs: Self) -> Self {
        let mut result = [0; LIMBS];
        let mut carry = false;
        for (index, output) in result.iter_mut().enumerate() {
            let (sum, first_carry) = self.0[index].overflowing_add(rhs.0[index]);
            let (sum, second_carry) = sum.overflowing_add(u64::from(carry));
            *output = sum;
            carry = first_carry || second_carry;
        }
        Self(result)
    }

    pub(super) fn subtract(self, rhs: Self) -> Self {
        let mut result = [0; LIMBS];
        let mut borrow = false;
        for (index, output) in result.iter_mut().enumerate() {
            let (difference, first_borrow) = self.0[index].overflowing_sub(rhs.0[index]);
            let (difference, second_borrow) = difference.overflowing_sub(u64::from(borrow));
            *output = difference;
            borrow = first_borrow || second_borrow;
        }
        Self(result)
    }
}

impl Ord for ExactSignificand {
    fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
        self.0.iter().rev().cmp(rhs.0.iter().rev())
    }
}

impl PartialOrd for ExactSignificand {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

/// Decode a bit pattern into its exact sign, class, significand, and exponent.
pub(super) fn decode<P: SpecialValues, const E: u32, const M: u32>(bits: u32) -> Decoded {
    let format_mask = (1 << (1 + E + M)) - 1;
    let bits = bits & format_mask;
    let sign = (bits >> (E + M)) != 0;
    let exponent = (bits >> M) & ((1 << E) - 1);
    let mantissa = bits & ((1 << M) - 1);
    let all_exponents = (1 << E) - 1;
    let bias = i32::try_from(1_u32 << (E - 1)).expect("invariant: exponent bias fits i32") - 1;

    if exponent == all_exponents {
        let class = if P::HAS_INFINITY && mantissa == 0 {
            Class::Infinity
        } else {
            Class::Nan
        };
        return Decoded {
            sign,
            class,
            significand: 0,
            exponent: 0,
        };
    }
    if exponent == 0 {
        if mantissa == 0 {
            return Decoded {
                sign,
                class: Class::Zero,
                significand: 0,
                exponent: 0,
            };
        }
        return Decoded {
            sign,
            class: Class::Finite,
            significand: mantissa,
            exponent: 1 - bias - i32::try_from(M).expect("invariant: mantissa width fits i32"),
        };
    }
    Decoded {
        sign,
        class: Class::Finite,
        significand: (1 << M) | mantissa,
        exponent: i32::try_from(exponent).expect("invariant: encoded exponent fits i32")
            - bias
            - i32::try_from(M).expect("invariant: mantissa width fits i32"),
    }
}

fn sign_bits<const E: u32, const M: u32>(sign: bool) -> u32 {
    (u32::from(sign)) << (E + M)
}

fn all_exponents<const E: u32>() -> u32 {
    (1 << E) - 1
}

fn mantissa_mask<const M: u32>() -> u32 {
    (1 << M) - 1
}

pub(super) fn zero<const E: u32, const M: u32>(sign: bool) -> u32 {
    sign_bits::<E, M>(sign)
}

pub(super) fn nan<P: SpecialValues, const E: u32, const M: u32>(sign: bool) -> u32 {
    let payload = if P::HAS_INFINITY {
        1 << M.saturating_sub(1)
    } else {
        mantissa_mask::<M>()
    };
    sign_bits::<E, M>(sign) | (all_exponents::<E>() << M) | payload
}

fn max_finite<const E: u32, const M: u32>(sign: bool) -> u32 {
    sign_bits::<E, M>(sign) | ((all_exponents::<E>() - 1) << M) | mantissa_mask::<M>()
}

pub(super) fn encode_infinity_result<P: SpecialValues, const E: u32, const M: u32>(
    sign: bool,
) -> u32 {
    if P::HAS_INFINITY {
        sign_bits::<E, M>(sign) | (all_exponents::<E>() << M)
    } else {
        max_finite::<E, M>(sign)
    }
}

/// Encode an exact magnitude, applying one nearest-ties-to-even rounding step.
///
/// Underflow returns signed zero or the smallest normal/subnormal encoding;
/// overflow uses the result prescribed by the special-value representation.
pub(super) fn round_pack<P: SpecialValues, const E: u32, const M: u32>(
    sign: bool,
    magnitude: ExactSignificand,
    exponent: i32,
) -> u32 {
    let sign_bits = sign_bits::<E, M>(sign);
    let bit_len = magnitude.bit_len();
    if bit_len == 0 {
        return zero::<E, M>(sign);
    }

    let bias = i32::try_from(1_u32 << (E - 1)).expect("invariant: exponent bias fits i32") - 1;
    let all_exponents = all_exponents::<E>();
    let minimum_normal = 1 - bias;
    let maximum_normal =
        i32::try_from(all_exponents).expect("invariant: exponent field fits i32") - 1 - bias;
    let precision = M + 1;
    let top_exponent =
        exponent + i32::try_from(bit_len).expect("invariant: exact significand fits i32") - 1;

    if top_exponent >= minimum_normal {
        let drop = bit_len.saturating_sub(precision);
        let mut significand = if bit_len > precision {
            magnitude.rounded_shift_right(drop)
        } else {
            magnitude.shifted_low(0) << (precision - bit_len)
        };
        let mut result_exponent = top_exponent;
        if significand == 1 << precision {
            significand >>= 1;
            result_exponent += 1;
        }
        if result_exponent > maximum_normal {
            return encode_infinity_result::<P, E, M>(sign);
        }
        let encoded_exponent = u32::try_from(result_exponent + bias)
            .expect("invariant: rounded exponent is in the normal range");
        let encoded_mantissa = u32::try_from(significand)
            .expect("invariant: rounded significand fits u32")
            & mantissa_mask::<M>();
        return sign_bits | (encoded_exponent << M) | encoded_mantissa;
    }

    let minimum_subnormal_exponent =
        minimum_normal - i32::try_from(M).expect("invariant: mantissa width fits i32");
    let shift = minimum_subnormal_exponent - exponent;
    let rounded = if shift > 0 {
        magnitude.rounded_shift_right(
            u32::try_from(shift).expect("invariant: positive rounded shift fits u32"),
        )
    } else {
        magnitude.shifted_low(0) << shift.unsigned_abs()
    };
    if rounded == 0 {
        return zero::<E, M>(sign);
    }
    if rounded >= 1 << M {
        return sign_bits | (1 << M);
    }
    sign_bits | u32::try_from(rounded).expect("invariant: subnormal significand fits u32")
}

/// Compare two positive finite values without rounding either operand.
pub(super) fn compare_magnitudes(
    left_significand: u32,
    left_exponent: i32,
    right_significand: u32,
    right_exponent: i32,
) -> core::cmp::Ordering {
    let left_top = left_exponent
        + i32::try_from(left_significand.ilog2()).expect("invariant: significand width fits i32");
    let right_top = right_exponent
        + i32::try_from(right_significand.ilog2()).expect("invariant: significand width fits i32");
    match left_top.cmp(&right_top) {
        core::cmp::Ordering::Equal => {
            let common_exponent = left_exponent.min(right_exponent);
            let left = ExactSignificand::from_u32_shifted(
                left_significand,
                u32::try_from(left_exponent - common_exponent)
                    .expect("invariant: common exponent does not exceed left exponent"),
            );
            let right = ExactSignificand::from_u32_shifted(
                right_significand,
                u32::try_from(right_exponent - common_exponent)
                    .expect("invariant: common exponent does not exceed right exponent"),
            );
            left.cmp(&right)
        }
        order => order,
    }
}

#[cfg(test)]
mod tests {
    use super::ExactSignificand;

    #[test]
    fn exact_sum_keeps_guard_and_sticky_limbs() {
        let high = ExactSignificand::from_u32_shifted(0xA5, 253);
        let low = ExactSignificand::from_u32_shifted(3, 0);
        let exact = high.add(low);
        assert_eq!(exact.bit_len(), 261);
        assert_eq!(exact.rounded_shift_right(253), 0xA5);
        assert!(exact.any_below(253));
        assert_eq!(exact.subtract(low), high);
    }
}
