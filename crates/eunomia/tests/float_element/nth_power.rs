//! Contract of [`FloatElement::nth_power`]: the base raised to a `u32`
//! exponent by squaring, evaluated in the element type.
//!
//! Exact-value observations are the `f64` widening of the result compared by
//! bit pattern, so the sign of zero and of infinity is part of each assertion
//! and carries no tolerance. Only the error-bound test compares against a
//! reference value, and it derives its slack.

use eunomia::{Bf16, Bf4, Bf8, FloatElement, NumericElement, F16, F32, F4, F64, F8};

use super::integer_power::{assert_bits, make, power_of_two};

/// Range facts of a format, from its definition. The unit roundoff is
/// `2^-SIGNIFICAND_BITS`; the largest finite value lies in
/// `[2^MAX_EXPONENT, 2^(MAX_EXPONENT + 1))`; the least positive value is
/// `2^-MIN_SUBNORMAL_EXPONENT`; `OVERFLOW` is what a positive result past the
/// largest finite value becomes: infinity, or for a finite-only format that
/// largest finite value.
trait FormatRange: FloatElement {
    const SIGNIFICAND_BITS: i32;
    const MAX_EXPONENT: i32;
    const MIN_SUBNORMAL_EXPONENT: i32;
    const OVERFLOW: f64;
}

macro_rules! format_range {
    ($ty:ty, $bits:literal, $max:literal, $min_subnormal:literal, $overflow:expr) => {
        impl FormatRange for $ty {
            const SIGNIFICAND_BITS: i32 = $bits;
            const MAX_EXPONENT: i32 = $max;
            const MIN_SUBNORMAL_EXPONENT: i32 = $min_subnormal;
            const OVERFLOW: f64 = $overflow;
        }
    };
}

format_range!(F4, 1, 3, 2, 8.0);
format_range!(Bf4, 2, 1, 1, 3.0);
format_range!(F8, 4, 7, 9, 240.0);
format_range!(Bf8, 3, 15, 16, f64::INFINITY);
format_range!(F16, 11, 15, 24, f64::INFINITY);
format_range!(Bf16, 8, 127, 133, f64::INFINITY);
format_range!(F32, 24, 127, 149, f64::INFINITY);
format_range!(f32, 24, 127, 149, f64::INFINITY);
format_range!(F64, 53, 1023, 1074, f64::INFINITY);
format_range!(f64, 53, 1023, 1074, f64::INFINITY);

fn power<T: FloatElement>(base: f64, exponent: u32) -> f64 {
    NumericElement::to_f64(FloatElement::nth_power(make::<T>(base), exponent))
}

fn through<T: FloatElement>(value: f64) -> f64 {
    NumericElement::to_f64(make::<T>(value))
}

/// `-1` when a negative base meets an odd exponent, else `1`.
fn parity_sign(base_negative: bool, exponent: u32) -> f64 {
    if base_negative && exponent % 2 == 1 {
        -1.0
    } else {
        1.0
    }
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

/// The range facts agree with the conversion, which does not use the kernel.
fn check_range_facts<T: FormatRange>() {
    let least = power_of_two(-T::MIN_SUBNORMAL_EXPONENT);
    assert_bits::<T>(through::<T>(least), least, "least positive value");
    assert_bits::<T>(through::<T>(least / 2.0), 0.0, "half of it ties to zero");
    let largest = power_of_two(T::MAX_EXPONENT);
    assert_bits::<T>(through::<T>(largest), largest, "2^MAX_EXPONENT");
    assert_bits::<T>(
        through::<T>(2.0 * largest),
        T::OVERFLOW,
        "2^(MAX_EXPONENT + 1)",
    );
    let ulp = power_of_two(1 - T::SIGNIFICAND_BITS);
    assert_bits::<T>(through::<T>(1.0 + ulp), 1.0 + ulp, "1 + ulp");
    assert_bits::<T>(through::<T>(1.0 + ulp / 4.0), 1.0, "1 + ulp / 4");
}

fn check_small_powers<T: FormatRange>() {
    let table: [(f64, u32, f64); 17] = [
        (2.0, 1, 2.0),
        (2.0, 2, 4.0),
        (-2.0, 2, 4.0),
        (2.0, 5, 32.0),
        (2.0, 10, 1024.0),
        (-2.0, 3, -8.0),
        (-2.0, 4, 16.0),
        (0.5, 4, 0.0625),
        (-0.5, 3, -0.125),
        (-4.0, 3, -64.0),
        (0.25, 3, 0.015_625),
        (3.0, 3, 27.0),
        (-3.0, 3, -27.0),
        (3.0, 2, 9.0),
        (1.5, 3, 3.375),
        (-1.5, 2, 2.25),
        (10.0, 2, 100.0),
    ];
    let mut exercised = 0;
    for (base, exponent, expected) in table {
        // Every partial power has fewer significant bits and a lower exponent
        // magnitude than the result, so a result the format holds exactly is
        // reached without rounding.
        if through::<T>(expected) == expected {
            assert_bits::<T>(
                power::<T>(base, exponent),
                expected,
                &format!("{base}^{exponent}"),
            );
            exercised += 1;
        }
    }
    // `Bf4`'s grid {0.5, 1, 1.5, 2, 3} holds only the row 2^1; the others hold more.
    assert!(
        exercised >= 1,
        "{}: no row applies",
        core::any::type_name::<T>()
    );
}

fn check_zero_unit_and_special_bases<T: FormatRange>() {
    for base in [1.0, -1.0, 2.0, -2.0, 0.0, -0.0] {
        assert_bits::<T>(power::<T>(base, 0), 1.0, &format!("{base}^0"));
        assert_bits::<T>(power::<T>(base, 1), base, &format!("{base}^1"));
    }
    let nan = |exponent| FloatElement::nth_power(<T as NumericElement>::NAN, exponent);
    assert_bits::<T>(NumericElement::to_f64(nan(0)), 1.0, "NaN^0");
    for exponent in [1, 2, 3, 1 << 31, u32::MAX] {
        assert!(NumericElement::is_nan(nan(exponent)), "NaN^{exponent}");
    }
    // The finite-only formats have no infinity to use as a base.
    if T::OVERFLOW.is_infinite() {
        for base in [f64::INFINITY, f64::NEG_INFINITY] {
            assert_bits::<T>(power::<T>(base, 0), 1.0, &format!("{base}^0"));
            for exponent in [1, 2, 3, u32::MAX - 1, u32::MAX] {
                assert_bits::<T>(
                    power::<T>(base, exponent),
                    f64::INFINITY * parity_sign(base < 0.0, exponent),
                    &format!("{base}^{exponent}"),
                );
            }
        }
    }
}

fn check_zero_base_sign<T: FormatRange>() {
    for exponent in [7, 8, u32::MAX - 1, u32::MAX] {
        for base in [0.0_f64, -0.0] {
            assert_bits::<T>(
                power::<T>(base, exponent),
                0.0 * parity_sign(base.is_sign_negative(), exponent),
                &format!("{base}^{exponent}"),
            );
        }
    }
}

fn check_negative_unit_parity<T: FormatRange>() {
    for exponent in LARGE_EXPONENTS {
        assert_bits::<T>(
            power::<T>(-1.0, exponent),
            parity_sign(true, exponent),
            &format!("(-1)^{exponent}"),
        );
        assert_bits::<T>(power::<T>(1.0, exponent), 1.0, &format!("1^{exponent}"));
    }
}

fn check_overflow<T: FormatRange>() {
    let top = u32::try_from(T::MAX_EXPONENT).expect("invariant: positive");
    assert_bits::<T>(
        power::<T>(2.0, top),
        power_of_two(T::MAX_EXPONENT),
        "2^MAX_EXPONENT is finite",
    );
    assert_bits::<T>(
        power::<T>(2.0, top + 1),
        T::OVERFLOW,
        "2^(MAX_EXPONENT + 1)",
    );
    for exponent in LARGE_EXPONENTS {
        assert_bits::<T>(
            power::<T>(2.0, exponent),
            T::OVERFLOW,
            &format!("2^{exponent}"),
        );
        assert_bits::<T>(
            power::<T>(-2.0, exponent),
            T::OVERFLOW * parity_sign(true, exponent),
            &format!("(-2)^{exponent}"),
        );
    }
}

fn check_underflow<T: FormatRange>() {
    let least = u32::try_from(T::MIN_SUBNORMAL_EXPONENT).expect("invariant: positive");
    assert_bits::<T>(
        power::<T>(0.5, least),
        power_of_two(-T::MIN_SUBNORMAL_EXPONENT),
        "0.5^least is the least positive value",
    );
    // Half the least positive value is a tie between it and zero; ties go to even.
    assert_bits::<T>(power::<T>(0.5, least + 1), 0.0, "0.5^(least + 1)");
    assert_bits::<T>(
        power::<T>(-0.5, least + 1),
        0.0 * parity_sign(true, least + 1),
        "(-0.5)^(least + 1)",
    );
    for exponent in LARGE_EXPONENTS {
        assert_bits::<T>(power::<T>(0.5, exponent), 0.0, &format!("0.5^{exponent}"));
        assert_bits::<T>(
            power::<T>(-0.5, exponent),
            0.0 * parity_sign(true, exponent),
            &format!("(-0.5)^{exponent}"),
        );
    }
}

/// The relative error of `(1 + 2u)^n`, one ulp above one, stays within the
/// documented bound `(1 + u)^(n−1) − 1`, against the `f64` reference
/// `exp(n·ln_1p(2u))`. Returns whether the row applied.
///
/// The reference argument `A = n·L` rounds `ln_1p` (within one `f64` ulp, at
/// most `EPSILON` relative) and the product (at most `EPSILON / 2`), so it
/// carries relative error at most `2·EPSILON`; `exp` adds one ulp. The
/// reference is then within `e^a (1 + EPSILON) − 1` of the exact value, where
/// `a = 2·A·EPSILON`, which is the slack allowed beyond the bound. Evaluating
/// the bound and the observed error adds a few `EPSILON` of their own size, far
/// below the slack's `EPSILON` term wherever the rows leave a margin.
fn check_error_bound<T: FormatRange>(exponent: u32) -> bool {
    let unit_roundoff = power_of_two(-T::SIGNIFICAND_BITS);
    let base = 1.0 + 2.0 * unit_roundoff;
    let argument = f64::from(exponent) * (2.0 * unit_roundoff).ln_1p();
    let reference = argument.exp();
    let bound = (f64::from(exponent - 1) * unit_roundoff.ln_1p()).exp_m1();
    let largest = if T::OVERFLOW.is_finite() {
        T::OVERFLOW
    } else {
        power_of_two(T::MAX_EXPONENT)
    };
    // The computed result is within `bound` of the reference and exceeds every
    // partial power, so none overflows while it stays below the largest value.
    if reference * (1.0 + bound) >= largest {
        return false;
    }
    let observed = ((power::<T>(base, exponent) - reference) / reference).abs();
    let argument_error = 2.0 * argument * f64::EPSILON;
    let slack = argument_error.exp_m1() + f64::EPSILON * argument_error.exp();
    assert!(
        observed <= bound + slack,
        "{}: n = {exponent}: error {observed:e} exceeds bound {bound:e} + {slack:e}",
        core::any::type_name::<T>()
    );
    true
}

fn check_error_bound_sweep<T: FormatRange>() {
    let mut exercised = 0;
    for exponent in (2..=40).chain([100, 1000, 2050, 1 << 20, (1 << 24) + 1, 1 << 25]) {
        exercised += u32::from(check_error_bound::<T>(exponent));
    }
    assert!(
        exercised >= 1,
        "{}: no row applies",
        core::any::type_name::<T>()
    );
}

#[test]
fn small_powers_are_exact() {
    every_format!(check_small_powers);
}

#[test]
fn range_facts_agree_with_the_conversion() {
    every_format!(check_range_facts);
}

#[test]
fn zero_exponent_gives_one_and_unit_exponent_gives_the_base() {
    every_format!(check_zero_unit_and_special_bases);
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
fn overflow_reaches_the_signed_largest_value_by_parity() {
    every_format!(check_overflow);
}

#[test]
fn underflow_reaches_the_least_positive_value_then_signed_zero() {
    every_format!(check_underflow);
}

#[test]
fn error_stays_within_the_bound_where_the_gamma_form_is_undefined() {
    // The product count n - 1 reaches 1/u: f32 past 2^24, F16 past 2^11, Bf8 past 8.
    assert!(check_error_bound::<f32>((1 << 24) + 1));
    assert!(check_error_bound::<F32>(1 << 25));
    assert!(check_error_bound::<Bf8>(10));
    assert!(check_error_bound::<F16>(2050));
}

#[test]
fn error_stays_within_the_bound_for_every_format() {
    every_format!(check_error_bound_sweep);
}
