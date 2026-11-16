//! Contract of [`FloatElement::powi`] for a negative exponent `-e` whose upward power `|x|^e` or
//! result `y = |x|^-e` is subnormal: finite and within `B_e·|y| + s/2` of `y`, with
//! `B_e = (1 - u)^-e - 1`, `u` the unit roundoff and `s` the least positive value
//! (`traits/float/power.rs` derives it), whenever `y(1 + B_e)` stays below the overflow threshold
//! `MAX + ulp(MAX)/2`; the sweeps select only such rows, and the board item's pinned rows lie
//! above it.
//!
//! The oracle is `1/|x|^e` in `f64`: `e - 1` products and a division, each within `2^-53` relative,
//! so the tolerance adds `(e + 1)·2^-53·y`, above the `f64` evaluation error of `B_e`. The exact
//! `y` of the pinned rows are `2^24/343` (`Bf8`, `x = 7·2^-8`, `e = 3`), `2^198/10568984^3`
//! (`f32`, `x = 10568984·2^-66`, `e = 3`) and `2^36/1062961` (`F16`, `x = 1031·2^-18`, `e = 2`).

use core::ops::RangeInclusive;

use eunomia::{Bf8, FloatElement, NumericElement, F16, F8};

use super::integer_power::{make, power_of_two};

/// Unit roundoff of the `f64` oracle.
const ORACLE_ROUNDOFF: f64 = f64::EPSILON / 2.0;
/// Roundings of the oracle beyond its `e - 1` products: the division and the one of `B_e`.
const ORACLE_EXTRA_ROUNDINGS: f64 = 2.0;
/// Largest `e` swept, and the `f32` bit patterns sampled per exponent.
const EXPONENT_LIMIT: u32 = 24;
const F32_SAMPLES: u32 = 4000;

/// The parameters of an IEEE-style format that the bound reads.
struct Format {
    /// Significand bits `p`, so `u = 2^-p`.
    precision: i32,
    /// Largest unbiased exponent; the least normal exponent is `1 - exponent_max`.
    exponent_max: i32,
    /// `-σ` with `s = 2^-σ`.
    least_positive_exponent: i32,
}

const BF8_FORMAT: Format = Format::new(3, 15, -16);
const F8_FORMAT: Format = Format::new(4, 7, -9);
const F16_FORMAT: Format = Format::new(11, 15, -24);
const F32_FORMAT: Format = Format::new(24, 127, -149);

impl Format {
    const fn new(precision: i32, exponent_max: i32, least_positive_exponent: i32) -> Self {
        Self {
            precision,
            exponent_max,
            least_positive_exponent,
        }
    }

    fn roundoff(&self) -> f64 {
        power_of_two(-self.precision)
    }

    fn least_normal(&self) -> f64 {
        power_of_two(1 - self.exponent_max)
    }

    /// `B_e = (1 - u)^-e - 1`.
    fn relative_bound(&self, exponent: u32) -> f64 {
        (1.0 - self.roundoff()).powi(-signed(exponent)) - 1.0
    }

    /// `B_e·y + s/2` plus the oracle's own rounding.
    fn tolerance(&self, exponent: u32, exact: f64) -> f64 {
        self.relative_bound(exponent) * exact
            + power_of_two(self.least_positive_exponent - 1)
            + exact * (f64::from(exponent) + ORACLE_EXTRA_ROUNDINGS) * ORACLE_ROUNDOFF
    }

    /// Whether `y(1 + B_e)` is below `MAX + ulp(MAX)/2 = (2 - 2^-p)·2^emax`, so that no overflow
    /// is reachable.
    fn below_overflow(&self, exponent: u32, exact: f64) -> bool {
        let threshold = (2.0 - self.roundoff()) * power_of_two(self.exponent_max);
        exact * (1.0 + self.relative_bound(exponent)) < threshold
    }

    /// A base below one whose upward power `|x|^e` is under twice the least normal value.
    fn subnormal_upward(&self, base: f64, exponent: u32, upward: f64) -> bool {
        base.abs() < 1.0
            && upward < 2.0 * self.least_normal()
            && self.below_overflow(exponent, 1.0 / upward)
    }

    /// A base above one with a finite, unclamped upward power and a subnormal exact result.
    fn subnormal_result(&self, base: f64, exponent: u32, upward: f64) -> bool {
        base.abs() > 1.0
            && self.below_overflow(exponent, upward)
            && 1.0 / upward < self.least_normal()
    }
}

fn signed(exponent: u32) -> i32 {
    i32::try_from(exponent).expect("invariant: small exponent")
}

/// `powi(base, -exponent)` asserted finite, of the right sign and within the bound; returns the
/// magnitude.
fn check_row<T: FloatElement>(format: &Format, base: f64, exponent: u32) -> f64 {
    let got = NumericElement::to_f64(FloatElement::powi(make::<T>(base), -signed(exponent)));
    let exact = 1.0 / base.abs().powi(signed(exponent));
    let what = format!(
        "{}: powi({base}, -{exponent}) = {got:e} for exact {exact:e}",
        core::any::type_name::<T>()
    );
    assert_eq!(
        got.is_sign_negative(),
        base < 0.0 && exponent % 2 == 1,
        "sign of {what}"
    );
    let (error, tolerance) = ((got.abs() - exact).abs(), format.tolerance(exponent, exact));
    assert!(
        error <= tolerance,
        "{what}: error {error:e} over {tolerance:e}"
    );
    got.abs()
}

/// A row selector: the format, the base, the exponent `e` and the upward power `|base|^e`.
type Select = fn(&Format, f64, u32, f64) -> bool;

/// Check every row of both signs of the positive finite `values` and `exponents` that `select`
/// accepts; returns the row count.
fn check_rows<T: FloatElement>(
    format: &Format,
    values: impl Iterator<Item = T>,
    exponents: RangeInclusive<u32>,
    select: Select,
) -> usize {
    let bases: Vec<f64> = values
        .map(NumericElement::to_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .flat_map(|value| [value, -value])
        .collect();
    let mut rows = 0;
    for exponent in exponents {
        for &base in &bases {
            if select(format, base, exponent, base.abs().powi(signed(exponent))) {
                check_row::<T>(format, base, exponent);
                rows += 1;
            }
        }
    }
    rows
}

/// Sweep `select` over every code of `Bf8`, `F8` and `F16` for `e ≤ 24`, and over strided `f32`
/// values with `|x|^e` between `2^low` and `2^high` for `2 ≤ e ≤ 24`; each of the four must
/// exceed its `minimum` row count, in that order.
fn sweep(select: Select, (low, high): (i32, i32), minimum: [usize; 4]) {
    let exponents = 1..=EXPONENT_LIMIT;
    let narrow = [
        check_rows(
            &BF8_FORMAT,
            (0..=u8::MAX).map(Bf8),
            exponents.clone(),
            select,
        ),
        check_rows(&F8_FORMAT, (0..=u8::MAX).map(F8), exponents.clone(), select),
        check_rows(&F16_FORMAT, (0..=u16::MAX).map(F16), exponents, select),
    ];
    let f32_rows = (2..=EXPONENT_LIMIT).map(|exponent| {
        let edge = |bits| make::<f32>(2.0_f64.powf(f64::from(bits) / f64::from(exponent)));
        let (first, last) = (edge(low).to_bits(), edge(high).to_bits());
        let stride = usize::try_from((last - first) / F32_SAMPLES).expect("invariant: u32");
        let values = (first..last).step_by(stride.max(1)).map(f32::from_bits);
        check_rows(&F32_FORMAT, values, exponent..=exponent, select)
    });
    let rows = narrow.into_iter().chain([f32_rows.sum()]);
    for (rows, minimum) in rows.zip(minimum) {
        assert!(rows > minimum, "{rows} rows, at most {minimum}");
    }
}

#[test]
fn powi_negative_exponent_with_subnormal_upward_power_stays_within_bound() {
    // `Bf8` and `f32` returned `+inf` and `F16` left the bound at these rows at the base revision.
    check_row::<Bf8>(&BF8_FORMAT, 7.0 * power_of_two(-8), 3);
    check_row::<f32>(&F32_FORMAT, f64::from(f32::from_bits(0x2a21_4518)), 3);
    check_row::<F16>(&F16_FORMAT, 1031.0 * power_of_two(-18), 2);
    sweep(
        Format::subnormal_upward,
        (-128, -125),
        [30, 100, 10_000, 10_000],
    );
}

#[test]
fn powi_negative_exponent_with_subnormal_result_stays_within_bound() {
    // `224` is normal and finite; `1/224 = 4.464e-3` lies between the subnormals `2·2^-9` and
    // `3·2^-9` and rounds to `2^-8`: error `5.58e-4`, over `B_1·y = 2.98e-4` and within `s/2`.
    let got = check_row::<F8>(&F8_FORMAT, 224.0, 1);
    assert_eq!(got.to_bits(), power_of_two(-8).to_bits(), "powi(224, -1)");
    assert!((got - 1.0 / 224.0).abs() > F8_FORMAT.relative_bound(1) / 224.0);
    sweep(
        Format::subnormal_result,
        (126, 128),
        [20, 50, 10_000, 10_000],
    );
}
