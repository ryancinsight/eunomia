//! E-025: `F16`/`Bf16` are native `u16`-backed.
//! Verify the re-back against an independent IEEE reference and ensure that
//! `PartialEq`/`PartialOrd` are **float-semantic** (via `f32`), not bitwise.

mod support;

use eunomia::{Bf16, F16};
use support::{is_nan, narrow as reference_narrow, widen as reference_widen};

#[test]
fn native_reduced_precision_bit_patterns_round_trip_exactly() {
    for bits in u16::MIN..=u16::MAX {
        assert_eq!(F16::from_bits(bits).to_bits(), bits);
        assert_eq!(Bf16::from_bits(bits).to_bits(), bits);
    }

    assert_eq!(F16::from_bits(0x8000).to_bits(), 0x8000);
    assert_eq!(Bf16::from_bits(0x8000).to_bits(), 0x8000);
    assert_eq!(F16::from_bits(0x7E55).to_bits(), 0x7E55);
    assert_eq!(Bf16::from_bits(0x7FC5).to_bits(), 0x7FC5);
}

#[test]
fn f16_widening_matches_ieee_reference_exhaustively() {
    for bits in 0u16..=u16::MAX {
        let native = F16(bits).to_f32();
        let reference = f32::from_bits(reference_widen::<5, 10>(u32::from(bits)));
        if is_nan::<5, 10>(u32::from(bits)) {
            assert!(native.is_nan(), "F16({bits:#06x}) expected NaN");
        } else {
            assert_eq!(native.to_bits(), reference.to_bits(), "F16({bits:#06x})");
        }
    }
}

#[test]
fn bf16_widening_matches_ieee_reference_exhaustively() {
    for bits in 0u16..=u16::MAX {
        let native = Bf16(bits).to_f32();
        let reference = f32::from_bits(reference_widen::<8, 7>(u32::from(bits)));
        if is_nan::<8, 7>(u32::from(bits)) {
            assert!(native.is_nan(), "Bf16({bits:#06x}) expected NaN");
        } else {
            assert_eq!(native.to_bits(), reference.to_bits(), "Bf16({bits:#06x})");
        }
    }
}

#[test]
fn f16_narrowing_matches_ieee_reference_across_f32_space() {
    // Sweep every exponent and the rounding-relevant high mantissa bits + sticky.
    for sign in [0u32, 1] {
        for exp in 0u32..=0xFF {
            for top in 0u32..(1 << 12) {
                for sticky in [0u32, 1] {
                    let x = f32::from_bits((sign << 31) | (exp << 23) | (top << 11) | sticky);
                    let native = F16::from_f32(x).0;
                    let reference = reference_narrow::<5, 10>(x.to_bits()) as u16;
                    if x.is_nan() {
                        // NaN → some NaN encoding (exp all ones, mantissa ≠ 0).
                        assert_eq!(native & 0x7C00, 0x7C00);
                        assert_ne!(native & 0x03FF, 0);
                    } else {
                        assert_eq!(native, reference, "F16::from_f32({x})");
                    }
                }
            }
        }
    }
}

#[test]
fn f16_from_f64_of_representable_values_is_exact() {
    // Every input is constructed from an F16 value, so its f32 conversion is
    // exact and the format-level reference can verify the narrowing result.
    for bits in 0u16..=u16::MAX {
        let value = f64::from(F16(bits).to_f32());
        let native = F16::from_f64(value).0;
        if value.is_nan() {
            assert!(
                F16(native).to_f32().is_nan(),
                "F16::from_f64 NaN for {bits:#06x}"
            );
            continue;
        }
        let reference = reference_narrow::<5, 10>((value as f32).to_bits()) as u16;
        assert_eq!(native, reference, "F16::from_f64 for {bits:#06x}");
    }
}

#[test]
fn partial_eq_is_float_semantic_not_bitwise() {
    // +0 and -0 differ in bits but are equal as floats.
    assert_eq!(F16(0x0000), F16(0x8000));
    assert_eq!(Bf16(0x0000), Bf16(0x8000));
    // NaN is never equal to itself (bitwise would say equal).
    assert_ne!(F16::NAN, F16::NAN);
    assert_ne!(Bf16::NAN, Bf16::NAN);
    // Distinct finite values compare as their float values.
    assert_eq!(F16::ONE, F16::from_f32(1.0));
    assert_ne!(F16::ONE, F16::from_f32(2.0));
}

#[test]
fn partial_ord_is_float_semantic_not_bitwise() {
    // Negative values have the high bit set, so a bitwise `u16` order would rank
    // them *above* positives. Float order must rank `-2 < 1`.
    let neg_two = F16::from_f32(-2.0);
    let one = F16::from_f32(1.0);
    assert!(neg_two < one);
    assert!(one > neg_two);
    assert!(F16::NEG_INFINITY < F16::from_f32(-1e4));
    assert!(F16::from_f32(1e4) < F16::INFINITY);
    // NaN is unordered.
    assert_eq!(F16::NAN.partial_cmp(&F16::ONE), None);

    let neg = Bf16::from_f32(-3.0);
    let pos = Bf16::from_f32(2.0);
    assert!(neg < pos);
}

#[test]
fn f16_bulk_slice_conversion_matches_ieee_reference_and_scalar() {
    // Widen every `f16` pattern (F16C path on an F16C host) vs the reference.
    let all: Vec<F16> = (0u16..=u16::MAX).map(F16).collect();
    let mut widened = vec![0.0f32; all.len()];
    F16::widen_slice(&all, &mut widened);
    for (bits, &out) in widened.iter().enumerate() {
        let reference = f32::from_bits(reference_widen::<5, 10>(bits as u32));
        if reference.is_nan() {
            assert!(out.is_nan(), "widen_slice[{bits:#06x}]");
        } else {
            assert_eq!(
                out.to_bits(),
                reference.to_bits(),
                "widen_slice[{bits:#06x}]"
            );
        }
    }

    // Narrow a rounding-relevant `f32` sweep vs the reference and scalar path. The
    // trailing specials make the length a non-multiple of 8, exercising the
    // vector kernel's scalar remainder.
    let mut sweep: Vec<f32> = Vec::new();
    for exp in 0u32..=0xFF {
        for top in 0u32..(1 << 8) {
            sweep.push(f32::from_bits((exp << 23) | (top << 15)));
            sweep.push(f32::from_bits((1 << 31) | (exp << 23) | (top << 15)));
        }
    }
    sweep.push(f32::INFINITY);
    sweep.push(f32::NEG_INFINITY);
    sweep.push(f32::from_bits(0x0000_0001)); // smallest positive subnormal

    let mut narrowed = vec![F16::default(); sweep.len()];
    F16::narrow_slice(&sweep, &mut narrowed);
    for (&x, out) in sweep.iter().zip(&narrowed) {
        if x.is_nan() {
            assert_eq!(out.0 & 0x7C00, 0x7C00);
            assert_ne!(out.0 & 0x03FF, 0);
        } else {
            assert_eq!(
                out.0,
                reference_narrow::<5, 10>(x.to_bits()) as u16,
                "narrow_slice({x})"
            );
            assert_eq!(out.0, F16::from_f32(x).0, "narrow_slice vs scalar ({x})");
        }
    }
}

#[test]
fn bf16_bulk_slice_conversion_matches_ieee_reference_and_kernel() {
    // Widen every bfloat16 pattern (a shift) vs the reference.
    let all: Vec<Bf16> = (0u16..=u16::MAX).map(Bf16).collect();
    let mut widened = vec![0.0f32; all.len()];
    Bf16::widen_slice(&all, &mut widened);
    for (bits, &out) in widened.iter().enumerate() {
        let reference = f32::from_bits(reference_widen::<8, 7>(bits as u32));
        if reference.is_nan() {
            assert!(out.is_nan(), "widen_slice[{bits:#06x}]");
        } else {
            assert_eq!(
                out.to_bits(),
                reference.to_bits(),
                "widen_slice[{bits:#06x}]"
            );
        }
    }

    // Narrow an f32 sweep vs the reference AND the scalar kernel — confirming the fast
    // round-and-truncate is bit-identical to `narrow::<8, 7>`. Non-multiple-of-8
    // length exercises the tail.
    let mut sweep: Vec<f32> = Vec::new();
    for exp in 0u32..=0xFF {
        for top in 0u32..(1 << 9) {
            sweep.push(f32::from_bits((exp << 23) | (top << 14)));
            sweep.push(f32::from_bits((1 << 31) | (exp << 23) | (top << 14)));
        }
    }
    sweep.push(f32::INFINITY);
    sweep.push(f32::NEG_INFINITY);
    sweep.push(f32::from_bits(0x0000_0001));

    let mut narrowed = vec![Bf16::default(); sweep.len()];
    Bf16::narrow_slice(&sweep, &mut narrowed);
    for (&x, out) in sweep.iter().zip(&narrowed) {
        if x.is_nan() {
            assert_eq!(out.0 & 0x7F80, 0x7F80);
            assert_ne!(out.0 & 0x007F, 0);
        } else {
            assert_eq!(
                out.0,
                reference_narrow::<8, 7>(x.to_bits()) as u16,
                "narrow_slice({x})"
            );
            assert_eq!(out.0, Bf16::from_f32(x).0, "narrow_slice vs kernel ({x})");
        }
    }
}

#[test]
fn bf16_bulk_narrow_matches_scalar_path_on_adversarial_patterns() {
    // The bulk path (AVX2 where available) must be bit-identical to the
    // scalar path. One-element slices always take the scalar remainder, so
    // they are the differential oracle for the vector lanes; non-NaN lanes
    // additionally pin to the scalar kernel via `from_f32`.
    let patterns: [u32; 32] = [
        0x0000_0000, // +0
        0x8000_0000, // -0
        0x0000_0001, // smallest f32 subnormal
        0x0000_8000, // f32 subnormal at the bf16 rounding boundary
        0x0040_8000, // f32 subnormal, tie in bf16 subnormal range
        0x007F_FFFF, // largest f32 subnormal
        0x0080_0000, // smallest f32 normal
        0x3F80_0000, // 1.0
        0x3F80_7FFF, // just below the tie above 1.0
        0x3F80_8000, // exact tie, retained LSB even (rounds down)
        0x3F80_8001, // just above the tie (rounds up)
        0x3F81_8000, // exact tie, retained LSB odd (rounds up)
        0xBF80_8000, // negative tie
        0x7F7F_FFFF, // largest finite f32
        0xFF7F_FFFF, // most negative finite f32
        0x7F80_0000, // +inf
        0xFF80_0000, // -inf
        0x7F80_0001, // NaN, payload only below the retained bits
        0x7F80_8000, // NaN, payload at the truncation boundary
        0x7FC0_0000, // quiet NaN
        0xFFA0_0000, // negative NaN with retained payload
        0x7FFF_FFFF, // NaN, full payload
        0x0000_FFFF, // f32 subnormal, dense low bits
        0x4B00_0001, // large normal, low bit set
        0x3380_0000, // 2^-24
        0x33C0_8000, // small normal tie
        0x0080_8000, // smallest normals, tie
        0x8080_8000, // negative smallest-normal tie
        0x7F00_0000, // 2^127
        0x0080_0001, // smallest normal plus one ulp-bit
        0x3DCC_CCCD, // 0.1 bit pattern
        0xC248_0000, // -50.0
    ];
    // Rotate the pattern window through every bulk length 0..=17 so the
    // vector body and every scalar-tail length see each pattern class.
    for len in 0..=17usize {
        for rotation in 0..patterns.len() {
            let src: Vec<f32> = (0..len)
                .map(|k| f32::from_bits(patterns[(rotation + k) % patterns.len()]))
                .collect();
            let mut bulk = vec![Bf16::default(); len];
            Bf16::narrow_slice(&src, &mut bulk);
            for (k, (&x, out)) in src.iter().zip(&bulk).enumerate() {
                let mut scalar = [Bf16::default()];
                Bf16::narrow_slice(&[x], &mut scalar);
                assert_eq!(
                    out.0,
                    scalar[0].0,
                    "bulk vs scalar len={len} rotation={rotation} lane={k} bits={:#010x}",
                    x.to_bits()
                );
                if !x.is_nan() {
                    assert_eq!(out.0, Bf16::from_f32(x).0, "bulk vs kernel ({x})");
                }
            }
        }
    }
}
