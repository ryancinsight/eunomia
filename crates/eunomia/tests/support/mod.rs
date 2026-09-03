//! Independent numerical references shared by reduced-precision integration tests.

/// Return whether a reduced-precision bit pattern encodes a NaN.
pub(crate) const fn is_nan<const E: u32, const M: u32>(bits: u32) -> bool {
    let exponent = (bits >> M) & ((1 << E) - 1);
    exponent == (1 << E) - 1 && bits & ((1 << M) - 1) != 0
}

/// Widen an IEEE reduced-precision bit pattern through the format definition.
///
/// The reference uses exact `f64` powers of two and then narrows to `f32`.
/// Every shipped reduced format is exactly representable in `f32`, so this is
/// an independent value-level oracle for the production bit-manipulation path.
pub(crate) fn widen<const E: u32, const M: u32>(bits: u32) -> u32 {
    let sign = (bits >> (E + M)) & 1;
    let exponent = (bits >> M) & ((1 << E) - 1);
    let mantissa = bits & ((1 << M) - 1);
    let all_ones = (1 << E) - 1;
    if exponent == all_ones {
        return if mantissa == 0 {
            if sign == 0 {
                f32::INFINITY.to_bits()
            } else {
                f32::NEG_INFINITY.to_bits()
            }
        } else {
            f32::NAN.to_bits()
        };
    }

    let bias = (1 << (E - 1)) - 1;
    let unbiased = if exponent == 0 {
        1 - bias - M as i32
    } else {
        exponent as i32 - bias - M as i32
    };
    let significand = if exponent == 0 {
        mantissa
    } else {
        (1 << M) | mantissa
    };
    let mut value = f64::from(significand) * 2.0f64.powi(unbiased);
    if sign != 0 {
        value = -value;
    }
    (value as f32).to_bits()
}

fn round_nearest_even(value: f64) -> u32 {
    let lower = value.floor() as u32;
    let fraction = value - f64::from(lower);
    if fraction > 0.5 || (fraction == 0.5 && lower & 1 == 1) {
        lower + 1
    } else {
        lower
    }
}

/// Narrow an `f32` bit pattern using the IEEE format definition.
///
/// The source is widened to `f64`, which is exact for every `f32` input. The
/// target significand is then rounded from an exact power-of-two scaling; this
/// keeps the oracle independent from the production integer-shift kernel while
/// preserving ties-to-even at normal and subnormal boundaries.
pub(crate) fn narrow<const E: u32, const M: u32>(f32_bits: u32) -> u32 {
    let sign = f32_bits >> 31;
    let source_exponent = (f32_bits >> 23) & 0xFF;
    let source_mantissa = f32_bits & 0x7F_FFFF;
    let bias = (1 << (E - 1)) - 1;
    let exponent_all_ones = (1 << E) - 1;
    let sign_prefix = sign << (E + M);

    if source_exponent == 0xFF {
        return if source_mantissa == 0 {
            sign_prefix | (exponent_all_ones << M)
        } else {
            sign_prefix | (exponent_all_ones << M) | 1
        };
    }
    if source_exponent == 0 && source_mantissa == 0 {
        return sign_prefix;
    }

    let value = f64::from(f32::from_bits(f32_bits)).abs();
    let source_unbiased = if source_exponent == 0 {
        -149 + source_mantissa.ilog2() as i32
    } else {
        source_exponent as i32 - 127
    };
    let minimum_normal_exponent = 1 - bias;
    let maximum_normal_exponent = exponent_all_ones as i32 - 1 - bias;

    if source_unbiased > maximum_normal_exponent {
        return sign_prefix | (exponent_all_ones << M);
    }

    if source_unbiased < minimum_normal_exponent {
        let minimum_subnormal_exponent = minimum_normal_exponent - M as i32;
        let scaled = value / 2.0f64.powi(minimum_subnormal_exponent);
        let rounded = round_nearest_even(scaled);
        return if rounded >= 1 << M {
            sign_prefix | (1 << M)
        } else {
            sign_prefix | rounded
        };
    }

    let mut exponent = source_unbiased;
    let scaled = (value / 2.0f64.powi(exponent) - 1.0) * 2.0f64.powi(M as i32);
    let rounded = round_nearest_even(scaled + f64::from(1 << M));
    if rounded == 1 << (M + 1) {
        exponent += 1;
        if exponent > maximum_normal_exponent {
            return sign_prefix | (exponent_all_ones << M);
        }
        return sign_prefix | (((exponent + bias) as u32) << M);
    }

    sign_prefix | (((exponent + bias) as u32) << M) | (rounded - (1 << M))
}
