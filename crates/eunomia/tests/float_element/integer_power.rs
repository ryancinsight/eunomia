//! Contract of [`FloatElement::powi`]: the base raised to a signed exponent
//! in the element type, including the reciprocal and underflow-preserving
//! paths of negative exponents.
//!
//! Two independent references. For `±2^k` the correctly rounded value is known
//! in closed form: `2^k` is an exact `f64` over `-1074..=1023`, and rounding it
//! into the format is the conversion, not the squaring loop. For other bases
//! the result is pinned bit for bit to the loop `powi` ran before its kernel
//! moved. Every observation is the exact `f64` widening of the result,
//! compared by bit pattern, so the sign of zero and of infinity is asserted
//! and no assertion carries a tolerance.

use eunomia::{Bf16, Bf4, Bf8, FloatElement, NumericElement, F16, F32, F4, F64, F8};

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

pub(super) fn make<T: FloatElement>(value: f64) -> T {
    <T as FloatElement>::from_f64(value)
}

/// `2^exponent` as an `f64`, built from its bit pattern: a normal for
/// `-1022 <= exponent <= 1023`, else the subnormal with that single bit set.
pub(super) fn power_of_two(exponent: i32) -> f64 {
    if exponent >= -1022 {
        let field = u64::try_from(1023 + exponent).expect("invariant: exponent at least -1022");
        f64::from_bits(field << 52)
    } else {
        let shift = u32::try_from(1074 + exponent).expect("invariant: exponent at least -1074");
        f64::from_bits(1_u64 << shift)
    }
}

pub(super) fn assert_bits<T>(actual: f64, expected: f64, what: &str) {
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "{}: {what}: got {actual:e}, expected {expected:e}",
        core::any::type_name::<T>()
    );
}

/// Exponent magnitudes at the edges of the shipped formats' normal, subnormal
/// and overflow ranges; each is used with both signs.
const EDGE_EXPONENTS: [i32; 24] = [
    0, 1, 2, 3, 7, 8, 15, 16, 17, 24, 25, 126, 127, 128, 133, 134, 149, 150, 1023, 1024, 1074,
    1075, 1076, 2000,
];

fn exponents() -> impl Iterator<Item = i32> + Clone {
    EDGE_EXPONENTS
        .into_iter()
        .flat_map(|edge| [edge, -edge])
        .chain([i32::MIN, i32::MIN + 1, i32::MAX - 1, i32::MAX])
}

/// `±2^k` rounded to nearest even into `T`, seen as an `f64`.
fn rounded_power_of_two<T: FloatElement>(negative: bool, exponent: i32) -> f64 {
    // Past `f64`'s range the magnitude is infinite above it and, below it, no
    // more than the tie under `f64`'s least subnormal, which rounds to zero in
    // every shipped format.
    let magnitude = match exponent {
        -1074..=1023 => power_of_two(exponent),
        1024.. => f64::INFINITY,
        _ => 0.0,
    };
    NumericElement::to_f64(make::<T>(if negative { -magnitude } else { magnitude }))
}

fn check_powers_of_two<T: FloatElement>() {
    // A finite-only format saturates an overflowing power instead of reaching
    // infinity, so `powi` inverts the saturated value for a negative
    // exponent; the closed form holds for those only while `2^|k|` is exact.
    let infinite = NumericElement::to_f64(make::<T>(f64::INFINITY)).is_infinite();
    for exponent in exponents() {
        let magnitude = exponent.saturating_abs();
        let exact = magnitude <= 1023
            && rounded_power_of_two::<T>(false, magnitude) == power_of_two(magnitude);
        for negative in [false, true] {
            if exponent < 0 && !exact && !infinite {
                continue;
            }
            let base = make::<T>(if negative { -2.0 } else { 2.0 });
            assert_bits::<T>(
                NumericElement::to_f64(FloatElement::powi(base, exponent)),
                rounded_power_of_two::<T>(negative && exponent % 2 != 0, exponent),
                &format!("{}2^{exponent}", if negative { "-" } else { "" }),
            );
        }
    }
}

/// `powi` as it stood before its kernel moved: the squaring loop forms one
/// square past the last exponent bit.
fn previous_powi<T: FloatElement>(base: T, n: i32) -> T {
    fn upward<T: FloatElement>(mut base: T, mut exponent: u32) -> T {
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
    let exponent = n.unsigned_abs();
    let accumulator = upward(base, exponent);
    if n >= 0 {
        accumulator
    } else if !NumericElement::is_finite(accumulator) && !NumericElement::is_nan(accumulator) {
        upward(<T as NumericElement>::ONE / base, exponent)
    } else {
        <T as NumericElement>::ONE / accumulator
    }
}

fn check_previous_powi<T: FloatElement>() {
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
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    for base in bases {
        let base = make::<T>(base);
        for exponent in exponents().chain(-70..=70) {
            let got = NumericElement::to_f64(FloatElement::powi(base, exponent));
            let before = NumericElement::to_f64(previous_powi(base, exponent));
            assert!(
                got.to_bits() == before.to_bits() || (got.is_nan() && before.is_nan()),
                "{}: {}^{exponent}: got {got:e}, previous loop {before:e}",
                core::any::type_name::<T>(),
                NumericElement::to_f64(base)
            );
        }
    }
}

#[test]
fn powi_of_two_is_the_rounded_power_for_every_exponent_sign() {
    every_format!(check_powers_of_two);
}

#[test]
fn powi_equals_the_loop_it_replaced_bit_for_bit() {
    every_format!(check_previous_powi);
}
