//! Generic IEEE-754 binary narrowing/widening kernel — the native conversion
//! SSOT that replaces the `half` crate's `f16`/`bf16` conversions and unifies
//! the sub-byte float formats onto one authoritative implementation.
//!
//! One const-parameterized pair of functions converts between `f32` and any
//! reduced binary floating-point format described by its exponent width `E` and
//! mantissa width `M`. The format layout is `[sign(1) | exp(E) | mant(M)]` with
//! exponent bias `2^(E-1) - 1`. A monomorphized special-value policy selects
//! either IEEE infinity/NaN encodings or a finite-only format that reserves the
//! entire top exponent for NaN and saturates overflow. Narrowing rounds to
//! nearest, ties to even, matching IEEE 754 and the `half` reference this
//! displaces.
//!
//! The widths and policy are compile-time parameters, so each format
//! monomorphizes without runtime width or policy dispatch while sharing one
//! implementation.
//!
//! The policy is internal until a distinct externally required format family
//! needs public selection (backlog E-024).

trait SpecialValues {
    const HAS_INFINITY: bool;
}

struct Ieee;

impl SpecialValues for Ieee {
    const HAS_INFINITY: bool = true;
}

struct Finite;

impl SpecialValues for Finite {
    const HAS_INFINITY: bool = false;
}

/// `f32` mantissa field width (IEEE 754 binary32).
const F32_MANT_BITS: u32 = 23;
/// `f32` exponent bias.
const F32_EXP_BIAS: i32 = 127;
/// `f32` all-ones exponent field (infinity/NaN).
const F32_EXP_ALL_ONES: u32 = 0xFF;

/// Shift `value` right by `drop` bits, rounding to nearest with ties to even.
///
/// `value` is treated as an exact non-negative integer significand; the returned
/// integer is the correctly rounded quotient `value / 2^drop`. A rounding carry
/// (e.g. `0b0111… -> 0b1000…`) is reflected in the result, so callers relying on
/// the carry rippling into an adjacent exponent field get the IEEE-correct
/// behavior for free.
#[inline]
const fn round_to_nearest_even(value: u32, drop: u32) -> u32 {
    if drop == 0 {
        return value;
    }
    if drop >= u32::BITS {
        // Every significand bit is dropped; the value is below half the LSB.
        return 0;
    }
    let keep = value >> drop;
    let dropped = value & ((1 << drop) - 1);
    let half = 1u32 << (drop - 1);
    // Round up on `> half`, or on an exact tie (`== half`) when `keep` is odd
    // (round to even). Round down otherwise.
    if dropped > half || (dropped == half && (keep & 1) == 1) {
        keep + 1
    } else {
        keep
    }
}

/// Widen a reduced `(E, M)` bit pattern (held in the low `1 + E + M` bits of
/// `bits`) to the `f32` bit pattern of the same numeric value.
///
/// Exact for every input: `f32` represents every value any `(E, M)` format with
/// `E <= 8` and `M <= 23` can hold, so no rounding occurs.
#[inline]
#[must_use]
pub const fn widen<const E: u32, const M: u32>(bits: u32) -> u32 {
    widen_with::<Ieee, E, M>(bits)
}

/// Widen a finite-only format that reserves its top exponent for NaN.
#[inline]
#[must_use]
pub(crate) const fn widen_finite<const E: u32, const M: u32>(bits: u32) -> u32 {
    widen_with::<Finite, E, M>(bits)
}

/// Widen an IEEE reduced format and return the exact high 16 bits.
#[inline]
#[must_use]
pub(crate) const fn widen_high_word<const E: u32, const M: u32>(bits: u32) -> u16 {
    high_word::<M>(widen::<E, M>(bits))
}

/// Widen a finite-only reduced format and return the exact high 16 bits.
///
/// This is the storage representation of the result for any destination format
/// that shares binary32's exponent and retains at most seven mantissa bits.
#[inline]
#[must_use]
pub(crate) const fn widen_finite_high_word<const E: u32, const M: u32>(bits: u32) -> u16 {
    high_word::<M>(widen_finite::<E, M>(bits))
}

const fn high_word<const M: u32>(bits: u32) -> u16 {
    assert!(
        M <= 7,
        "high-word widening requires at most seven mantissa bits"
    );
    // The shift proves the discarded low word is zero for the admitted source
    // widths; narrowing to u16 intentionally retains the remaining high word.
    (bits >> 16) as u16
}

const fn widen_with<P: SpecialValues, const E: u32, const M: u32>(bits: u32) -> u32 {
    let sign = (bits >> (E + M)) & 1;
    let exp = (bits >> M) & ((1 << E) - 1);
    let mant = bits & ((1 << M) - 1);
    let bias: i32 = (1 << (E - 1)) - 1;
    let f32_sign = sign << 31;
    let mant_align = F32_MANT_BITS - M;

    if exp == (1 << E) - 1 {
        if P::HAS_INFINITY && mant == 0 {
            f32_sign | (F32_EXP_ALL_ONES << F32_MANT_BITS)
        } else {
            // A zero-width mantissa or a finite-only top exponent can carry no
            // payload, so force the canonical quiet-NaN bit.
            let payload = mant << mant_align;
            let payload = if payload == 0 {
                1 << (F32_MANT_BITS - 1)
            } else {
                payload
            };
            f32_sign | (F32_EXP_ALL_ONES << F32_MANT_BITS) | payload
        }
    } else if exp == 0 {
        if mant == 0 {
            f32_sign // signed zero
        } else {
            // Subnormal: shift the mantissa left until the implicit leading 1
            // reaches bit `M`, decrementing the exponent per shift.
            let mut m = mant;
            let mut shift: i32 = 0;
            while (m & (1 << M)) == 0 {
                m <<= 1;
                shift += 1;
            }
            let f32_exp = F32_EXP_BIAS - bias - shift + 1;
            if f32_exp > 0 {
                // The value lands in `f32`'s normal range (always true for
                // `binary16`; `bfloat16` shares `f32`'s exponent range and can
                // instead underflow into `f32` subnormals — handled below).
                let m = m & ((1 << M) - 1); // drop the now-explicit leading 1
                f32_sign | ((f32_exp as u32) << F32_MANT_BITS) | (m << mant_align)
            } else {
                // `f32` subnormal output: the whole significand (implicit 1
                // included) becomes the fraction, scaled so the smallest `f32`
                // subnormal is 2⁻¹⁴⁹. Exact for every eunomia format — reduced
                // subnormals never fall below 2⁻¹⁴⁹ — so the shift is
                // non-negative and loses no bits.
                let scale = F32_MANT_BITS as i32 - 1 + f32_exp - M as i32;
                let frac = if scale >= 0 {
                    m << scale
                } else {
                    round_to_nearest_even(m, (-scale) as u32)
                };
                f32_sign | frac
            }
        }
    } else {
        // Normal: rebias the exponent and left-align the mantissa.
        let f32_exp = (exp as i32 - bias + F32_EXP_BIAS) as u32;
        f32_sign | (f32_exp << F32_MANT_BITS) | (mant << mant_align)
    }
}

/// Narrow an `f32` bit pattern to the reduced `(E, M)` format, rounding to
/// nearest with ties to even. The result occupies the low `1 + E + M` bits.
///
/// Handles the full IEEE domain: normals, `f32` subnormals (significant for
/// `bfloat16`, which shares `f32`'s exponent range), reduced subnormals,
/// gradual underflow to signed zero, overflow to infinity, and NaN payload
/// propagation.
#[inline]
#[must_use]
pub const fn narrow<const E: u32, const M: u32>(f32_bits: u32) -> u32 {
    narrow_with::<Ieee, E, M>(f32_bits)
}

/// Narrow into a finite-only format that reserves its top exponent for NaN.
#[inline]
#[must_use]
pub(crate) const fn narrow_finite<const E: u32, const M: u32>(f32_bits: u32) -> u32 {
    narrow_with::<Finite, E, M>(f32_bits)
}

const fn narrow_with<P: SpecialValues, const E: u32, const M: u32>(f32_bits: u32) -> u32 {
    let sign = (f32_bits >> 31) & 1;
    let f32_exp = ((f32_bits >> F32_MANT_BITS) & F32_EXP_ALL_ONES) as i32;
    let f32_mant = f32_bits & ((1 << F32_MANT_BITS) - 1);
    let bias: i32 = (1 << (E - 1)) - 1;
    let exp_all_ones: u32 = (1 << E) - 1;
    let out_sign = sign << (E + M);
    let mant_align = F32_MANT_BITS - M;
    let max_finite = out_sign | ((exp_all_ones - 1) << M) | ((1 << M) - 1);

    // Infinity / NaN mapping follows the selected format contract.
    if f32_exp == F32_EXP_ALL_ONES as i32 {
        if f32_mant == 0 {
            return if P::HAS_INFINITY {
                out_sign | (exp_all_ones << M)
            } else {
                max_finite
            };
        }
        if !P::HAS_INFINITY {
            return out_sign | (exp_all_ones << M) | ((1 << M) - 1);
        }
        // Right-align the payload; force a nonzero mantissa so a NaN never
        // collapses into the infinity encoding.
        let payload = f32_mant >> mant_align;
        let mant = if payload == 0 { 1 } else { payload };
        return out_sign | (exp_all_ones << M) | mant;
    }

    // Normalize the source into `(unbiased exponent, 24-bit significand with the
    // implicit leading 1 at bit 23)`. `f32` subnormals carry no implicit 1, so
    // renormalize them; their exponent is `1 - 127`, not `0 - 127`.
    let (unbiased, significand) = if f32_exp == 0 {
        if f32_mant == 0 {
            return out_sign; // signed zero
        }
        let shift = f32_mant.leading_zeros() - (u32::BITS - 1 - F32_MANT_BITS);
        (1 - F32_EXP_BIAS - shift as i32, f32_mant << shift)
    } else {
        (f32_exp - F32_EXP_BIAS, f32_mant | (1 << F32_MANT_BITS))
    };

    let out_exp = unbiased + bias;

    if out_exp >= exp_all_ones as i32 {
        return if P::HAS_INFINITY {
            out_sign | (exp_all_ones << M)
        } else {
            max_finite
        };
    }

    if out_exp <= 0 {
        // Reduced subnormal or underflow. Drop the normal `mant_align` bits plus
        // the `1 - out_exp` denormalization shift, rounding to nearest even. A
        // carry into bit `M` promotes the result to the smallest normal, which
        // the contiguous layout encodes correctly.
        let drop = mant_align as i32 + (1 - out_exp);
        if drop >= u32::BITS as i32 {
            return out_sign;
        }
        return out_sign | round_to_nearest_even(significand, drop as u32);
    }

    // Normal: round the significand down to `M` fraction bits. `reduced_sig` is
    // `2^M + mantissa` (or `2^(M+1)` on a rounding carry); subtracting the
    // implicit `2^M` and adding the exponent field lets a carry ripple into the
    // exponent (and, at the top, into the infinity encoding) automatically.
    let reduced_sig = round_to_nearest_even(significand, mant_align);
    let encoded = out_sign | (((out_exp as u32) << M) + (reduced_sig - (1 << M)));
    if !P::HAS_INFINITY && ((encoded >> M) & exp_all_ones) == exp_all_ones {
        max_finite
    } else {
        encoded
    }
}
