use super::{upper_exclusive, IntegerConversionError, IntegerTarget};
use crate::traits::{FloatElement, NumericElement};
use crate::types::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8};
use core::fmt::Debug;

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

fn checked_pair_contract<T, I>()
where
    T: FloatElement,
    I: IntegerTarget + Debug + Eq + TryFrom<i8>,
{
    let zero = I::try_from(0_i8).unwrap_or_else(|_| panic!("invariant: every target represents 0"));

    assert_eq!(I::try_from_rounded(T::from_f64(0.5)), Ok(zero));
    assert_eq!(I::try_from_rounded(T::from_f64(-0.5)), Ok(zero));
    assert_eq!(
        I::try_from_rounded(T::from_f64(f64::NAN)),
        Err(IntegerConversionError::NotANumber)
    );
}

fn checked_nonfinite_pair_contract<T, I>()
where
    T: FloatElement,
    I: IntegerTarget + Debug + Eq,
{
    assert_eq!(
        I::try_from_rounded(T::from_f64(f64::INFINITY)),
        Err(IntegerConversionError::Infinite)
    );
    assert_eq!(
        I::try_from_rounded(T::from_f64(f64::NEG_INFINITY)),
        Err(IntegerConversionError::Infinite)
    );
}

fn checked_upward_tie_pair_contract<T, I>()
where
    T: FloatElement,
    I: IntegerTarget + Debug + Eq + TryFrom<i8>,
{
    let two = I::try_from(2_i8).unwrap_or_else(|_| panic!("invariant: every target represents 2"));

    assert_eq!(T::from_f64(1.5).to_f64(), 1.5);
    assert_eq!(I::try_from_rounded(T::from_f64(1.5)), Ok(two));
}

fn checked_contract_for_float<T: FloatElement>() {
    macro_rules! check_targets {
        ($($target:ty),+ $(,)?) => {
            $(checked_pair_contract::<T, $target>();)+
        };
    }

    check_targets!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
}

fn checked_nonfinite_contract_for_float<T: FloatElement>() {
    macro_rules! check_targets {
        ($($target:ty),+ $(,)?) => {
            $(checked_nonfinite_pair_contract::<T, $target>();)+
        };
    }

    check_targets!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
}

fn checked_upward_tie_contract_for_float<T: FloatElement>() {
    macro_rules! check_targets {
        ($($target:ty),+ $(,)?) => {
            $(checked_upward_tie_pair_contract::<T, $target>();)+
        };
    }

    check_targets!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
}

fn checked_signed_boundary<T, I>(minimum: f64, upper_exclusive: f64, expected_minimum: I)
where
    T: FloatElement,
    I: IntegerTarget + Debug + Eq,
{
    let minimum_input = T::from_f64(minimum);
    let upper_input = T::from_f64(upper_exclusive);

    assert_eq!(minimum_input.to_f64(), minimum);
    assert_eq!(upper_input.to_f64(), upper_exclusive);
    assert_eq!(I::try_from_rounded(minimum_input), Ok(expected_minimum));
    assert_eq!(
        I::try_from_rounded(upper_input),
        Err(IntegerConversionError::OutOfRange)
    );
}

fn checked_unsigned_boundary<T, I>(upper_exclusive: f64, expected_zero: I)
where
    T: FloatElement,
    I: IntegerTarget + Debug + Eq,
{
    let zero_input = T::from_f64(0.0);
    let upper_input = T::from_f64(upper_exclusive);

    assert_eq!(zero_input.to_f64(), 0.0);
    assert_eq!(upper_input.to_f64(), upper_exclusive);
    assert_eq!(I::try_from_rounded(zero_input), Ok(expected_zero));
    assert_eq!(
        I::try_from_rounded(upper_input),
        Err(IntegerConversionError::OutOfRange)
    );
}

fn checked_finite_extrema_pair<T, I>(minimum: i16, maximum: i16)
where
    T: FloatElement,
    I: IntegerTarget + Debug + Eq + TryFrom<i16>,
{
    let expected_minimum = I::try_from(minimum).map_err(|_| IntegerConversionError::OutOfRange);
    let expected_maximum = I::try_from(maximum).map_err(|_| IntegerConversionError::OutOfRange);
    let minimum_input = T::from_f64(f64::NEG_INFINITY);
    let maximum_input = T::from_f64(f64::INFINITY);

    assert_eq!(minimum_input.to_f64(), f64::from(minimum));
    assert_eq!(maximum_input.to_f64(), f64::from(maximum));
    assert_eq!(I::try_from_rounded(minimum_input), expected_minimum);
    assert_eq!(I::try_from_rounded(maximum_input), expected_maximum);
}

fn checked_finite_extrema_contract<T: FloatElement>(minimum: i16, maximum: i16) {
    macro_rules! check_targets {
        ($($target:ty),+ $(,)?) => {
            $(checked_finite_extrema_pair::<T, $target>(minimum, maximum);)+
        };
    }

    check_targets!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
}

#[test]
fn checked_conversion_covers_every_float_and_integer_pair() {
    checked_contract_for_float::<f32>();
    checked_contract_for_float::<f64>();
    checked_contract_for_float::<F16>();
    checked_contract_for_float::<Bf16>();
    checked_contract_for_float::<Bf8>();
    checked_contract_for_float::<Bf4>();
    checked_contract_for_float::<F8>();
    checked_contract_for_float::<F4>();
    checked_contract_for_float::<F32>();
    checked_contract_for_float::<F64>();
}

#[test]
fn checked_conversion_rounds_upward_half_integer_ties() {
    checked_upward_tie_contract_for_float::<f32>();
    checked_upward_tie_contract_for_float::<f64>();
    checked_upward_tie_contract_for_float::<F16>();
    checked_upward_tie_contract_for_float::<Bf16>();
    checked_upward_tie_contract_for_float::<Bf8>();
    checked_upward_tie_contract_for_float::<Bf4>();
    checked_upward_tie_contract_for_float::<F8>();
    checked_upward_tie_contract_for_float::<F32>();
    checked_upward_tie_contract_for_float::<F64>();

    // `F4`'s E3M0 grid has no representable upward half-integer: its only
    // finite half-integers are ±0.5, both covered by the all-pairs test.
    assert_eq!(F4::from_f64(1.5).to_f64(), 2.0);
    assert_eq!(i8::try_from_rounded(F4::from_f64(1.5)), Ok(2));
}

#[test]
fn checked_conversion_exercises_representable_destination_boundaries() {
    macro_rules! i8_boundaries {
        ($($source:ty),+ $(,)?) => {$({
            checked_signed_boundary::<$source, i8>(-128.0, 128.0, i8::MIN);
        })+};
    }

    macro_rules! u8_boundaries {
        ($($source:ty),+ $(,)?) => {$({
            checked_unsigned_boundary::<$source, u8>(256.0, u8::MIN);
        })+};
    }

    i8_boundaries!(f32, f64, F16, Bf16, Bf8, F8, F32, F64);
    u8_boundaries!(f32, f64, F16, Bf16, Bf8, F32, F64);

    // Bf4 and F4 cannot encode an i8 or u8 range edge. Their complete source
    // ranges are checked against every destination in the extrema test.
}

#[test]
fn checked_conversion_rejects_nonfinite_values_for_formats_with_infinity() {
    checked_nonfinite_contract_for_float::<f32>();
    checked_nonfinite_contract_for_float::<f64>();
    checked_nonfinite_contract_for_float::<F16>();
    checked_nonfinite_contract_for_float::<Bf16>();
    checked_nonfinite_contract_for_float::<Bf8>();
    checked_nonfinite_contract_for_float::<F32>();
    checked_nonfinite_contract_for_float::<F64>();
}

#[test]
fn checked_conversion_uses_saturated_extrema_for_finite_only_formats() {
    checked_finite_extrema_contract::<Bf4>(-3, 3);
    checked_finite_extrema_contract::<F8>(-240, 240);
    checked_finite_extrema_contract::<F4>(-8, 8);
}

#[test]
fn checked_conversion_enforces_every_destination_boundary() {
    macro_rules! signed_boundary {
        ($($target:ty),+ $(,)?) => {$({
            let upper = upper_exclusive(<$target>::BITS - 1);
            let inside_upper = if <$target>::BITS <= 53 {
                upper - 1.0
            } else {
                f64::from_bits(upper.to_bits() - 1)
            };
            let below_lower = if <$target>::BITS <= 53 {
                -upper - 1.0
            } else {
                f64::from_bits((-upper).to_bits() + 1)
            };

            assert_eq!(
                <$target>::try_from_rounded(-upper),
                Ok(<$target>::MIN)
            );
            assert_eq!(
                <$target>::try_from_rounded(below_lower),
                Err(IntegerConversionError::OutOfRange)
            );
            assert_eq!(
                <$target>::try_from_rounded(inside_upper),
                Ok(<$target>::from_truncated(inside_upper))
            );
            assert_eq!(
                <$target>::try_from_rounded(upper),
                Err(IntegerConversionError::OutOfRange)
            );
        })+};
    }

    macro_rules! unsigned_boundary {
        ($($target:ty),+ $(,)?) => {$({
            let upper = upper_exclusive(<$target>::BITS);
            let inside_upper = if <$target>::BITS <= 53 {
                upper - 1.0
            } else {
                f64::from_bits(upper.to_bits() - 1)
            };

            assert_eq!(<$target>::try_from_rounded(0.0_f64), Ok(<$target>::MIN));
            assert_eq!(
                <$target>::try_from_rounded(-1.0_f64),
                Err(IntegerConversionError::OutOfRange)
            );
            assert_eq!(
                <$target>::try_from_rounded(inside_upper),
                Ok(<$target>::from_truncated(inside_upper))
            );
            assert_eq!(
                <$target>::try_from_rounded(upper),
                Err(IntegerConversionError::OutOfRange)
            );
        })+};
    }

    signed_boundary!(i8, i16, i32, i64, isize);
    unsigned_boundary!(u8, u16, u32, u64, usize);
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
