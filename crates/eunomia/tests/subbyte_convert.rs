//! Analytical contracts for Eunomia's native sub-byte floating-point formats.

use eunomia::{
    unpack_bf4_to_bf16, unpack_bf4_to_bf16_packed, unpack_bf8_to_bf16, unpack_f4_to_f32,
    unpack_f4_to_f32_packed, unpack_f8_to_f32, Bf16, Bf4, Bf8, NumericElement, F32, F4, F8,
};

fn assert_signed_zero(value: f32, negative: bool) {
    assert_eq!(value.to_bits(), if negative { 0x8000_0000 } else { 0 });
}

#[test]
fn subbyte_widening_matches_declared_layouts() {
    assert_signed_zero(Bf8(0x00).to_f32(), false);
    assert_signed_zero(Bf8(0x80).to_f32(), true);
    assert_eq!(Bf8(0x01).to_f32(), 2.0f32.powi(-16));
    assert_eq!(Bf8(0x04).to_f32(), 2.0f32.powi(-14));
    assert_eq!(Bf8(0x3C).to_f32(), 1.0);
    assert_eq!(Bf8(0x7B).to_f32(), 57_344.0);
    assert_eq!(Bf8(0x7C).to_f32(), f32::INFINITY);
    assert!(Bf8(0x7D).to_f32().is_nan());

    assert_signed_zero(Bf4(0x00).to_f32(), false);
    assert_signed_zero(Bf4(0x08).to_f32(), true);
    assert_eq!(Bf4(0x01).to_f32(), 0.5);
    assert_eq!(Bf4(0x02).to_f32(), 1.0);
    assert_eq!(Bf4(0x03).to_f32(), 1.5);
    assert_eq!(Bf4(0x05).to_f32(), 3.0);
    assert!(Bf4(0x06).to_f32().is_nan());
    assert!(Bf4(0x07).to_f32().is_nan());

    assert_signed_zero(F8(0x00).to_f32(), false);
    assert_signed_zero(F8(0x80).to_f32(), true);
    assert_eq!(F8(0x01).to_f32(), 2.0f32.powi(-9));
    assert_eq!(F8(0x08).to_f32(), 2.0f32.powi(-6));
    assert_eq!(F8(0x38).to_f32(), 1.0);
    assert_eq!(F8(0x77).to_f32(), 240.0);
    assert!(F8(0x78).to_f32().is_nan());
    assert!(F8(0x7F).to_f32().is_nan());

    assert_signed_zero(F4(0x00).to_f32(), false);
    assert_signed_zero(F4(0x08).to_f32(), true);
    assert_eq!(F4(0x01).to_f32(), 0.25);
    assert_eq!(F4(0x03).to_f32(), 1.0);
    assert_eq!(F4(0x06).to_f32(), 8.0);
    assert!(F4(0x07).to_f32().is_nan());
}

#[test]
fn subbyte_narrowing_rounds_to_nearest_ties_to_even() {
    assert_eq!(Bf8::from_f32(1.125), Bf8(0x3C));
    assert_eq!(Bf8::from_f32(1.375), Bf8(0x3E));

    assert_eq!(Bf4::from_f32(1.25), Bf4(0x02));
    assert_eq!(Bf4::from_f32(1.75), Bf4(0x04));

    assert_eq!(F8::from_f32(1.0625), F8(0x38));
    assert_eq!(F8::from_f32(1.1875), F8(0x3A));

    assert_eq!(F4::from_f32(0.75), F4(0x03));
    assert_eq!(F4::from_f32(1.5), F4(0x04));
}

#[test]
fn finite_only_formats_saturate_and_canonicalize_nan() {
    assert_eq!(Bf4::from_f32(f32::INFINITY), Bf4(0x05));
    assert_eq!(Bf4::from_f32(f32::NEG_INFINITY), Bf4(0x0D));
    assert_eq!(Bf4::from_f32(f32::NAN), Bf4(0x07));

    assert_eq!(F8::from_f32(f32::INFINITY), F8(0x77));
    assert_eq!(F8::from_f32(f32::NEG_INFINITY), F8(0xF7));
    assert_eq!(F8::from_f32(f32::NAN), F8(0x7F));

    assert_eq!(F4::from_f32(f32::INFINITY), F4(0x06));
    assert_eq!(F4::from_f32(f32::NEG_INFINITY), F4(0x0E));
    assert_eq!(F4::from_f32(f32::NAN), F4(0x07));
}

#[test]
fn every_finite_subbyte_encoding_round_trips() {
    for bits in 0u8..=u8::MAX {
        if bits & 0x7C != 0x7C {
            assert_eq!(Bf8::from_f32(Bf8(bits).to_f32()), Bf8(bits));
        }
        if bits & 0x78 != 0x78 {
            assert_eq!(F8::from_f32(F8(bits).to_f32()), F8(bits));
        }
    }

    for bits in 0u8..16 {
        if bits & 0x06 != 0x06 {
            assert_eq!(Bf4::from_f32(Bf4(bits).to_f32()), Bf4(bits));
        }
        if bits & 0x07 != 0x07 {
            assert_eq!(F4::from_f32(F4(bits).to_f32()), F4(bits));
        }
    }
}

#[test]
fn numeric_constants_match_subbyte_value_contracts() {
    assert_eq!(Bf8::MIN_VALUE.to_f32(), f32::NEG_INFINITY);
    assert_eq!(Bf8::MAX_VALUE.to_f32(), f32::INFINITY);

    assert_eq!(Bf4::INFINITY, Bf4(0x05));
    assert_eq!(Bf4::MIN_VALUE, Bf4(0x0D));
    assert_eq!(Bf4::MAX_VALUE, Bf4(0x05));

    assert_eq!(F8::INFINITY, F8(0x77));
    assert_eq!(F8::MIN_VALUE, F8(0xF7));
    assert_eq!(F8::MAX_VALUE, F8(0x77));

    assert_eq!(F4::INFINITY, F4(0x06));
    assert_eq!(F4::MIN_VALUE, F4(0x0E));
    assert_eq!(F4::MAX_VALUE, F4(0x06));
}

fn assert_every_rounding_boundary(
    max_finite_bits: u8,
    sign_mask: u8,
    widen: impl Fn(u8) -> f32,
    narrow: impl Fn(f32) -> u8,
) {
    for lower_bits in 0..max_finite_bits {
        let upper_bits = lower_bits + 1;
        let lower = widen(lower_bits);
        let upper = widen(upper_bits);
        let midpoint = (lower + upper) * 0.5;
        let below = f32::from_bits(midpoint.to_bits() - 1);
        let above = f32::from_bits(midpoint.to_bits() + 1);
        let tie = if lower != 0.0 && upper == lower * 2.0 {
            // For zero-mantissa formats, adjacent normal values straddle a
            // binade: at the lower value's quantum the upper significand is
            // even and the lower significand is odd.
            upper_bits
        } else if lower_bits & 1 == 0 {
            lower_bits
        } else {
            upper_bits
        };

        assert_eq!(narrow(below), lower_bits);
        assert_eq!(narrow(midpoint), tie);
        assert_eq!(narrow(above), upper_bits);
        assert_eq!(narrow(-below), sign_mask | lower_bits);
        assert_eq!(narrow(-midpoint), sign_mask | tie);
        assert_eq!(narrow(-above), sign_mask | upper_bits);
    }
}

#[test]
fn every_subbyte_rounding_boundary_uses_ties_to_even() {
    assert_every_rounding_boundary(
        0x7B,
        0x80,
        |bits| Bf8(bits).to_f32(),
        |value| Bf8::from_f32(value).0,
    );
    assert_every_rounding_boundary(
        0x05,
        0x08,
        |bits| Bf4(bits).to_f32(),
        |value| Bf4::from_f32(value).0,
    );
    assert_every_rounding_boundary(
        0x77,
        0x80,
        |bits| F8(bits).to_f32(),
        |value| F8::from_f32(value).0,
    );
    assert_every_rounding_boundary(
        0x06,
        0x08,
        |bits| F4(bits).to_f32(),
        |value| F4::from_f32(value).0,
    );
}

fn exact_high_word(value: f32) -> u16 {
    u16::try_from(value.to_bits() >> 16)
        .expect("invariant: shifting a binary32 pattern leaves exactly 16 bits")
}

fn assert_bf16_bits_equal(left: &[Bf16], right: &[Bf16]) {
    assert_eq!(left.len(), right.len());
    for (left, right) in left.iter().zip(right) {
        assert_eq!(left.0, right.0);
    }
}

#[test]
fn packed_dispatch_matches_scalar_conversion_for_every_encoding() {
    let bf8: Vec<_> = (u8::MIN..=u8::MAX).map(Bf8).collect();
    let mut unpacked_bf8 = vec![Bf16::default(); bf8.len()];
    unpack_bf8_to_bf16(&bf8, &mut unpacked_bf8);
    for (source, result) in bf8.iter().zip(&unpacked_bf8) {
        assert_eq!(result.0, exact_high_word(source.to_f32()));
    }

    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            let mut direct = vec![Bf16::default(); bf8.len()];
            // SAFETY: AVX2 is detected at runtime and both buffers have equal
            // lengths, satisfying the intrinsic kernel's bounds contract.
            unsafe { eunomia::unsafe_intrinsics::avx2::unpack_bf8_to_bf16(&bf8, &mut direct) };
            assert_bf16_bits_equal(&direct, &unpacked_bf8);
        }
        if std::arch::is_x86_feature_detected!("avx512f")
            && std::arch::is_x86_feature_detected!("avx512bw")
            && std::arch::is_x86_feature_detected!("avx512vl")
        {
            let mut direct = vec![Bf16::default(); bf8.len()];
            // SAFETY: the three target features are detected at runtime and
            // both buffers have equal lengths.
            unsafe { eunomia::unsafe_intrinsics::avx512::unpack_bf8_to_bf16(&bf8, &mut direct) };
            assert_bf16_bits_equal(&direct, &unpacked_bf8);
        }
    }

    let bf4: Vec<_> = (0..16).map(Bf4).collect();
    let mut unpacked_bf4 = vec![Bf16::default(); bf4.len()];
    unpack_bf4_to_bf16(&bf4, &mut unpacked_bf4);
    for (source, result) in bf4.iter().zip(&unpacked_bf4) {
        assert_eq!(result.0, exact_high_word(source.to_f32()));
    }

    let f8: Vec<_> = (u8::MIN..=u8::MAX).map(F8).collect();
    let mut unpacked_f8 = vec![F32::default(); f8.len()];
    unpack_f8_to_f32(&f8, &mut unpacked_f8);
    for (source, result) in f8.iter().zip(&unpacked_f8) {
        assert_eq!(result.0.to_bits(), source.to_f32().to_bits());
    }

    let f4: Vec<_> = (0..16).map(F4).collect();
    let mut unpacked_f4 = vec![F32::default(); f4.len()];
    unpack_f4_to_f32(&f4, &mut unpacked_f4);
    for (source, result) in f4.iter().zip(&unpacked_f4) {
        assert_eq!(result.0.to_bits(), source.to_f32().to_bits());
    }

    let packed: Vec<_> = (u8::MIN..=u8::MAX).collect();
    let mut unpacked_bf4_pairs = vec![Bf16::default(); packed.len() * 2];
    unpack_bf4_to_bf16_packed(&packed, &mut unpacked_bf4_pairs);
    let mut unpacked_f4_pairs = vec![F32::default(); packed.len() * 2];
    unpack_f4_to_f32_packed(&packed, &mut unpacked_f4_pairs);
    for (index, byte) in packed.iter().copied().enumerate() {
        let (low_bf4, high_bf4) = Bf4::unpack_pair(byte);
        assert_eq!(
            unpacked_bf4_pairs[index * 2].0,
            exact_high_word(low_bf4.to_f32())
        );
        assert_eq!(
            unpacked_bf4_pairs[index * 2 + 1].0,
            exact_high_word(high_bf4.to_f32())
        );

        let (low_f4, high_f4) = F4::unpack_pair(byte);
        assert_eq!(
            unpacked_f4_pairs[index * 2].0.to_bits(),
            low_f4.to_f32().to_bits()
        );
        assert_eq!(
            unpacked_f4_pairs[index * 2 + 1].0.to_bits(),
            high_f4.to_f32().to_bits()
        );
    }
}
