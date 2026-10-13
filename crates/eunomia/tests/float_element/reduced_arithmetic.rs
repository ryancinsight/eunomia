//! Value-semantic arithmetic checks for the reduced binary formats.

use super::support;
use core::ops::{AddAssign, DivAssign, MulAssign, Neg, Rem, RemAssign, SubAssign};
use eunomia::{Bf16, Bf4, Bf8, FloatElement, F16, F4, F8};
trait ReducedFormat:
    FloatElement
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + Rem<Output = Self>
    + RemAssign
    + Neg<Output = Self>
{
    const EXPONENT_BITS: u32;
    const MANTISSA_BITS: u32;
    const FINITE_ONLY: bool;

    fn raw(self) -> u32;
    fn from_raw(bits: u32) -> Self;
    fn widen(bits: u32) -> f64;
    fn narrow(value: f64) -> u32;

    fn is_nan_raw(bits: u32) -> bool;
    fn max_finite_raw(negative: bool) -> u32 {
        let sign = u32::from(negative) << (Self::EXPONENT_BITS + Self::MANTISSA_BITS);
        let exponent = ((1 << Self::EXPONENT_BITS) - 2) << Self::MANTISSA_BITS;
        let mantissa = (1 << Self::MANTISSA_BITS) - 1;
        sign | exponent | mantissa
    }
}

macro_rules! reduced_format {
    ($ty:ty, $storage:ty, $exponent:literal, $mantissa:literal, $finite:literal) => {
        impl ReducedFormat for $ty {
            const EXPONENT_BITS: u32 = $exponent;
            const MANTISSA_BITS: u32 = $mantissa;
            const FINITE_ONLY: bool = $finite;

            fn raw(self) -> u32 {
                u32::from(self.0)
            }

            fn from_raw(bits: u32) -> Self {
                Self(<$storage>::try_from(bits).expect("invariant: bit pattern fits storage"))
            }

            fn widen(bits: u32) -> f64 {
                let exponent = (bits >> $mantissa) & ((1 << $exponent) - 1);
                if $finite && exponent == (1 << $exponent) - 1 {
                    return f64::NAN;
                }
                f64::from(f32::from_bits(support::widen::<$exponent, $mantissa>(bits)))
            }

            fn is_nan_raw(bits: u32) -> bool {
                let exponent = (bits >> $mantissa) & ((1 << $exponent) - 1);
                support::is_nan::<$exponent, $mantissa>(bits)
                    || ($finite && exponent == (1 << $exponent) - 1)
            }

            #[expect(
                clippy::cast_possible_truncation,
                reason = "the independent value oracle narrows before format encoding"
            )]
            fn narrow(value: f64) -> u32 {
                let bits = support::narrow::<$exponent, $mantissa>((value as f32).to_bits());
                let exponent = (bits >> $mantissa) & ((1 << $exponent) - 1);
                if $finite && !value.is_nan() && exponent == (1 << $exponent) - 1 {
                    let sign = u32::from(value.is_sign_negative()) << ($exponent + $mantissa);
                    return sign | ((1 << $exponent) - 2) << $mantissa | ((1 << $mantissa) - 1);
                }
                bits
            }
        }
    };
}

reduced_format!(F16, u16, 5, 10, false);
reduced_format!(Bf16, u16, 8, 7, false);
reduced_format!(Bf8, u8, 5, 2, false);
reduced_format!(Bf4, u8, 2, 1, true);
reduced_format!(F8, u8, 4, 3, true);
reduced_format!(F4, u8, 3, 0, true);

fn assert_arithmetic_result<T: ReducedFormat>(actual: u32, expected: f64, operation: &str) {
    if expected.is_nan() {
        assert!(
            T::is_nan_raw(actual),
            "{operation} must encode NaN, got {actual:#x}"
        );
    } else {
        assert_eq!(
            actual,
            T::narrow(expected),
            "{operation} differs for operands in the declared format"
        );
    }
}

fn arithmetic_cases<T: ReducedFormat>() -> Vec<u32> {
    let width = 1 + T::EXPONENT_BITS + T::MANTISSA_BITS;
    let count = 1 << width;
    if width <= 8 {
        return (0..count).collect();
    }

    let all_exponents = (1 << T::EXPONENT_BITS) - 1;
    let bias: u32 = (1 << (T::EXPONENT_BITS - 1)) - 1;
    let exponents = [
        0,
        1,
        bias.saturating_sub(1),
        bias,
        bias + 1,
        all_exponents - 2,
        all_exponents - 1,
        all_exponents,
    ];
    let mantissa_mask = (1 << T::MANTISSA_BITS) - 1;
    let mantissas = [0, 1, 1 << T::MANTISSA_BITS.saturating_sub(1), mantissa_mask];
    let sign = 1 << (T::EXPONENT_BITS + T::MANTISSA_BITS);
    let mut cases = Vec::with_capacity(128);
    for exponent in exponents {
        for mantissa in mantissas {
            for negative in [false, true] {
                cases
                    .push((u32::from(negative) * sign) | (exponent << T::MANTISSA_BITS) | mantissa);
            }
        }
    }
    let mask = count - 1;
    let mut state = 0x9E37_79B9_u32;
    for _ in 0..32 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        cases.push(state & mask);
    }
    cases.sort_unstable();
    cases.dedup();
    cases
}

fn check_reduced_arithmetic<T: ReducedFormat>() {
    let cases = arithmetic_cases::<T>();
    for &left_bits in &cases {
        let left = T::from_raw(left_bits);
        let left_value = T::widen(left_bits);
        assert_arithmetic_result::<T>((-left).raw(), -left_value, "negation");
        for &right_bits in &cases {
            let right = T::from_raw(right_bits);
            let right_value = T::widen(right_bits);
            assert_arithmetic_result::<T>(
                (left + right).raw(),
                left_value + right_value,
                "addition",
            );
            assert_arithmetic_result::<T>(
                (left - right).raw(),
                left_value - right_value,
                "subtraction",
            );
            assert_arithmetic_result::<T>(
                (left * right).raw(),
                left_value * right_value,
                "multiplication",
            );
            assert_arithmetic_result::<T>(
                (left / right).raw(),
                left_value / right_value,
                "division",
            );
            assert_arithmetic_result::<T>(
                (left % right).raw(),
                left_value % right_value,
                "remainder",
            );
        }
    }

    let sign = 1 << (T::EXPONENT_BITS + T::MANTISSA_BITS);
    assert_eq!(
        (T::from_raw(sign) + T::from_raw(sign)).raw(),
        sign,
        "adding two negative zeros preserves negative zero"
    );
    assert_eq!(
        (-T::from_raw(0)).raw(),
        sign,
        "negation flips the zero sign"
    );

    let maximum = T::from_raw(T::max_finite_raw(false));
    let minimum = T::from_raw(1);
    assert_eq!(
        (maximum + minimum).raw(),
        maximum.raw(),
        "a minimum positive value cannot move the largest finite value"
    );
    assert_eq!(
        (maximum % minimum).raw(),
        0,
        "the largest finite encoding is an integral multiple of the minimum"
    );
    assert_eq!(
        (minimum % maximum).raw(),
        minimum.raw(),
        "a smaller positive operand is its own remainder"
    );

    let precision = T::MANTISSA_BITS;
    let exponent_bias = (1 << (T::EXPONENT_BITS - 1)) - 1;
    let one = T::from_raw(exponent_bias << precision);
    let halfway_value =
        2.0_f64.powi(-i32::try_from(precision + 1).expect("invariant: precision fits i32"));
    let halfway = T::from_f64(halfway_value);
    if T::widen(halfway.raw()) == halfway_value {
        if precision == 0 {
            assert_eq!(
                (one + halfway).raw(),
                one.raw() + 1,
                "a midpoint at a binade boundary rounds to the even significand"
            );
        } else {
            assert_eq!(
                (one + halfway).raw(),
                one.raw(),
                "a midpoint rounds to the even lower significand"
            );
            let odd = T::from_raw(one.raw() + 1);
            assert_eq!(
                (odd + halfway).raw(),
                odd.raw() + 1,
                "a midpoint rounds to the even upper significand"
            );
        }
    }

    let two = one + one;
    let four = two * two;
    let mut assigned = one;
    assigned += one;
    assert_eq!(assigned.raw(), two.raw(), "AddAssign matches Add");
    assigned -= one;
    assert_eq!(assigned.raw(), one.raw(), "SubAssign matches Sub");
    assigned *= two;
    assert_eq!(assigned.raw(), two.raw(), "MulAssign matches Mul");
    assigned /= two;
    assert_eq!(assigned.raw(), one.raw(), "DivAssign matches Div");
    let expected_remainder = (four % two).raw();
    assigned = four;
    assigned %= two;
    assert_eq!(assigned.raw(), expected_remainder, "RemAssign matches Rem");

    if T::FINITE_ONLY {
        let maximum = T::from_raw(T::max_finite_raw(false));
        let negative_maximum = T::from_raw(T::max_finite_raw(true));
        assert_eq!((maximum + maximum).raw(), maximum.raw());
        assert_eq!((maximum * two).raw(), maximum.raw());
        assert_eq!((maximum / T::from_raw(0)).raw(), maximum.raw());
        assert_eq!(
            (negative_maximum / T::from_raw(0)).raw(),
            negative_maximum.raw()
        );
    } else {
        let maximum = T::from_raw(T::max_finite_raw(false));
        assert!(T::is_nan_raw((T::from_raw(0) / T::from_raw(0)).raw()));
        assert!(T::widen((maximum + maximum).raw()).is_infinite());
        assert!(T::widen((maximum * two).raw()).is_infinite());
        assert!(T::widen((one / T::from_raw(0)).raw()).is_infinite());
    }
}

#[test]
fn reduced_arithmetic_matches_format_rounding() {
    check_reduced_arithmetic::<F16>();
    check_reduced_arithmetic::<Bf16>();
    check_reduced_arithmetic::<Bf8>();
    check_reduced_arithmetic::<Bf4>();
    check_reduced_arithmetic::<F8>();
    check_reduced_arithmetic::<F4>();
}
