//! Differential verification of the native reduced-precision conversion kernel
//! ([`eunomia::convert`]) against the `half` crate — the authoritative
//! `binary16`/`bfloat16` reference this kernel replaces.
//!
//! Widening is verified **exhaustively** (all 2¹⁶ bit patterns) and bit-exactly.
//! Narrowing is verified by (a) an exhaustive round-trip identity over every
//! finite value and (b) a near-exhaustive `f32` differential sweep covering
//! every exponent regime and every round/guard/sticky rounding decision. NaN
//! outputs are checked for NaN-ness (payload bits are implementation-defined),
//! everything else for exact bit equality.

use eunomia::convert::{narrow, widen};

/// Assert `widen::<E, M>(bits)` reproduces the reference `f32` bit-for-bit
/// (NaN → NaN, since NaN payloads are not contractually fixed).
fn assert_widen<const E: u32, const M: u32>(bits: u32, reference: f32) {
    let got = f32::from_bits(widen::<E, M>(bits));
    if reference.is_nan() {
        assert!(
            got.is_nan(),
            "widen::<{E},{M}>({bits:#06x}) = {got} expected NaN",
        );
    } else {
        assert_eq!(
            got.to_bits(),
            reference.to_bits(),
            "widen::<{E},{M}>({bits:#06x}) = {got} ({:#010x}) expected {reference} ({:#010x})",
            got.to_bits(),
            reference.to_bits(),
        );
    }
}

/// Assert `narrow::<E, M>(x)` reproduces the reference reduced bit pattern
/// exactly (NaN input → any NaN encoding: top exponent all ones, mantissa ≠ 0).
fn assert_narrow<const E: u32, const M: u32>(x: f32, reference_bits: u32) {
    let got = narrow::<E, M>(x.to_bits());
    if x.is_nan() {
        let exp_all_ones = (1u32 << E) - 1;
        assert_eq!(
            (got >> M) & exp_all_ones,
            exp_all_ones,
            "narrow::<{E},{M}>(NaN {:#010x}) exponent not all-ones: {got:#06x}",
            x.to_bits(),
        );
        assert_ne!(
            got & ((1 << M) - 1),
            0,
            "narrow::<{E},{M}>(NaN {:#010x}) mantissa zero (would be Inf): {got:#06x}",
            x.to_bits(),
        );
    } else {
        assert_eq!(
            got,
            reference_bits,
            "narrow::<{E},{M}>({x}, {:#010x}) = {got:#06x} expected {reference_bits:#06x}",
            x.to_bits(),
        );
    }
}

#[test]
fn widen_binary16_matches_half_exhaustively() {
    for bits in 0u32..=0xFFFF {
        let reference = half::f16::from_bits(bits as u16).to_f32();
        assert_widen::<5, 10>(bits, reference);
    }
}

#[test]
fn widen_bfloat16_matches_half_exhaustively() {
    for bits in 0u32..=0xFFFF {
        let reference = half::bf16::from_bits(bits as u16).to_f32();
        assert_widen::<8, 7>(bits, reference);
    }
}

#[test]
fn narrow_binary16_round_trips_every_finite_value() {
    for bits in 0u32..=0xFFFF {
        let value = half::f16::from_bits(bits as u16);
        if value.is_nan() {
            continue; // NaN payload is not preserved bit-for-bit by design
        }
        let widened = f32::from_bits(widen::<5, 10>(bits));
        assert_eq!(
            narrow::<5, 10>(widened.to_bits()),
            bits,
            "binary16 round-trip failed for {bits:#06x} ({value})",
        );
    }
}

#[test]
fn narrow_bfloat16_round_trips_every_finite_value() {
    for bits in 0u32..=0xFFFF {
        let value = half::bf16::from_bits(bits as u16);
        if value.is_nan() {
            continue;
        }
        let widened = f32::from_bits(widen::<8, 7>(bits));
        assert_eq!(
            narrow::<8, 7>(widened.to_bits()),
            bits,
            "bfloat16 round-trip failed for {bits:#06x} ({value})",
        );
    }
}

/// Sweep every `f32` exponent, every combination of the kept + guard + round
/// mantissa bits, a sticky bit, and both signs — the complete set of inputs that
/// determine a rounding decision — comparing `narrow` to `half` bit-for-bit.
fn sweep_narrow_against_half<const E: u32, const M: u32>(reference: impl Fn(f32) -> u32) {
    // Kept mantissa + guard + round bits, placed at the top of the f32 field.
    let top_bits = M + 2;
    let top_shift = 23 - top_bits;
    for sign in [0u32, 1] {
        for exp in 0u32..=0xFF {
            for top in 0u32..(1 << top_bits) {
                for sticky in [0u32, 1] {
                    let mant = (top << top_shift) | sticky;
                    let x = f32::from_bits((sign << 31) | (exp << 23) | mant);
                    assert_narrow::<E, M>(x, reference(x));
                }
            }
        }
    }
}

#[test]
fn narrow_binary16_matches_half_across_rounding_sweep() {
    sweep_narrow_against_half::<5, 10>(|x| half::f16::from_f32(x).to_bits() as u32);
}

#[test]
fn narrow_bfloat16_matches_half_across_rounding_sweep() {
    sweep_narrow_against_half::<8, 7>(|x| half::bf16::from_f32(x).to_bits() as u32);
}

#[test]
fn narrow_binary16_pins_known_values_and_rounding() {
    let f16 = |x: f32| narrow::<5, 10>(x.to_bits());
    // Exact representables.
    assert_eq!(f16(0.0), 0x0000);
    assert_eq!(f16(-0.0), 0x8000);
    assert_eq!(f16(1.0), 0x3C00);
    assert_eq!(f16(-2.0), 0xC000);
    // Largest finite binary16.
    assert_eq!(f16(65504.0), 0x7BFF);
    // Overflow beyond the finite range saturates to infinity.
    assert_eq!(f16(70000.0), 0x7C00);
    assert_eq!(f16(f32::INFINITY), 0x7C00);
    assert_eq!(f16(f32::NEG_INFINITY), 0xFC00);
    // Ties-to-even: 1 + 2^-11 sits exactly between 1.0 (0x3C00, even) and
    // 1 + 2^-10 (0x3C01, odd) → rounds to the even neighbour, 1.0.
    assert_eq!(f16(1.0 + f32::from_bits(0x3A00_0000)), 0x3C00);
    // 1 + 3·2^-11 sits exactly between 0x3C01 (odd) and 0x3C02 (even) → 0x3C02.
    assert_eq!(f16(1.0 + 3.0 * f32::from_bits(0x3A00_0000)), 0x3C02);
    // Smallest positive subnormal: 2^-24.
    assert_eq!(f16(f32::from_bits(0x3380_0000)), 0x0001);
    // Half of the smallest subnormal rounds to zero (tie to even → 0).
    assert_eq!(f16(f32::from_bits(0x3300_0000)), 0x0000);
}

#[test]
fn widen_binary16_pins_known_values() {
    let to_f32 = |bits: u32| f32::from_bits(widen::<5, 10>(bits));
    assert_eq!(to_f32(0x0000), 0.0);
    assert_eq!(to_f32(0x8000), -0.0);
    assert_eq!(to_f32(0x3C00), 1.0);
    assert_eq!(to_f32(0x7BFF), 65504.0);
    assert_eq!(to_f32(0x0001), f32::from_bits(0x3380_0000)); // 2^-24 subnormal
    assert!(to_f32(0x7C00).is_infinite() && to_f32(0x7C00) > 0.0);
    assert!(to_f32(0xFC00).is_infinite() && to_f32(0xFC00) < 0.0);
    assert!(to_f32(0x7E00).is_nan());
}
