use crate::traits::{FloatElement, NumericElement, TryFromCount};
use crate::types::{Bf16, Bf4, Bf8, Complex, F16, F32, F4, F64, F8, I16, I32, I8};
use alloc::string::ToString;

/// Every count `0..=2^p` converts exactly, `p` being the format's significand
/// precision, and so does its negation.
fn exact_through_precision<T: FloatElement>(precision: u32) {
    for n in 0..=(1_usize << precision) {
        let value = T::from_count(n);
        assert_eq!(
            value.to_f64(),
            f64::from(u32::try_from(n).expect("invariant: the sweep stops at 2^20"))
        );
        let negated =
            T::from_integer(-i64::try_from(n).expect("invariant: the sweep stops at 2^20"));
        assert_eq!(negated.to_f64(), -value.to_f64());
    }
}

/// Past `2^p` the spacing is 2: `2^p + 1` ties to the even neighbour `2^p`,
/// and `2^p + 3` ties to the even neighbour `2^p + 4`.
fn ties_to_even_past_precision<T: FloatElement>(precision: u32) {
    let limit = 1_usize << precision;
    assert_eq!(T::from_count(limit + 1), T::from_count(limit));
    assert_eq!(T::from_count(limit + 3), T::from_count(limit + 4));
    assert_ne!(T::from_count(limit + 2), T::from_count(limit));
}

#[test]
fn counts_are_exact_through_the_significand() {
    exact_through_precision::<f64>(20);
    exact_through_precision::<F64>(20);
    exact_through_precision::<f32>(20);
    exact_through_precision::<F32>(20);
    exact_through_precision::<F16>(11);
    exact_through_precision::<Bf16>(8);
    exact_through_precision::<F8>(4);
    exact_through_precision::<Bf8>(3);
    exact_through_precision::<Bf4>(1);
    exact_through_precision::<F4>(1);
}

#[test]
fn counts_tie_to_even_past_the_significand() {
    ties_to_even_past_precision::<f32>(24);
    ties_to_even_past_precision::<F32>(24);
    ties_to_even_past_precision::<F16>(11);
    ties_to_even_past_precision::<Bf16>(8);
    ties_to_even_past_precision::<F8>(4);
    ties_to_even_past_precision::<Bf8>(3);
}

/// `Bf4` (E2M1, top exponent reserved) holds 0, 0.5, 1, 1.5, 2 and 3: counts
/// through 3 are exact, and a larger count saturates as `from_f32` does.
#[test]
fn e2m1_counts_saturate_past_its_range() {
    let expected = [0.0, 1.0, 2.0, 3.0, 3.0, 3.0];
    for (n, want) in expected.into_iter().enumerate() {
        assert_eq!(Bf4::from_count(n).to_f64(), want);
    }
    assert_eq!(Bf4::from_count(4), Bf4::from_f32(4.0));
}

/// `n` rounded to `precision` significant bits, nearest, ties to even, in
/// integer arithmetic: the oracle shares no code with the conversion.
fn nearest_even(n: usize, precision: u32) -> usize {
    let width = usize::BITS - n.leading_zeros();
    let Some(shift) = width.checked_sub(precision).filter(|&shift| shift > 0) else {
        return n;
    };
    let unit = 1_usize << shift;
    let below = n & !(unit - 1);
    let half = unit >> 1;
    let rest = n - below;
    let up = rest > half || (rest == half && below & unit != 0);
    if up {
        below + unit
    } else {
        below
    }
}

/// Every midpoint between neighbours of `precision` bits with exponents in
/// `exponents`, and the counts one either side of it, round as the integer
/// oracle does. A count just above a midpoint is the case a round-to-nearest
/// `f32` intermediate gets wrong.
fn rounds_like_the_oracle<T: FloatElement>(precision: u32, exponents: core::ops::Range<u32>) {
    for exponent in exponents {
        let base = 1_usize << exponent;
        let unit = 1_usize << (exponent + 1 - precision);
        for step in [0, 1, 2, 3, 5, 127] {
            let midpoint = base + step * unit + unit / 2;
            for n in [midpoint - 1, midpoint, midpoint + 1] {
                let expected = T::from_f32(F32::from_count(nearest_even(n, precision)).0);
                assert_eq!(T::from_count(n), expected, "count {n}");
                let k = i64::try_from(n).expect("invariant: exponents stay below 63");
                assert_eq!(T::from_integer(-k), T::ZERO - expected, "integer -{n}");
            }
        }
    }
}

#[test]
fn counts_round_once_past_the_intermediate() {
    rounds_like_the_oracle::<Bf16>(8, 24..62);
    rounds_like_the_oracle::<f32>(24, 25..62);
    rounds_like_the_oracle::<F32>(24, 25..62);
}

#[test]
fn counts_round_once_within_range() {
    rounds_like_the_oracle::<F16>(11, 11..15);
    rounds_like_the_oracle::<Bf16>(8, 8..24);
    rounds_like_the_oracle::<F8>(4, 4..7);
    rounds_like_the_oracle::<Bf8>(3, 3..14);
}

/// The widest counts: midpoints in the top two binades and `usize::MAX`,
/// whose 64 one bits round up to `2^64`.
#[test]
fn counts_round_at_full_width() {
    for exponent in [62, 63] {
        let base = 1_usize << exponent;
        let unit = 1_usize << (exponent + 1 - 8);
        for step in [0, 1, 2, 3] {
            let midpoint = base + step * unit + unit / 2;
            for n in [midpoint - 1, midpoint, midpoint + 1] {
                let expected = Bf16::from_f32(F32::from_count(nearest_even(n, 8)).0);
                assert_eq!(Bf16::from_count(n), expected, "count {n}");
            }
        }
    }
    let two_to_64 = 18_446_744_073_709_551_616.0_f32;
    assert_eq!(Bf16::from_count(usize::MAX), Bf16::from_f32(two_to_64));
    assert_eq!(F16::from_count(usize::MAX), F16::from_f32(f32::INFINITY));
    assert_eq!(f32::from_count(usize::MAX), two_to_64);
}

/// `i64::MIN` has no positive counterpart in `i64`; its magnitude `2^63` is
/// exact in every format whose range reaches it.
#[test]
fn most_negative_integer_converts() {
    let two_to_63 = 9_223_372_036_854_775_808.0_f32;
    assert_eq!(Bf16::from_integer(i64::MIN), Bf16::from_f32(-two_to_63));
    assert_eq!(
        F16::from_integer(i64::MIN),
        F16::from_f32(f32::NEG_INFINITY)
    );
    assert_eq!(f32::from_integer(i64::MIN), -two_to_63);
    assert_eq!(F32::from_integer(i64::MIN).0, -two_to_63);
}

#[test]
fn bf16_count_above_a_midpoint_rounds_up() {
    // 2^25 + 2^17 + 1 lies just above the bf16 midpoint 2^25 + 2^17; an `f32`
    // round-to-nearest intermediate lands on the midpoint and ties down.
    let n = (1 << 25) + (1 << 17) + 1;
    assert_eq!(Bf16::from_count(n), Bf16::from_f32(33_816_576.0));
}

/// `2^53 + 1` is the first count `f64` cannot hold; it ties to `2^53`.
#[test]
fn f64_counts_round_past_the_significand() {
    let limit = 1_usize << 53;
    assert_eq!(
        f64::from_count(limit + 1).to_bits(),
        F64::from_count(limit).0.to_bits()
    );
    assert_eq!(f64::from_count(limit + 3), F64::from_count(limit + 4).0);
    assert_eq!(f64::from_integer(i64::MIN), -9_223_372_036_854_775_808.0);
}

#[test]
fn f16_counts_past_its_range_overflow_to_infinity() {
    assert_eq!(F16::from_count(65_504).to_f64(), 65_504.0);
    assert_eq!(F16::from_count(65_520), F16::from_f32(f32::INFINITY));
}

fn integer_counts<T: NumericElement + core::fmt::Debug>(max: usize) {
    assert_eq!(T::try_from_count(0), Ok(T::ZERO));
    assert_eq!(T::try_from_count(1), Ok(T::ONE));
    assert_eq!(T::try_from_count(max), Ok(T::MAX_VALUE));
    let error =
        T::try_from_count(max + 1).expect_err("invariant: the count exceeds the target range");
    assert_eq!(error.count(), max + 1);
    assert_eq!(error.target(), core::any::type_name::<T>());
}

#[test]
fn integer_counts_are_exact_or_refused() {
    integer_counts::<i8>(127);
    integer_counts::<i16>(32_767);
    integer_counts::<i32>(2_147_483_647);
    integer_counts::<i64>(9_223_372_036_854_775_807);
    integer_counts::<isize>(9_223_372_036_854_775_807);
    integer_counts::<u8>(255);
    integer_counts::<u16>(65_535);
    integer_counts::<u32>(4_294_967_295);
    assert_eq!(usize::try_from_count(usize::MAX), Ok(usize::MAX));
    assert_eq!(u64::try_from_count(usize::MAX), Ok(u64::MAX));
}

#[test]
fn integer_wrappers_take_their_inner_range() {
    assert_eq!(I8::try_from_count(127), Ok(I8(127)));
    assert!(I8::try_from_count(128).is_err());
    assert_eq!(I16::try_from_count(32_767), Ok(I16(32_767)));
    assert!(I16::try_from_count(32_768).is_err());
    assert_eq!(I32::try_from_count(7), Ok(I32(7)));
    let error =
        I32::try_from_count(1 << 31).expect_err("invariant: the count exceeds the target range");
    assert_eq!(
        error.to_string(),
        "count 2147483648 lies outside the range of `eunomia::types::ints::I32`"
    );
}

#[test]
fn float_counts_never_fail() {
    assert_eq!(
        f32::try_from_count(usize::MAX),
        Ok(f32::from_count(usize::MAX))
    );
    assert_eq!(Bf16::try_from_count(9), Ok(Bf16::from_f32(9.0)));
}

#[test]
fn const_forms_evaluate_in_const_context() {
    const LENGTH: F64 = F64::from_count(3);
    const OFFSET: F32 = F32::from_integer(-5);
    assert_eq!(LENGTH.0, 3.0);
    assert_eq!(OFFSET.0, -5.0);
}

#[test]
fn complex_counts_embed_as_the_real_part() {
    assert_eq!(
        Complex::<f64>::try_from_count(4),
        Ok(Complex::new(4.0, 0.0))
    );
    assert_eq!(
        Complex::<i8>::try_from_count(200)
            .expect_err("invariant: the count exceeds the target range")
            .count(),
        200
    );
}

/// `value = m * 2^e` for a finite non-negative `f64`, `m` its integer
/// significand.
fn significand(value: f64) -> (u128, i32) {
    let bits = value.to_bits();
    let field = i32::try_from(bits >> 52).expect("invariant: a non-negative f64 has no sign bit");
    let fraction = u128::from(bits & ((1_u64 << 52) - 1));
    if field == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), field - 1075)
    }
}

/// `|a * n - 1| * 2^-scale` in exact integer arithmetic, for a grid value `a`
/// whose exponent is at least `scale`.
fn scaled_distance(a: f64, n: usize, scale: i32) -> u128 {
    let (m, e) = significand(a);
    let one =
        1_u128 << u32::try_from(-scale).expect("invariant: reciprocals of counts are below 1");
    if m == 0 {
        return one;
    }
    let shift = u32::try_from(e - scale).expect("invariant: scale is the smallest exponent");
    let product = (m * u128::try_from(n).expect("invariant: usize fits u128")) << shift;
    product.abs_diff(one)
}

/// `T::from_count_reciprocal(n)` is strictly nearer to `1/n` than both of its
/// grid neighbours. The neighbours come from the format's bit pattern and
/// the distances from integer arithmetic, so the check shares no code with the
/// conversion; the strict inequality also pins that `1/n` never ties.
fn nearest_reciprocal<T: FloatElement + core::fmt::Debug>(
    n: usize,
    neighbours: impl Fn(T) -> (T, T),
) {
    let value = T::from_count_reciprocal(n);
    let (below, above) = neighbours(value);
    let candidates = [value.to_f64(), below.to_f64(), above.to_f64()];
    assert!(
        candidates[1] < candidates[0] && candidates[0] < candidates[2],
        "count {n}: {value:?}"
    );
    let scale = candidates
        .iter()
        .filter(|&&v| v > 0.0)
        .map(|&v| significand(v).1)
        .min()
        .expect("invariant: the converted reciprocal is nonzero");
    let [nearest, low, high] = candidates.map(|v| scaled_distance(v, n, scale));
    assert!(
        nearest < low && nearest < high,
        "count {n}: {value:?} is not the nearest"
    );
}

fn f64_neighbours(x: f64) -> (f64, f64) {
    (
        f64::from_bits(x.to_bits() - 1),
        f64::from_bits(x.to_bits() + 1),
    )
}

fn f32_neighbours(x: f32) -> (f32, f32) {
    (
        f32::from_bits(x.to_bits() - 1),
        f32::from_bits(x.to_bits() + 1),
    )
}

fn half_neighbours(x: F16) -> (F16, F16) {
    (
        F16::from_bits(x.to_bits() - 1),
        F16::from_bits(x.to_bits() + 1),
    )
}

fn brain_neighbours(x: Bf16) -> (Bf16, Bf16) {
    (
        Bf16::from_bits(x.to_bits() - 1),
        Bf16::from_bits(x.to_bits() + 1),
    )
}

/// Counts at the boundaries the reciprocal must survive: just above `2^p`
/// for each precision (where rounding `n` first costs a full ulp: 269 in
/// `Bf16`, 2079 in `F16`), around the largest finite `F16` (65504) and the
/// first count that overflows it (65520), and powers of two.
const BOUNDARY_COUNTS: [usize; 14] = [
    1,
    2,
    3,
    7,
    257,
    269,
    2049,
    2079,
    65_504,
    65_519,
    65_520,
    65_536,
    1 << 20,
    (1 << 24) + 1,
];

/// A deterministic spread of counts across `1..2^bits` (a linear congruential
/// sequence, so failures replay).
fn spread(bits: u32) -> impl Iterator<Item = usize> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    core::iter::repeat_with(move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let count = usize::try_from(state >> (64 - bits)).expect("invariant: bits <= 64");
        count.max(1)
    })
    .take(4096)
}

#[test]
fn count_reciprocals_are_nearest_in_every_shipped_float() {
    for n in BOUNDARY_COUNTS.into_iter().chain(1..=4096) {
        nearest_reciprocal::<f64>(n, f64_neighbours);
        nearest_reciprocal::<f32>(n, f32_neighbours);
        nearest_reciprocal::<F16>(n, half_neighbours);
        nearest_reciprocal::<Bf16>(n, brain_neighbours);
        assert_eq!(
            F64::from_count_reciprocal(n).0,
            f64::from_count_reciprocal(n)
        );
        assert_eq!(
            F32::from_count_reciprocal(n).0,
            f32::from_count_reciprocal(n)
        );
    }
}

/// Counts up to `2^25 - 1`, where the `F16` reciprocal is still above half
/// its smallest subnormal `2^-24`; the full `usize` range for the others.
#[test]
fn count_reciprocals_are_nearest_across_the_count_range() {
    for n in spread(25) {
        nearest_reciprocal::<F16>(n, half_neighbours);
    }
    for n in spread(64).chain([(1 << 53) + 1, (1 << 63) + 12_345, usize::MAX]) {
        nearest_reciprocal::<f64>(n, f64_neighbours);
        nearest_reciprocal::<f32>(n, f32_neighbours);
        nearest_reciprocal::<Bf16>(n, brain_neighbours);
    }
}

/// `1/2^25` is the midpoint between zero and the smallest `F16` subnormal
/// `2^-24`; it ties to the even neighbour, zero. One count less lies above
/// the midpoint and rounds to the subnormal.
#[test]
fn f16_count_reciprocals_underflow_at_half_the_smallest_subnormal() {
    let smallest = F16::from_bits(1);
    assert_eq!(F16::from_count_reciprocal((1 << 25) - 1), smallest);
    assert_eq!(F16::from_count_reciprocal(1 << 25), F16::ZERO);
    assert_eq!(F16::from_count_reciprocal(usize::MAX), F16::ZERO);
}

/// The overflow the reciprocal avoids: `F16::from_count(65520)` is infinite,
/// so `ONE / from_count` is zero there, while the reciprocal is the nearest
/// subnormal `256 * 2^-24`.
#[test]
fn f16_count_reciprocal_survives_count_overflow() {
    assert_eq!(F16::ONE / F16::from_count(65_520), F16::ZERO);
    assert_eq!(F16::from_count_reciprocal(65_520), F16::from_bits(256));
}

#[test]
fn zero_count_reciprocal_is_the_ieee_quotient() {
    assert_eq!(f64::from_count_reciprocal(0), f64::INFINITY);
    assert_eq!(f32::from_count_reciprocal(0), f32::INFINITY);
    assert_eq!(F16::from_count_reciprocal(0), F16::from_f32(f32::INFINITY));
    assert_eq!(F8::from_count_reciprocal(0), F8::from_f32(f32::INFINITY));
}

/// The byte formats share the narrow default with `F16` and `Bf16`; their
/// grids are small enough to check against `from_f32` of exact reciprocals.
#[test]
fn byte_format_count_reciprocals_round_exact_powers_of_two() {
    for exponent in 0..4 {
        let n = 1_usize << exponent;
        let exact = 1.0 / f32::from(u8::try_from(n).expect("invariant: n <= 8"));
        assert_eq!(F8::from_count_reciprocal(n), F8::from_f32(exact));
        assert_eq!(Bf8::from_count_reciprocal(n), Bf8::from_f32(exact));
    }
}
