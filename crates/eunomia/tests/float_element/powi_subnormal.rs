//! Contract of [`FloatElement::powi`] for a negative exponent `-e` whose upward power `|x|^e` or
//! result `y = |x|^-e` is subnormal. With `u` the unit roundoff, `s` the least positive value,
//! `B_e = (1 - u)^-e - 1` and `T = (2 - u)·2^emax = MAX + ulp(MAX)/2` the overflow threshold, the
//! result (`traits/float/power.rs` derives it) is within `B_e·y + s/2` of `y` where
//! `y(1 + B_e) < T`; infinite (the largest finite value on a finite-only format) where
//! `y(1 - B_e) ≥ T`; either between, within the bound unless it is that overflow value.
//!
//! Oracle. For `x = ±m·2^k`, `m` in `[1, 2)`, `y = ζ·2^-ke` and `ζ = m^-e` is formed in `f64` as
//! `1/m^e`: `e - 1` products and a division, `e` roundings within `a = 2^-53`, so `ζ̃ = ζρ` with
//! `|ρ - 1| ≤ ε = 2·e·a` (`(1 - a)^-e ≤ 1/(1 - e·a) ≤ 1 + ε` for `e·a ≤ 1/2`). Forming
//! `q̃ = (1 - u)^-e` alike gives `B_e ≤ B̃ + 2εq̃` for `B̃ = q̃ - 1`, so
//! `|ŷ - ỹ| ≤ (1 + 2ε)(B̃ + 2εq̃ + ε)·ỹ + s/2 = (B̃ + 4εq̃ - ε + (4q̃ + 2)ε²)·ỹ + s/2`, at most
//! `(B̃ + 4εq̃)·ỹ + s/2` when `(4q̃ + 2)ε ≤ 1`, and `ỹ(1 ± τ)` brackets the pre-overflow value for
//! `τ = B̃ + 8εq̃`. The extra `4εq̃ ≥ 8a·τ` (as `τ ≤ q̃` when `8εq̃ ≤ 1`) covers the at most `7a` of
//! roundings in forming `τ`, `τ·ỹ + s/2` and the comparisons. Both conditions hold on the rows
//! below (`q̃ < 100`, `ε < 10^-12`); `Row::new` asserts `e·log2(m) < 1024`. For `f64` and `F64`,
//! `a = u`, so the check holds `y` to about `17·B_e` there.
//!
//! Pinned rows (exact rationals). `Bf8` `powi(7·2^-8, -3)` has `y = 2^24/343 = 48913.2` and
//! `y(1 ± B_3)` = 73013, 24813 around `T = 61440`; the `f32` row `powi(0x2a214518, -3)` has
//! `y = 3.4028234746e38` and 3.402824083e38, 3.402822866e38 around `T = 3.4028235678e38`. Both lie
//! in the overflow band, returned `+inf` at the base and are held finite. `F16`
//! `powi(1031·2^-18, -2)` has `y = 2^36/1062961 = 64649.1` and `y(1 + B_2) = 64712 < 65520`:
//! finite and in bound at the head (64512 at the base, 137 off).

use eunomia::{Bf8, FloatElement, NumericElement, F16, F8};

use super::integer_power::{largest_exponent, make, power_of_two};

/// Unit roundoff `a` of the `f64` oracle, and the multiple of `e·a` in `τ`: `8·ε`, `ε = 2·e·a`.
const ORACLE_ROUNDOFF: f64 = f64::EPSILON / 2.0;
const ORACLE_ROUNDOFFS: f64 = 16.0;

/// The parameters of a format that the bound reads: significand bits `p`, so `u = 2^-p`, and the
/// largest unbiased exponent, whose negation plus one is the least normal exponent.
struct Format {
    precision: i32,
    exponent_max: i32,
}

impl Format {
    /// `T`'s parameters: `1 + 3·ulp/4` rounds up from one exactly when `ulp = 2^-(p-1)` or more.
    fn of<T: FloatElement>() -> Self {
        let rounded = |value: f64| NumericElement::to_f64(make::<T>(value));
        let ulp_exponent = (0..64)
            .rev()
            .find(|&j| rounded(1.0 + 0.75 * power_of_two(-j)) != 1.0);
        let precision = ulp_exponent.expect("invariant: 1.75 is not one") + 1;
        let format = Self {
            precision,
            exponent_max: largest_exponent::<T>(),
        };
        let least = power_of_two(format.least_positive_exponent());
        assert_eq!(
            (rounded(least), rounded(least / 2.0)),
            (least, 0.0),
            "least positive"
        );
        format
    }

    fn roundoff(&self) -> f64 {
        power_of_two(-self.precision)
    }

    /// `s = 2^(2 - emax - p)`, the least positive value.
    fn least_positive_exponent(&self) -> i32 {
        2 - self.exponent_max - self.precision
    }

    /// The overflow threshold in units of `2^emax`.
    fn threshold(&self) -> f64 {
        2.0 - self.roundoff()
    }
}

fn signed(exponent: u32) -> i32 {
    i32::try_from(exponent).expect("invariant: small exponent")
}

fn scale(value: f64, exponent: i32) -> f64 {
    FloatElement::scale_binary(value, exponent)
}

/// `base^exponent` by `exponent - 1` products.
fn power(base: f64, exponent: u32) -> f64 {
    (1..exponent).fold(base, |power, _| power * base)
}

/// A base `±m·2^k` and exponent `e` in the scaled domain: `|x|^e = upward·2^shift` and
/// `y = reciprocal·2^-shift` with `shift = k·e`.
struct Row {
    exponent: u32,
    shift: i32,
    reciprocal: f64,
}

impl Row {
    fn new(base: f64, exponent: u32) -> Self {
        let k = FloatElement::binary_exponent(base).expect("invariant: finite nonzero base");
        let upward = power(scale(base.abs(), -k), exponent);
        assert!(upward.is_finite(), "invariant: e·log2(m) < 1024");
        Self {
            exponent,
            shift: k * signed(exponent),
            reciprocal: 1.0 / upward,
        }
    }

    /// `τ = B̃ + 16·e·a·q̃`: the bound relative to `y`, with the oracle's own rounding.
    fn relative_bound(&self, format: &Format) -> f64 {
        let q = 1.0 / power(1.0 - format.roundoff(), self.exponent);
        q - 1.0 + ORACLE_ROUNDOFFS * f64::from(self.exponent) * ORACLE_ROUNDOFF * q
    }
}

/// Assert `powi(base, -exponent)` has the sign and magnitude the contract allows; returns the
/// magnitude.
fn check_row<T: FloatElement>(format: &Format, base: f64, exponent: u32) -> f64 {
    let row = Row::new(base, exponent);
    let got = NumericElement::to_f64(FloatElement::powi(make::<T>(base), -signed(exponent)));
    let name = core::any::type_name::<T>();
    let what = || format!("{name}: powi({base}, -{exponent}) = {got:e}");
    let negative = base < 0.0 && exponent % 2 == 1;
    assert_eq!(got.is_sign_negative(), negative, "sign of {}", what());
    let bound = row.relative_bound(format);
    let top = scale(row.reciprocal, -row.shift - format.exponent_max);
    let overflow = NumericElement::to_f64(<T as NumericElement>::INFINITY);
    if top * (1.0 - bound) >= format.threshold() {
        assert_eq!(got.abs(), overflow, "overflow expected: {}", what());
    } else if got.abs() != overflow || top * (1.0 + bound) < format.threshold() {
        let least = scale(1.0, row.shift + format.least_positive_exponent() - 1);
        let error = (scale(got.abs(), row.shift) - row.reciprocal).abs();
        let allowed = row.reciprocal * bound + least;
        assert!(
            error <= allowed,
            "{}: error {error:e} over {allowed:e}",
            what()
        );
    }
    got.abs()
}

#[test]
fn powi_negative_exponent_with_subnormal_upward_power_stays_within_bound() {
    // `Bf8` and `f32` returned `+inf` and `F16` left the bound at these rows at the base revision.
    let bf8 = check_row::<Bf8>(&Format::of::<Bf8>(), 7.0 * power_of_two(-8), 3);
    let wide = f64::from(f32::from_bits(0x2a21_4518));
    let wide = check_row::<f32>(&Format::of::<f32>(), wide, 3);
    check_row::<F16>(&Format::of::<F16>(), 1031.0 * power_of_two(-18), 2);
    assert!(
        bf8.is_finite() && wide.is_finite(),
        "Bf8 {bf8:e}, f32 {wide:e}"
    );
}

#[test]
fn powi_negative_exponent_with_subnormal_result_stays_within_bound() {
    // `1/224 = 4.464e-3` rounds to `2^-8`: error `5.58e-4`, over `B_1·y = 2.98e-4`, within `s/2`.
    let got = check_row::<F8>(&Format::of::<F8>(), 224.0, 1);
    assert_eq!(got.to_bits(), power_of_two(-8).to_bits(), "powi(224, -1)");
    let relative_bound = 1.0 / (1.0 - power_of_two(-4)) - 1.0;
    assert!((got - 1.0 / 224.0).abs() > relative_bound / 224.0);
}
