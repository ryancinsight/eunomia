//! Contract of [`FloatElement::powi`] for a negative exponent on the finite-only formats `Bf4`,
//! `F8` and `F4`, whose overflowing product clamps to the largest finite value. A clamped upward
//! power `|x|^e` is not `x^e` (`F8` `powi(2, -9)`: `512` clamps to `240`, `1/240` rounds to
//! `2^-8`); the result must be the exact `x^-e` rounded once into the format.
//!
//! Two independent references check every saturated row:
//!
//! * an exact integer oracle. A base is `±m·2^κ` with `m` odd, so `|x^-e| / s = 2^a / m^e` with
//!   `s = 2^-σ` the least positive value and `a = σ − κ·e`; the quotient lies above the midpoint
//!   `(c + c')·s/2` of grid neighbours `c·s`, `c'·s` exactly when `2^(a+1) > (c + c')·m^e`,
//!   decided in arbitrary precision. That never ties for `m > 1` (`m^e` is odd); for `m = 1` the
//!   quotient is a power of two and the format's `from_f64` rounds it.
//! * the exact power in `f64`, for `e ≤ 9` (`m^e < 2^36`): one correctly rounded division, then
//!   `from_f64`. The two roundings agree because a quotient with `m > 1` is at least
//!   `1/((c + c')·m^e) ≥ 15^-10 > 2^-40` from a midpoint, above the `2^-53` the division moves it.
//!
//! Past `e = 128` the result is a signed zero (`(9/8)^e > 2^21`). Observations are `f64` widenings
//! compared by bit pattern, so the sign of zero is asserted; no assertion carries a tolerance.

use eunomia::{Bf4, FloatElement, NumericElement, F16, F4, F8};

use super::integer_power::{assert_bits, make, power_of_two};

/// Largest `|n|` with an exact integer oracle row.
const ORACLE_EXPONENT_LIMIT: u32 = 128;
/// Largest `|n|` with an exact `f64` power for every base: `m^9 < 2^36 ≤ 2^53`.
const WIDE_EXACT_EXPONENT_LIMIT: u32 = 9;
/// Exponents past the oracle limit, up to `i32::MIN`.
const FAR_EXPONENTS: [i32; 7] = [
    -129,
    -200,
    -(1 << 23),
    -(1 << 24),
    -16_777_217,
    -i32::MAX,
    i32::MIN,
];

/// Little-endian base-`2^32` limbs of a positive integer.
struct BigUint(Vec<u32>);

impl BigUint {
    fn one() -> Self {
        Self(vec![1])
    }

    fn mul_small(&mut self, factor: u32) {
        let mut carry = 0_u64;
        for limb in &mut self.0 {
            let product = u64::from(*limb) * u64::from(factor) + carry;
            *limb =
                u32::try_from(product & u64::from(u32::MAX)).expect("invariant: masked to 32 bits");
            carry = product >> 32;
        }
        if carry > 0 {
            self.0
                .push(u32::try_from(carry).expect("invariant: carry below 2^32"));
        }
    }

    /// The number of bits of the value, `⌊log2(self)⌋ + 1`.
    fn bit_length(&self) -> i64 {
        let top = *self.0.last().expect("invariant: at least one limb");
        let limbs = i64::try_from(self.0.len()).expect("invariant: few limbs");
        32 * limbs - i64::from(top.leading_zeros())
    }
}

/// `value` is exactly representable in `T`.
fn representable<T: FloatElement>(value: f64) -> bool {
    NumericElement::to_f64(make::<T>(value)) == value
}

/// `σ` with `2^-σ` the least positive value of `T`: the largest `k` with `2^-k` held (half of it
/// is a tie that rounds to zero).
fn least_positive_exponent<T: FloatElement>() -> i32 {
    (0..=30)
        .rev()
        .find(|&k| NumericElement::to_f64(make::<T>(power_of_two(-k))) != 0.0)
        .expect("invariant: every format holds 2^0")
}

/// `value = significand · 2^exponent` with `significand` odd, for a normal `f64`.
fn odd_significand(value: f64) -> (u32, i32) {
    let bits = value.abs().to_bits();
    let field = i32::try_from(bits >> 52).expect("invariant: 11-bit field");
    assert!(field > 0, "invariant: a normal f64");
    let significand = (1_u64 << 52) | (bits & ((1_u64 << 52) - 1));
    let trailing = significand.trailing_zeros();
    (
        u32::try_from(significand >> trailing).expect("invariant: base significand below 2^32"),
        field - 1075 + i32::try_from(trailing).expect("invariant: at most 52"),
    )
}

/// The exact `x^-e` rounded once into `T`, as an `f64` magnitude, by the
/// integer oracle. `x > 1` and `e ≤ ORACLE_EXPONENT_LIMIT`.
fn oracle_magnitude<T: FloatElement>(base: f64, exponent: u32) -> f64 {
    let (significand, power) = odd_significand(base);
    let sigma = least_positive_exponent::<T>();
    let exponent_signed = i32::try_from(exponent).expect("invariant: small exponent");
    if significand == 1 {
        let shift = power
            .checked_mul(exponent_signed)
            .expect("invariant: small product");
        return NumericElement::to_f64(make::<T>(power_of_two(-shift)));
    }
    let mut odd_power = BigUint::one();
    for _ in 0..exponent {
        odd_power.mul_small(significand);
    }
    // |x^-e| / s = 2^a / odd_power.
    let a = i64::from(sigma) - i64::from(power) * i64::from(exponent_signed);
    let grid: Vec<u32> = (0..=8_u32)
        .filter(|&k| representable::<T>(f64::from(k) * power_of_two(-sigma)))
        .collect();
    let above_midpoint = |lower: u32, upper: u32| {
        let mut scaled = BigUint(odd_power.0.clone());
        scaled.mul_small(lower + upper);
        a + 1 >= scaled.bit_length()
    };
    let mut index = 0;
    while index + 1 < grid.len() && above_midpoint(grid[index], grid[index + 1]) {
        index += 1;
    }
    assert!(
        index + 1 < grid.len(),
        "{}: oracle grid exhausted at {base}^-{exponent}",
        core::any::type_name::<T>()
    );
    f64::from(grid[index]) * power_of_two(-sigma)
}

/// The exact `x^-e` as an `f64` magnitude rounded into `T`, for `e` with an
/// exact `f64` power: one `f64` division, then the format's conversion.
fn wide_route_magnitude<T: FloatElement>(base: f64, exponent: u32) -> f64 {
    let power = (0..exponent).fold(1.0_f64, |product, _| product * base);
    NumericElement::to_f64(make::<T>(1.0 / power))
}

/// Every finite base of `T` with magnitude above one, both signs.
fn bases_above_one<T: FloatElement>() -> Vec<f64> {
    let mut bases = Vec::new();
    for significand in (1..=15_i32).step_by(2) {
        for exponent in -12..=8 {
            let magnitude = f64::from(significand) * power_of_two(exponent);
            if magnitude > 1.0 && representable::<T>(magnitude) {
                bases.extend([magnitude, -magnitude]);
            }
        }
    }
    bases
}

/// The upward power formed in `T` is the format's clamped largest value.
fn upward_saturates<T: FloatElement>(base: f64, exponent: u32) -> bool {
    let upward = NumericElement::to_f64(FloatElement::nth_power(make::<T>(base), exponent));
    upward.abs() == NumericElement::to_f64(<T as NumericElement>::MAX_VALUE)
}

fn with_parity_sign(base: f64, exponent: u32, magnitude: f64) -> f64 {
    if base < 0.0 && exponent % 2 == 1 {
        -magnitude
    } else {
        magnitude
    }
}

/// Every saturated row of `T` equals both references.
fn check_saturated_sweep<T: FloatElement>() {
    let name = core::any::type_name::<T>();
    assert!(
        NumericElement::to_f64(<T as NumericElement>::INFINITY).is_finite(),
        "{name}: a finite-only format"
    );
    let mut rows = 0_usize;
    for base in bases_above_one::<T>() {
        for exponent in 1..=ORACLE_EXPONENT_LIMIT {
            if !upward_saturates::<T>(base, exponent) {
                continue;
            }
            rows += 1;
            let magnitude = oracle_magnitude::<T>(base.abs(), exponent);
            if exponent <= WIDE_EXACT_EXPONENT_LIMIT {
                assert_bits::<T>(
                    magnitude,
                    wide_route_magnitude::<T>(base.abs(), exponent),
                    &format!("oracle against the f64 route at {}^-{exponent}", base.abs()),
                );
            }
            let n = -i32::try_from(exponent).expect("invariant: small exponent");
            assert_bits::<T>(
                NumericElement::to_f64(FloatElement::powi(make::<T>(base), n)),
                with_parity_sign(base, exponent, magnitude),
                &format!("powi({base}, {n})"),
            );
        }
        for n in FAR_EXPONENTS {
            let exponent = n.unsigned_abs();
            assert!(
                upward_saturates::<T>(base, exponent),
                "{name}: {base}^{exponent} saturates"
            );
            assert_bits::<T>(
                NumericElement::to_f64(FloatElement::powi(make::<T>(base), n)),
                with_parity_sign(base, exponent, 0.0),
                &format!("powi({base}, {n})"),
            );
        }
    }
    assert!(rows > 0, "{name}: no saturated row exercised");
}

/// Rows with a stated value: saturated rows that fail without the fix, and
/// rows whose upward power is the largest finite value without having
/// overflowed, which the fix must keep right.
fn check_rows<T: FloatElement>(rows: &[(f64, i32, f64)]) {
    for &(base, n, expected) in rows {
        assert_bits::<T>(
            NumericElement::to_f64(FloatElement::powi(make::<T>(base), n)),
            expected,
            &format!("powi({base}, {n})"),
        );
    }
}

#[test]
fn powi_negative_exponent_rounds_the_exact_power_when_upward_saturates() {
    check_saturated_sweep::<Bf4>();
    check_saturated_sweep::<F8>();
    check_saturated_sweep::<F4>();
    // 0.25 is half the least `Bf4` value, a tie to zero; 1.5^3 clamps to 3, 1/3.375 rounds to 0.5.
    check_rows::<Bf4>(&[(2.0, -2, 0.0), (1.5, -3, 0.5), (-1.5, -3, -0.5)]);
    // 2^-9 is the least `F8` value, 2^-10 a tie to zero; 2.25^7 clamps, 1/291.9 rounds to 2^-8.
    check_rows::<F8>(&[
        (2.0, -9, power_of_two(-9)),
        (2.0, -10, 0.0),
        (2.25, -7, power_of_two(-8)),
        (-2.25, -7, -power_of_two(-8)),
    ]);
    // 1/8 is half the least `F4` value and ties to even, zero.
    check_rows::<F4>(&[(2.0, -3, 0.0), (-2.0, -3, -0.0)]);
}

/// `powi` returns `result` for `base^n`; its error against `exact` exceeds the bound for `2^-bits`.
fn assert_outside_bound<T: FloatElement>(base: f64, n: i32, result: f64, exact: f64, bits: i32) {
    let roundoff = power_of_two(-bits);
    let bound = if n > 0 {
        (1.0 + roundoff).powi(n - 1)
    } else {
        (1.0 - roundoff).powi(n)
    } - 1.0;
    assert_bits::<T>(
        NumericElement::to_f64(FloatElement::powi(make::<T>(base), n)),
        result,
        &format!("powi({base}, {n})"),
    );
    assert!(
        (exact - result).abs() / exact > bound,
        "powi({base}, {n}) is within {bound}"
    );
}

/// The bound excludes subnormal and clamped products. `F16` `1031·2^-18` squares to the subnormal
/// `260·2^-24` (EUNOMIA-POWI-SUBNORMAL-UPWARD owns that regime); `F8` 30^2 and `Bf4` 3^2 clamp.
#[test]
fn powi_bound_excludes_subnormal_and_clamped_products() {
    let subnormal_exact = power_of_two(36) / 1_062_961.0;
    assert_outside_bound::<F16>(1031.0 * power_of_two(-18), -2, 64512.0, subnormal_exact, 11);
    assert_outside_bound::<F8>(30.0, 2, 240.0, 900.0, 4);
    assert_outside_bound::<Bf4>(3.0, 2, 3.0, 9.0, 2);
}
