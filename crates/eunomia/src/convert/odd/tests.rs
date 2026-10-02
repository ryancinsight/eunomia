//! `f64` narrowing rounds once, against an integer oracle that shares no code
//! with the conversion: the oracle reads the `f64` bit pattern and rounds its
//! exact significand straight onto the target format's grid, never through
//! `f32`.

mod oracle;

use super::odd_rounded;
use crate::traits::FloatElement;
use crate::types::{Bf16, Bf4, Bf8, F16, F4, F8};
use oracle::{
    displaced, pow2, Format, BF16_FORMAT, BF4_FORMAT, BF8_FORMAT, F16_FORMAT, F4_FORMAT,
    F64_FRACTION_BITS, F64_FRACTION_MASK, F8_FORMAT,
};

/// A format's own `from_f64`, through the trait consumers bind, as raw bits.
struct Converter {
    name: &'static str,
    format: Format,
    convert: fn(f64) -> u32,
    is_nan: fn(u32) -> bool,
}

fn converters() -> [Converter; 6] {
    [
        Converter {
            name: "F16",
            format: F16_FORMAT,
            convert: |x| u32::from(<F16 as FloatElement>::from_f64(x).0),
            is_nan: |bits| F16(u16::try_from(bits).expect("invariant: 16 bits")).is_nan(),
        },
        Converter {
            name: "Bf16",
            format: BF16_FORMAT,
            convert: |x| u32::from(<Bf16 as FloatElement>::from_f64(x).0),
            is_nan: |bits| Bf16(u16::try_from(bits).expect("invariant: 16 bits")).is_nan(),
        },
        Converter {
            name: "Bf8",
            format: BF8_FORMAT,
            convert: |x| u32::from(<Bf8 as FloatElement>::from_f64(x).0),
            is_nan: |bits| {
                Bf8(u8::try_from(bits).expect("invariant: 8 bits"))
                    .to_f32()
                    .is_nan()
            },
        },
        Converter {
            name: "F8",
            format: F8_FORMAT,
            convert: |x| u32::from(<F8 as FloatElement>::from_f64(x).0),
            is_nan: |bits| {
                F8(u8::try_from(bits).expect("invariant: 8 bits"))
                    .to_f32()
                    .is_nan()
            },
        },
        Converter {
            name: "Bf4",
            format: BF4_FORMAT,
            convert: |x| u32::from(<Bf4 as FloatElement>::from_f64(x).0),
            is_nan: |bits| {
                Bf4(u8::try_from(bits).expect("invariant: 4 bits"))
                    .to_f32()
                    .is_nan()
            },
        },
        Converter {
            name: "F4",
            format: F4_FORMAT,
            convert: |x| u32::from(<F4 as FloatElement>::from_f64(x).0),
            is_nan: |bits| {
                F4(u8::try_from(bits).expect("invariant: 4 bits"))
                    .to_f32()
                    .is_nan()
            },
        },
    ]
}

impl Converter {
    fn assert_rounds_like_the_oracle(&self, x: f64) {
        let got = (self.convert)(x);
        match self.format.round(x) {
            None => assert!(
                (self.is_nan)(got),
                "{}::from_f64({x:e}, {:#018x}) = {got:#06x}, expected NaN",
                self.name,
                x.to_bits()
            ),
            Some(expected) => assert_eq!(
                got,
                expected,
                "{}::from_f64({x:e}, {:#018x})",
                self.name,
                x.to_bits()
            ),
        }
    }
}

/// Every point of the format and every midpoint between neighbours (the
/// overflow midpoint included), each displaced by `offsets` units in the last
/// place of `f64`, in both signs. `stride` thins the grid.
fn assert_grid_rounds_like_the_oracle(converter: &Converter, offsets: &[i64], stride: usize) {
    let magnitudes = converter.format.magnitudes();
    for (index, pair) in magnitudes.windows(2).enumerate().step_by(stride) {
        let [point, next] = pair else {
            unreachable!("invariant: windows(2) yields pairs");
        };
        let midpoint = (point + next) / 2.0;
        assert!(
            *point < midpoint && midpoint < *next,
            "invariant: the midpoint is exact and strictly between, at {index}"
        );
        for anchor in [*point, midpoint] {
            for &offset in offsets {
                if anchor == 0.0 && offset <= 0 {
                    continue;
                }
                let x = displaced(anchor, offset);
                converter.assert_rounds_like_the_oracle(x);
                converter.assert_rounds_like_the_oracle(-x);
            }
        }
    }
}

/// One unit in the last place either side.
const NEAR: [i64; 3] = [-1, 0, 1];
/// Displacements around half and one unit of `f32`'s last place, where the
/// intermediate's own rounding flips.
const INTERMEDIATE_UNIT: [i64; 11] = [
    -(1 << 29),
    -(1 << 28) - 1,
    -(1 << 28),
    -(1 << 28) + 1,
    -1,
    0,
    1,
    (1 << 28) - 1,
    1 << 28,
    (1 << 28) + 1,
    1 << 29,
];

/// Midpoints and midpoints +- 1 ulp of `f64`, for every format: the cases a
/// round-to-nearest `f32` intermediate gets wrong.
#[test]
fn every_midpoint_and_its_neighbours_round_as_the_oracle_does() {
    for converter in converters() {
        assert_grid_rounds_like_the_oracle(&converter, &NEAR, 1);
    }
}

#[test]
fn midpoints_displaced_across_the_intermediate_unit_round_as_the_oracle_does() {
    for converter in converters() {
        assert_grid_rounds_like_the_oracle(&converter, &INTERMEDIATE_UNIT, 7);
    }
}

/// A fixed-seed sweep of `f64` bit patterns across each format's exponent
/// range and a few binades beyond it on both sides.
#[test]
fn seeded_sweep_rounds_as_the_oracle_does() {
    for converter in converters() {
        let format = converter.format;
        let smallest = 1 - format.bias() - format.fraction_bits;
        let largest = i32::try_from(format.top_field()).expect("invariant: small") - format.bias();
        let (low, high) = (smallest - 4, largest + 4);
        let span = u64::try_from(high - low + 1).expect("invariant: low < high");
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state >> 11
        };
        for _ in 0..60_000 {
            let exponent = low + i32::try_from(next() % span).expect("invariant: span is small");
            let field = u64::try_from(exponent + 1023).expect("invariant: normal in f64");
            let sign = (next() & 1) << 63;
            let x =
                f64::from_bits(sign | (field << F64_FRACTION_BITS) | (next() & F64_FRACTION_MASK));
            converter.assert_rounds_like_the_oracle(x);
        }
    }
}

/// Signed zero, infinities, NaN, subnormals of `f64` and `f32`, and the
/// overflow edges of `f32` and `f64`.
#[test]
fn special_and_extreme_inputs_round_as_the_oracle_does() {
    let f32_max = f64::from(f32::MAX);
    let two_to_128 = pow2(128);
    let f32_overflow_midpoint = two_to_128 - pow2(103);
    let inputs = [
        0.0,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7FF0_0000_0000_0001),
        f64::from_bits(0xFFF8_0000_0000_0001),
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        f64::from_bits(F64_FRACTION_MASK),
        f32_max,
        displaced(f32_max, 1),
        displaced(f32_max, -1),
        two_to_128,
        displaced(two_to_128, -1),
        displaced(two_to_128, 1),
        f32_overflow_midpoint,
        displaced(f32_overflow_midpoint, 1),
        displaced(f32_overflow_midpoint, -1),
        // The smallest `f32` subnormal, its half, and their neighbours.
        pow2(-149),
        displaced(pow2(-149), -1),
        displaced(pow2(-149), 1),
        pow2(-150),
        displaced(pow2(-150), 1),
        displaced(pow2(-150), -1),
        // The `f32` normal/subnormal boundary.
        pow2(-126),
        displaced(pow2(-126), -1),
        displaced(pow2(-126), 1),
        // Half the smallest bfloat16 subnormal and its neighbours.
        pow2(-134),
        displaced(pow2(-134), 1),
        displaced(pow2(-134), -1),
        // Half the smallest binary16 subnormal and its neighbours.
        pow2(-25),
        displaced(pow2(-25), 1),
        displaced(pow2(-25), -1),
    ];
    for converter in converters() {
        for x in inputs {
            converter.assert_rounds_like_the_oracle(x);
            converter.assert_rounds_like_the_oracle(-x);
        }
    }
}

/// The inherent conversions round as the oracle does.
#[test]
fn inherent_conversions_round_as_the_oracle_does() {
    for bits in 0x3FF0_0000_0000_0000_u64..0x3FF0_0000_0000_0040 {
        let x = f64::from_bits(bits);
        assert_eq!(
            Some(u32::from(F16::from_f64(x).0)),
            F16_FORMAT.round(x),
            "{bits:#x}"
        );
        assert_eq!(
            Some(u32::from(Bf16::from_f64(x).0)),
            BF16_FORMAT.round(x),
            "{bits:#x}"
        );
    }
    assert_eq!(
        F16::from_f64(f64::from_bits(0x3FF0_0200_0000_0001)).0,
        0x3C01
    );
    assert_eq!(
        Bf16::from_f64(f64::from_bits(0x3FF0_1000_0000_0001)).0,
        0x3F81
    );
}

/// The oracle itself, against hand-derived encodings.
#[test]
fn the_oracle_encodes_known_values() {
    assert_eq!(F16_FORMAT.round(1.0), Some(0x3C00));
    assert_eq!(F16_FORMAT.round(-2.0), Some(0xC000));
    assert_eq!(F16_FORMAT.round(65504.0), Some(0x7BFF));
    // The overflow midpoint `65520` ties to even, which is infinity.
    assert_eq!(F16_FORMAT.round(65520.0), Some(0x7C00));
    assert_eq!(F16_FORMAT.round(displaced(65520.0, -1)), Some(0x7BFF));
    // `2^-24` is the smallest binary16 subnormal; `2^-25` ties to zero.
    assert_eq!(F16_FORMAT.round(pow2(-24)), Some(0x0001));
    assert_eq!(F16_FORMAT.round(pow2(-25)), Some(0x0000));
    assert_eq!(F16_FORMAT.round(displaced(pow2(-25), 1)), Some(0x0001));
    // Ties go to the even neighbour on both sides of `1 + 2^-10`.
    assert_eq!(F16_FORMAT.round(1.0 + pow2(-11)), Some(0x3C00));
    assert_eq!(F16_FORMAT.round(1.0 + pow2(-10) + pow2(-11)), Some(0x3C02));
    assert_eq!(BF16_FORMAT.round(1.0), Some(0x3F80));
    assert_eq!(F8_FORMAT.round(1.0), Some(0x38));
    assert_eq!(F8_FORMAT.round(1.0e9), Some(0x77));
    assert_eq!(F8_FORMAT.round(f64::NEG_INFINITY), Some(0xF7));
    assert_eq!(F4_FORMAT.round(1.0), Some(0x3));
    assert_eq!(F16_FORMAT.round(f64::NAN), None);
    // The value past the largest finite one opens the next binade, also for
    // a format with no fraction bits, whose step doubles there.
    assert_eq!(F16_FORMAT.magnitudes().last(), Some(&65536.0));
    assert_eq!(F4_FORMAT.magnitudes().last(), Some(&16.0));
}

/// Round-to-odd leaves every `f32` unchanged.
#[test]
fn odd_rounding_preserves_every_representable_value() {
    let mut state = 0x0123_4567_89AB_CDEF_u64;
    for _ in 0..200_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let value = f32::from_bits(u32::try_from(state >> 32).expect("invariant: 32 bits"));
        let rounded = odd_rounded(f64::from(value));
        if value.is_nan() {
            assert!(rounded.is_nan(), "{:#010x}", value.to_bits());
            assert_eq!(rounded.is_sign_negative(), value.is_sign_negative());
        } else {
            assert_eq!(rounded.to_bits(), value.to_bits(), "{value:e}");
        }
    }
}

/// An inexact value lands on the odd neighbour: the result's last bit is set
/// and it is within one `f32` unit of the input.
#[test]
fn odd_rounding_picks_the_odd_neighbour() {
    let mut state = 0xFEDC_BA98_7654_3210_u64;
    for _ in 0..200_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // The normal `f32` range: exponents -126..=127.
        let exponent = -126 + i32::try_from((state >> 40) % 254).expect("invariant: small");
        let field = u64::try_from(exponent + 1023).expect("invariant: normal in f64");
        let sign = (state & 1) << 63;
        let x = f64::from_bits(
            sign | (field << F64_FRACTION_BITS) | ((state >> 8) & F64_FRACTION_MASK),
        );
        let rounded = odd_rounded(x);
        let representable = x.to_bits() & ((1 << 29) - 1) == 0;
        let unit = pow2(exponent - 23);
        if representable {
            assert_eq!(f64::from(rounded), x, "{:#018x}", x.to_bits());
        } else {
            assert_eq!(rounded.to_bits() & 1, 1, "{:#018x}", x.to_bits());
            assert!(
                (f64::from(rounded) - x).abs() < unit,
                "{:#018x} -> {rounded:e}",
                x.to_bits()
            );
            assert_eq!(rounded.is_sign_negative(), x.is_sign_negative());
        }
    }
}

/// Underflow, `f32` subnormals and overflow: nothing nonzero collapses to
/// zero, and magnitudes of `2^128` or more are infinite.
#[test]
fn odd_rounding_handles_the_range_edges() {
    assert_eq!(odd_rounded(f64::from_bits(1)).to_bits(), 1);
    assert_eq!(odd_rounded(-f64::from_bits(1)).to_bits(), 0x8000_0001);
    assert_eq!(odd_rounded(f64::MIN_POSITIVE).to_bits(), 1);
    assert_eq!(odd_rounded(pow2(-149)).to_bits(), 1);
    assert_eq!(odd_rounded(displaced(pow2(-149), 1)).to_bits(), 1);
    assert_eq!(odd_rounded(pow2(-148)).to_bits(), 2);
    assert_eq!(odd_rounded(displaced(pow2(-148), 1)).to_bits(), 3);
    assert_eq!(odd_rounded(pow2(-126)).to_bits(), 0x0080_0000);
    assert_eq!(
        odd_rounded(displaced(pow2(-126), -1)).to_bits(),
        0x007F_FFFF
    );
    assert_eq!(odd_rounded(0.0).to_bits(), 0);
    assert_eq!(odd_rounded(-0.0).to_bits(), 0x8000_0000);
    assert_eq!(odd_rounded(f64::from(f32::MAX)), f32::MAX);
    assert_eq!(odd_rounded(displaced(f64::from(f32::MAX), 1)), f32::MAX);
    assert_eq!(odd_rounded(displaced(pow2(128), -1)), f32::MAX);
    assert_eq!(odd_rounded(pow2(128)), f32::INFINITY);
    assert_eq!(odd_rounded(f64::MIN), f32::NEG_INFINITY);
    assert_eq!(odd_rounded(f64::INFINITY), f32::INFINITY);
    assert!(odd_rounded(f64::NAN).is_nan());
}

/// A signalling NaN is quieted with its leading payload bits kept, as the
/// hardware `f64` to `f32` narrowing does: payload bit 50 of the `f64`
/// fraction lands on bit 21 of the `f32` fraction beside the quiet bit 22.
#[test]
fn odd_rounding_quiets_nan_and_keeps_its_payload() {
    let signalling = f64::from_bits(0x7FF4_0000_0000_0000);
    assert_eq!(odd_rounded(signalling).to_bits(), 0x7FE0_0000);
    assert_eq!(odd_rounded(-signalling).to_bits(), 0xFFE0_0000);
    assert_eq!(<F16 as FloatElement>::from_f64(signalling).0, 0x7F00);
    assert_eq!(<Bf16 as FloatElement>::from_f64(signalling).0, 0x7FE0);
}
