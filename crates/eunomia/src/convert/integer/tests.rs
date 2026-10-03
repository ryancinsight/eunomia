use super::IntegerTarget;
use crate::traits::{FloatElement, NumericElement};
use crate::types::{Bf16, F16, F32, F64};

/// Ties go to the even neighbour, `floor` goes toward `-∞`, and the
/// non-finite values saturate or map to 0. Every finite input has at most
/// four significant bits, so it is exact in each tested format.
fn rounding_and_special_values<T: FloatElement>() {
    let x = T::from_f64;
    assert_eq!(i32::from_rounded(x(0.5)), 0);
    assert_eq!(i32::from_rounded(x(1.5)), 2);
    assert_eq!(i32::from_rounded(x(2.5)), 2);
    assert_eq!(i32::from_rounded(x(-1.5)), -2);
    assert_eq!(i32::from_rounded(x(-0.5)), 0);
    assert_eq!(u8::from_rounded(x(-0.5)), 0);
    assert_eq!(usize::from_floor(x(3.75)), 3);
    assert_eq!(i32::from_floor(x(-0.5)), -1);
    assert_eq!(u8::from_floor(x(-0.5)), 0);
    assert_eq!(u8::from_rounded(x(320.0)), u8::MAX);
    assert_eq!(u32::from_rounded(x(f64::INFINITY)), u32::MAX);
    assert_eq!(usize::from_floor(x(f64::INFINITY)), usize::MAX);
    assert_eq!(i8::from_rounded(x(f64::NEG_INFINITY)), i8::MIN);
    assert_eq!(u32::from_floor(x(f64::NEG_INFINITY)), 0);
    assert_eq!(u8::from_rounded(x(f64::NAN)), 0);
    assert_eq!(usize::from_floor(x(f64::NAN)), 0);
}

#[test]
fn every_float_format_rounds_and_saturates_alike() {
    rounding_and_special_values::<f32>();
    rounding_and_special_values::<f64>();
    rounding_and_special_values::<F16>();
    rounding_and_special_values::<Bf16>();
    rounding_and_special_values::<F32>();
    rounding_and_special_values::<F64>();
}

/// `MAX + 0.5` ties to the even `MAX + 1` and saturates back to `MAX`, while
/// `floor` lands on `MAX` itself. `u8::MAX + 0.5` needs 9 significant bits,
/// so it is exact in `F16` and wider; the wider bounds need `f64`.
#[test]
fn half_past_the_maximum_saturates_to_the_maximum() {
    let half_past_u8 = F16::from_f64(255.5);
    assert_eq!(half_past_u8.to_f64(), 255.5);
    assert_eq!(u8::from_rounded(half_past_u8), u8::MAX);
    assert_eq!(u8::from_floor(half_past_u8), u8::MAX);
    assert_eq!(u32::from_rounded(4_294_967_295.5_f64), u32::MAX);
    assert_eq!(u32::from_floor(4_294_967_295.5_f64), u32::MAX);
    assert_eq!(u32::from_rounded(4_294_967_294.5_f64), 4_294_967_294);
    assert_eq!(i8::from_rounded(-128.5_f64), i8::MIN);
    assert_eq!(i8::from_floor(-128.5_f64), i8::MIN);
    assert_eq!(
        u64::from_rounded(18_446_744_073_709_551_616.0_f64),
        u64::MAX
    );
}

#[test]
fn from_truncated_cuts_toward_zero() {
    assert_eq!(i32::from_truncated(-2.9), -2);
    assert_eq!(i32::from_truncated(2.9), 2);
    assert_eq!(u16::from_truncated(65_535.9), u16::MAX);
    assert_eq!(i64::from_truncated(f64::INFINITY), i64::MAX);
    assert_eq!(isize::from_truncated(f64::NAN), 0);
}
