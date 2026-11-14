//! Contract of [`FloatElement::nth_power`]: the base raised to a `u32`
//! exponent by exponentiation by squaring, evaluated in the element type.
//!
//! Every observation is the exact `f64` widening of the result, compared by
//! bit pattern, so the sign of zero and of infinity is part of each assertion
//! and no assertion carries a tolerance. Calls are fully qualified for the
//! reason given in `float_element.rs`.

use eunomia::{Bf16, Bf4, Bf8, FloatElement, NumericElement, F16, F32, F4, F64, F8};

/// Range facts of a format with an infinity, from its IEEE 754 definition: the
/// largest finite value is below `2^(MAX_EXPONENT + 1)`, and the least positive
/// value is the subnormal `2^-MIN_SUBNORMAL_EXPONENT`.
trait IeeeFormat: FloatElement {
    const SIGNIFICAND_BITS: u32;
    const MAX_EXPONENT: i32;
    const MIN_SUBNORMAL_EXPONENT: i32;
}

macro_rules! ieee_format {
    ($ty:ty, $bits:literal, $max:literal, $min_subnormal:literal) => {
        impl IeeeFormat for $ty {
            const SIGNIFICAND_BITS: u32 = $bits;
            const MAX_EXPONENT: i32 = $max;
            const MIN_SUBNORMAL_EXPONENT: i32 = $min_subnormal;
        }
    };
}

ieee_format!(Bf8, 3, 15, 16);
ieee_format!(F16, 11, 15, 24);
ieee_format!(Bf16, 8, 127, 133);
ieee_format!(F32, 24, 127, 149);
ieee_format!(f32, 24, 127, 149);
ieee_format!(F64, 53, 1023, 1074);
ieee_format!(f64, 53, 1023, 1074);

/// Run a generic check on every shipped `FloatElement` implementor.
macro_rules! every_format {
    ($check:ident) => {
        $check::<F4>();
        $check::<Bf4>();
        $check::<F8>();
        $check::<Bf8>();
        $check::<F16>();
        $check::<Bf16>();
        $check::<F32>();
        $check::<f32>();
        $check::<F64>();
        $check::<f64>();
    };
}

/// Run a generic check on every shipped implementor that has an infinity.
macro_rules! every_ieee_format {
    ($check:ident) => {
        $check::<Bf8>();
        $check::<F16>();
        $check::<Bf16>();
        $check::<F32>();
        $check::<f32>();
        $check::<F64>();
        $check::<f64>();
    };
}

fn make<T: FloatElement>(value: f64) -> T {
    <T as FloatElement>::from_f64(value)
}

fn power<T: FloatElement>(base: f64, exponent: u32) -> f64 {
    NumericElement::to_f64(FloatElement::nth_power(make::<T>(base), exponent))
}

/// `2^exponent` as an `f64`, built from its bit pattern: a normal for
/// `-1022 <= exponent <= 1023`, else the subnormal with that single bit set.
fn power_of_two(exponent: i32) -> f64 {
    if exponent >= -1022 {
        let field = u64::try_from(1023 + exponent).expect("invariant: exponent at least -1022");
        f64::from_bits(field << 52)
    } else {
        let shift = u32::try_from(1074 + exponent).expect("invariant: exponent at least -1074");
        f64::from_bits(1_u64 << shift)
    }
}

fn assert_bits<T>(actual: f64, expected: f64, what: &str) {
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "{}: {what}: got {actual:e}, expected {expected:e}",
        core::any::type_name::<T>()
    );
}

/// Exponents past the `f32` exact-integer limit `2^24` and past `i32::MAX`.
const LARGE_EXPONENTS: [u32; 7] = [
    (1 << 24) + 1,
    (1 << 24) + 2,
    1 << 31,
    (1 << 31) + 1,
    u32::MAX - 2,
    u32::MAX - 1,
    u32::MAX,
];

fn check_small_powers<T: IeeeFormat>() {
    // (base, exponent, result, significant bits the result needs).
    let table: [(f64, u32, f64, u32); 15] = [
        (2.0, 1, 2.0, 1),
        (2.0, 5, 32.0, 1),
        (2.0, 10, 1024.0, 1),
        (-2.0, 3, -8.0, 1),
        (-2.0, 4, 16.0, 1),
        (0.5, 4, 0.0625, 1),
        (-0.5, 3, -0.125, 1),
        (-4.0, 3, -64.0, 1),
        (0.25, 3, 0.015_625, 1),
        (3.0, 3, 27.0, 5),
        (-3.0, 3, -27.0, 5),
        (3.0, 2, 9.0, 4),
        (1.5, 3, 3.375, 5),
        (-1.5, 2, 2.25, 4),
        (10.0, 2, 100.0, 7),
    ];
    for (base, exponent, expected, needs) in table {
        if needs <= T::SIGNIFICAND_BITS {
            assert_bits::<T>(
                power::<T>(base, exponent),
                expected,
                &format!("{base}^{exponent}"),
            );
        }
    }
}

fn check_zero_and_unit_exponents<T: FloatElement>() {
    for base in [1.0, -1.0, 2.0, -2.0, 0.0, -0.0] {
        assert_bits::<T>(power::<T>(base, 0), 1.0, &format!("{base}^0"));
        assert_bits::<T>(power::<T>(base, 1), base, &format!("{base}^1"));
    }
    let nan = FloatElement::nth_power(<T as NumericElement>::NAN, 1);
    assert!(
        NumericElement::is_nan(nan),
        "{}: NaN^1",
        core::any::type_name::<T>()
    );
}

fn check_zero_base_sign<T: FloatElement>() {
    assert_bits::<T>(power::<T>(0.0, 7), 0.0, "+0^7");
    assert_bits::<T>(power::<T>(-0.0, 7), -0.0, "-0^7");
    assert_bits::<T>(power::<T>(-0.0, 8), 0.0, "-0^8");
    assert_bits::<T>(power::<T>(0.0, u32::MAX), 0.0, "+0^u32::MAX");
    assert_bits::<T>(power::<T>(-0.0, u32::MAX), -0.0, "-0^u32::MAX");
}

fn check_negative_unit_parity<T: FloatElement>() {
    for exponent in LARGE_EXPONENTS {
        let expected = if exponent % 2 == 0 { 1.0 } else { -1.0 };
        assert_bits::<T>(
            power::<T>(-1.0, exponent),
            expected,
            &format!("(-1)^{exponent}"),
        );
        assert_bits::<T>(power::<T>(1.0, exponent), 1.0, &format!("1^{exponent}"));
    }
}

fn check_overflow_sign<T: IeeeFormat>() {
    assert_bits::<T>(
        power::<T>(
            2.0,
            u32::try_from(T::MAX_EXPONENT).expect("invariant: positive"),
        ),
        power_of_two(T::MAX_EXPONENT),
        "2^MAX_EXPONENT is finite",
    );
    assert_bits::<T>(
        power::<T>(
            2.0,
            u32::try_from(T::MAX_EXPONENT + 1).expect("invariant: positive"),
        ),
        f64::INFINITY,
        "2^(MAX_EXPONENT + 1) overflows",
    );
    for exponent in LARGE_EXPONENTS {
        let negative = if exponent % 2 == 0 {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        };
        assert_bits::<T>(
            power::<T>(2.0, exponent),
            f64::INFINITY,
            &format!("2^{exponent}"),
        );
        assert_bits::<T>(
            power::<T>(-2.0, exponent),
            negative,
            &format!("(-2)^{exponent}"),
        );
    }
}

fn check_underflow<T: IeeeFormat>() {
    let least = u32::try_from(T::MIN_SUBNORMAL_EXPONENT).expect("invariant: positive");
    assert_bits::<T>(
        power::<T>(0.5, least),
        power_of_two(-T::MIN_SUBNORMAL_EXPONENT),
        "0.5^least is the least subnormal",
    );
    // Half the least subnormal is a tie between it and zero; ties go to even.
    assert_bits::<T>(
        power::<T>(0.5, least + 1),
        0.0,
        "0.5^(least + 1) ties to +0",
    );
    let negative_zero = if (least + 1) % 2 == 0 { 0.0 } else { -0.0 };
    assert_bits::<T>(
        power::<T>(-0.5, least + 1),
        negative_zero,
        "(-0.5)^(least + 1)",
    );
    for exponent in LARGE_EXPONENTS {
        let expected = if exponent % 2 == 0 { 0.0 } else { -0.0 };
        assert_bits::<T>(power::<T>(0.5, exponent), 0.0, &format!("0.5^{exponent}"));
        assert_bits::<T>(
            power::<T>(-0.5, exponent),
            expected,
            &format!("(-0.5)^{exponent}"),
        );
    }
}

/// The loop `powi` carried before the kernel was exposed, which squares the
/// base once more after the last exponent bit.
fn previous_loop<T: FloatElement>(mut base: T, mut exponent: u32) -> T {
    let mut accumulator = <T as NumericElement>::ONE;
    while exponent > 0 {
        if exponent & 1 == 1 {
            accumulator *= base;
        }
        base *= base;
        exponent >>= 1;
    }
    accumulator
}

fn same_value(left: f64, right: f64) -> bool {
    left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
}

fn check_previous_loop_and_powi<T: FloatElement>() {
    let bases = [
        1.5,
        -1.25,
        0.75,
        3.0,
        -0.5,
        2.0,
        -1.0,
        0.5,
        1.000_000_1,
        -0.0,
    ];
    let exponents = (0..=70).chain(LARGE_EXPONENTS).chain([12_345_678, 1 << 30]);
    for base in bases {
        let base = make::<T>(base);
        for exponent in exponents.clone() {
            let got = NumericElement::to_f64(FloatElement::nth_power(base, exponent));
            let old = NumericElement::to_f64(previous_loop(base, exponent));
            assert!(
                same_value(got, old),
                "{}: {}^{exponent}: got {got:e}, previous loop {old:e}",
                core::any::type_name::<T>(),
                NumericElement::to_f64(base)
            );
            if let Ok(signed) = i32::try_from(exponent) {
                let via_powi = NumericElement::to_f64(FloatElement::powi(base, signed));
                assert!(
                    same_value(got, via_powi),
                    "{}: {}^{exponent}: got {got:e}, powi {via_powi:e}",
                    core::any::type_name::<T>(),
                    NumericElement::to_f64(base)
                );
            }
        }
    }
}

#[test]
fn small_powers_are_exact() {
    every_ieee_format!(check_small_powers);
}

#[test]
fn zero_exponent_gives_one_and_unit_exponent_gives_the_base() {
    every_format!(check_zero_and_unit_exponents);
}

#[test]
fn zero_base_keeps_its_sign_by_parity() {
    every_format!(check_zero_base_sign);
}

#[test]
fn negative_unit_keeps_parity_for_exponents_past_the_significand() {
    every_format!(check_negative_unit_parity);
}

#[test]
fn overflow_is_infinity_signed_by_parity() {
    every_ieee_format!(check_overflow_sign);
}

#[test]
fn underflow_reaches_the_least_subnormal_then_signed_zero() {
    every_ieee_format!(check_underflow);
}

#[test]
fn results_equal_the_previous_loop_and_powi_bit_for_bit() {
    every_format!(check_previous_loop_and_powi);
}
