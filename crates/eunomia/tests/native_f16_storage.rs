//! E-025: `F16`/`Bf16` are native `u16`-backed (no `half` in the wrapper path).
//! Verify the re-back is behavior-preserving vs the `half` reference and that
//! `PartialEq`/`PartialOrd` are **float-semantic** (via `f32`), not bitwise.

use eunomia::{Bf16, F16};

#[test]
fn f16_widening_matches_half_exhaustively() {
    for bits in 0u16..=u16::MAX {
        let native = F16(bits).to_f32();
        let reference = half::f16::from_bits(bits).to_f32();
        if reference.is_nan() {
            assert!(native.is_nan(), "F16({bits:#06x}) expected NaN");
        } else {
            assert_eq!(native.to_bits(), reference.to_bits(), "F16({bits:#06x})");
        }
    }
}

#[test]
fn bf16_widening_matches_half_exhaustively() {
    for bits in 0u16..=u16::MAX {
        let native = Bf16(bits).to_f32();
        let reference = half::bf16::from_bits(bits).to_f32();
        if reference.is_nan() {
            assert!(native.is_nan(), "Bf16({bits:#06x}) expected NaN");
        } else {
            assert_eq!(native.to_bits(), reference.to_bits(), "Bf16({bits:#06x})");
        }
    }
}

#[test]
fn f16_narrowing_matches_half_across_f32_space() {
    // Sweep every exponent and the rounding-relevant high mantissa bits + sticky.
    for sign in [0u32, 1] {
        for exp in 0u32..=0xFF {
            for top in 0u32..(1 << 12) {
                for sticky in [0u32, 1] {
                    let x = f32::from_bits((sign << 31) | (exp << 23) | (top << 11) | sticky);
                    let native = F16::from_f32(x).0;
                    let reference = half::f16::from_f32(x).to_bits();
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
fn f16_from_f64_is_exact_via_f32() {
    // Double rounding f64 -> f32 -> f16 equals direct f64 -> f16 (f32's 24 bits
    // ≥ 2·11 + 2); check against half's direct `from_f64`.
    for bits in 0u16..=u16::MAX {
        let value = f64::from(F16(bits).to_f32());
        let native = F16::from_f64(value).0;
        let reference = half::f16::from_f64(value).to_bits();
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
fn f16_bulk_slice_conversion_matches_half_and_scalar() {
    // Widen every `f16` pattern (F16C path on an F16C host) vs the `half` oracle.
    let all: Vec<F16> = (0u16..=u16::MAX).map(F16).collect();
    let mut widened = vec![0.0f32; all.len()];
    F16::widen_slice(&all, &mut widened);
    for (bits, &out) in widened.iter().enumerate() {
        let reference = half::f16::from_bits(bits as u16).to_f32();
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

    // Narrow a rounding-relevant `f32` sweep vs `half` and the scalar path. The
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
            assert_eq!(out.0, half::f16::from_f32(x).to_bits(), "narrow_slice({x})");
            assert_eq!(out.0, F16::from_f32(x).0, "narrow_slice vs scalar ({x})");
        }
    }
}
