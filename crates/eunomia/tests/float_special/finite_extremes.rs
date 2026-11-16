//! `RealField::{min_value, max_value}` are the extreme *finite* values.
//!
//! Distinct from the reduction identities `NumericElement::{MIN_VALUE,
//! MAX_VALUE}`, which seed Min/Max folds so that an infinite operand still
//! reduces to itself: `-inf`/`+inf` where the format encodes infinity, and the
//! saturated extreme finite value for the finite-only `Bf4`, `F8` and `F4`.
//!
//! The finite-extreme oracle is each format's own largest finite magnitude,
//! `(1 - 2^-p) * 2^(emax + 1)` with `p` the precision including the implicit
//! bit (`MANTISSA_DIGITS`) and `emax + 1` the std `MAX_EXP`, as documented on
//! `f32::MAX` in std 1.97.0 (`core/src/num/f32.rs`). That equals
//! `(2 - 2^(1-p)) * 2^emax`; both `f32` (`p = 24`, `emax = 127`) and `f64`
//! (`p = 53`, `emax = 1023`) match it exactly under rational arithmetic, and
//! std exposes the value as `MAX`. The finite-only formats' saturated values
//! (3, 240, 8) are the `to_f64` of `from_f64(inf)` pinned in the crate's
//! integer-conversion tests and their codes in `subbyte_convert.rs`. Every
//! comparison is exact, so no tolerance.

use core::fmt::Debug;

use eunomia::{Bf16, Bf4, Bf8, FloatElement, NumericElement, RealField, F16, F32, F4, F64, F8};

/// Asserts the finite-extreme contract for one scalar against the format's
/// largest finite value `largest` (which equals `-smallest`).
fn assert_finite_extremes<T: RealField + Debug>(largest: T) {
    let max = <T as RealField>::max_value();
    let min = <T as RealField>::min_value();
    assert_eq!(max, largest, "max_value is the largest finite value");
    assert_eq!(min, -largest, "min_value is the smallest finite value");
    assert!(NumericElement::is_finite(max), "max_value is finite");
    assert!(NumericElement::is_finite(min), "min_value is finite");
    assert!(max < <T as RealField>::infinity(), "max_value < +inf");
    assert!(min > <T as RealField>::neg_infinity(), "min_value > -inf");
    // Twice the largest finite value exceeds the overflow threshold, so it
    // rounds to infinity, which is not finite.
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

/// Asserts the reduction-identity contract for one float format: `MAX_VALUE`
/// is the format's `INFINITY` and `MIN_VALUE` is its negation. `identity` is
/// `+inf` for a format that encodes infinity and the saturated largest finite
/// value for one that does not.
fn assert_reduction_identities<T: FloatElement>(identity: f64) {
    assert_eq!(
        NumericElement::to_f64(<T as NumericElement>::MAX_VALUE),
        identity,
        "MAX_VALUE is the Min-reduction identity"
    );
    assert_eq!(
        NumericElement::to_f64(<T as NumericElement>::MIN_VALUE),
        -identity,
        "MIN_VALUE is the Max-reduction identity"
    );
    assert_eq!(
        NumericElement::to_f64(<T as NumericElement>::INFINITY),
        identity,
        "INFINITY equals MAX_VALUE"
    );
}

#[test]
fn reduction_identities_are_the_format_infinity_for_every_float() {
    // Formats that encode infinity: the fold seeds are +/-inf.
    assert_reduction_identities::<f32>(f64::INFINITY);
    assert_reduction_identities::<f64>(f64::INFINITY);
    assert_reduction_identities::<F32>(f64::INFINITY);
    assert_reduction_identities::<F64>(f64::INFINITY);
    assert_reduction_identities::<F16>(f64::INFINITY);
    assert_reduction_identities::<Bf16>(f64::INFINITY);
    assert_reduction_identities::<Bf8>(f64::INFINITY);
    // Finite-only formats: the seeds are the saturated largest finite value.
    assert_reduction_identities::<Bf4>(3.0);
    assert_reduction_identities::<F8>(240.0);
    assert_reduction_identities::<F4>(8.0);
}
