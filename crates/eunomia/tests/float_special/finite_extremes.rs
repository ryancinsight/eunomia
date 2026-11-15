//! `RealField::{min_value, max_value}` are the extreme *finite* values.
//!
//! Distinct from the reduction identities `NumericElement::{MIN_VALUE,
//! MAX_VALUE}` (`-inf`/`+inf` for floats), which seed Min/Max folds so that an
//! infinite operand still reduces to itself. The oracle is the format's own
//! largest finite magnitude, `(2 - 2^-p) * 2^emax` (IEEE 754-2019 section 3.3),
//! which std exposes as `MAX`; every comparison is exact, so no tolerance.

use eunomia::{NumericElement, RealField, F32, F64};

/// Asserts the finite-extreme contract for one scalar against the format's
/// largest finite value `largest` (which equals `-smallest`).
fn assert_finite_extremes<T: RealField + core::fmt::Debug>(largest: T) {
    let max = <T as RealField>::max_value();
    let min = <T as RealField>::min_value();
    assert_eq!(max, largest, "max_value is the largest finite value");
    assert_eq!(min, -largest, "min_value is the smallest finite value");
    assert!(NumericElement::is_finite(max), "max_value is finite");
    assert!(NumericElement::is_finite(min), "min_value is finite");
    assert!(max < <T as RealField>::infinity(), "max_value < +inf");
    assert!(min > <T as RealField>::neg_infinity(), "min_value > -inf");
    // Every finite value lies in [min_value, max_value]; one ulp beyond rounds
    // to infinity, which is not finite.
    assert!(
        !NumericElement::is_finite(max + max),
        "doubling max_value overflows to a non-finite value"
    );
}

#[test]
fn real_field_extremes_are_finite_for_every_real_scalar() {
    assert_finite_extremes::<f32>(f32::MAX);
    assert_finite_extremes::<f64>(f64::MAX);
    assert_finite_extremes::<F32>(F32(f32::MAX));
    assert_finite_extremes::<F64>(F64(f64::MAX));
}

#[test]
fn reduction_identities_stay_infinite() {
    // The Min/Max fold seeds are a separate contract that consumers depend on.
    assert_eq!(<f32 as NumericElement>::MAX_VALUE, f32::INFINITY);
    assert_eq!(<f32 as NumericElement>::MIN_VALUE, f32::NEG_INFINITY);
    assert_eq!(<f64 as NumericElement>::MAX_VALUE, f64::INFINITY);
    assert_eq!(<f64 as NumericElement>::MIN_VALUE, f64::NEG_INFINITY);
}
