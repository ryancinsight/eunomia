//! Round-to-odd narrowing of `f64` to `f32`: the intermediate that lets a
//! reduced format round a 53-bit significand once instead of twice.

/// `f64` fraction field width.
const F64_FRAC_BITS: i32 = 52;
/// All-ones `f64` exponent field (infinity and NaN).
const F64_EXP_ALL_ONES: u64 = 0x7FF;
/// Exponent `e` of `value = significand * 2^e` for the `f64` with exponent
/// field 1, the smallest normal exponent field, whose significand includes the
/// implicit bit: `1 - 1023 - 52`. A subnormal shares it.
const F64_SMALLEST_SCALE: i32 = -1074;
/// `f64` exponent bias plus fraction width: `scale = field - 1075` for a normal.
const F64_NORMAL_SCALE_OFFSET: i32 = 1075;
/// `f32` fraction field width.
const F32_FRAC_BITS: i32 = 23;
/// `f32` exponent bias.
const F32_EXP_BIAS: i32 = 127;
/// Largest unbiased `f32` exponent.
const F32_MAX_EXP: i32 = 127;
/// Smallest unbiased `f32` normal exponent.
const F32_MIN_EXP: i32 = -126;
/// Exponent of the `f32` subnormal quantum, `2^-149`.
const F32_QUANTUM_EXP: i32 = -149;
/// The `f32` quiet-NaN bit.
const F32_QUIET_BIT: u32 = 1 << (F32_FRAC_BITS - 1);
/// Bits the `f64` fraction loses becoming an `f32` fraction.
const FRAC_NARROWING: i32 = F64_FRAC_BITS - F32_FRAC_BITS;

/// `value >> shift`, with the last bit set when any shifted-out bit is set:
/// the round-to-odd truncation both float narrowing and count conversion
/// share. A `shift` of 64 or more discards every bit.
#[inline]
pub(crate) fn odd_truncated(value: u64, shift: u32) -> u64 {
    match 1_u64.checked_shl(shift) {
        Some(unit) => (value >> shift) | u64::from(value & (unit - 1) != 0),
        None => u64::from(value != 0),
    }
}

/// Rounds `value` to 24 significant bits with round-to-odd: the leading 24
/// bits are truncated toward zero and the last kept bit is set when any
/// discarded bit is nonzero. The result is an exactly representable `f32`.
///
/// Results below `2^-126` keep as many bits as the `f32` subnormal grid
/// `2^-149` holds, so every nonzero input yields a nonzero output; a
/// magnitude of `2^128` or more yields infinity. Infinity, NaN (payload
/// quieted as the hardware narrowing does) and signed zero pass through.
///
/// Narrowing the result with round-to-nearest-even into a format of precision
/// `p <= 22` whose exponent range lies inside `f32`'s equals rounding `value`
/// directly, for the reason given at [`odd_rounded_magnitude`](super::count::odd_rounded_magnitude):
/// every point and midpoint of such a format is an even multiple of the
/// quantum at the result's scale, and an inexact result is odd. In the `f32`
/// subnormal range the quantum is `2^-149`, so the same holds while the
/// format's smallest subnormal is at least `2^-147`; both hold for every
/// reduced format eunomia ships. A round-to-nearest intermediate fails: a
/// value just above a midpoint rounds onto it, and the tie then goes to the
/// even neighbour, which may be the wrong one.
#[inline]
pub(crate) fn odd_rounded(value: f64) -> f32 {
    let bits = value.to_bits();
    let sign = u32::from(bits >> 63 != 0) << 31;
    let field = (bits >> F64_FRAC_BITS) & F64_EXP_ALL_ONES;
    let fraction = bits & ((1 << F64_FRAC_BITS) - 1);

    if field == F64_EXP_ALL_ONES {
        let payload = if fraction == 0 {
            0
        } else {
            F32_QUIET_BIT
                | u32::try_from(fraction >> FRAC_NARROWING)
                    .expect("invariant: the shifted fraction has 23 bits")
        };
        return f32::from_bits(sign | (0xFF << F32_FRAC_BITS) | payload);
    }

    let (significand, scale) = if field == 0 {
        (fraction, F64_SMALLEST_SCALE)
    } else {
        (
            fraction | (1 << F64_FRAC_BITS),
            i32::try_from(field).expect("invariant: the exponent field has 11 bits")
                - F64_NORMAL_SCALE_OFFSET,
        )
    };
    if significand == 0 {
        return f32::from_bits(sign);
    }

    let width = i32::try_from(u64::BITS - significand.leading_zeros())
        .expect("invariant: a significand width is at most 64");
    let exponent = width - 1 + scale;
    if exponent > F32_MAX_EXP {
        return f32::from_bits(sign | (0xFF << F32_FRAC_BITS));
    }

    let quantum = (exponent - F32_FRAC_BITS).max(F32_QUANTUM_EXP);
    let shift = u32::try_from(quantum - scale)
        .expect("invariant: an f64 significand is at least 29 bits wider than an f32 one");
    let odd = u32::try_from(odd_truncated(significand, shift))
        .expect("invariant: at most 24 significant bits are kept");

    if exponent >= F32_MIN_EXP {
        let field = u32::try_from(exponent + F32_EXP_BIAS)
            .expect("invariant: a normal f32 exponent field is positive");
        // `odd` is `1.xxx` with its leading bit at 23: that bit carries into
        // the exponent field, so the field written is one below its value.
        f32::from_bits(sign | (((field - 1) << F32_FRAC_BITS) + odd))
    } else {
        f32::from_bits(sign | odd)
    }
}

#[cfg(test)]
mod tests;
