//! `FloatElement` impls for the wrapper float types (native f64 via the shared
//! native-`f64` macro; reduced-precision types route through f32, narrowing
//! from `f64` once via round-to-odd).

use crate::convert::odd_rounded;
use crate::impls::native_f64::impl_float_element_native_f64;
use crate::traits::FloatElement;
use crate::types::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8};

macro_rules! impl_float_element {
    ($t:ident, $acc:ty, $from_f32:expr, $from_f64:expr, $to_f32:expr) => {
        impl_float_element!($t, $acc, $from_f32, $from_f64, $to_f32, {});
    };
    ($t:ident, $acc:ty, $from_f32:expr, $from_f64:expr, $to_f32:expr, {$($count:item)*}) => {
        impl FloatElement for $t {
            type Accumulator = $acc;
            $($count)*

            #[inline(always)]
            fn from_f32(val: f32) -> Self {
                $from_f32(val)
            }
            #[inline(always)]
            fn from_f64(val: f64) -> Self {
                $from_f64(val)
            }
            #[inline(always)]
            fn to_f32(self) -> f32 {
                $to_f32(self)
            }
        }
    };
}

// Accumulator column: reduced-precision formats widen to `f32` (exact — each
// has a ≤11-bit significand — and it lifts them off their stagnation point,
// `n ≈ 1/ε`); the `f32`/`f64` wrappers accumulate in themselves. The rationale
// is stated once on `FloatElement::Accumulator`.
impl_float_element!(F16, f32, F16::from_f32, F16::from_f64, F16::to_f32);
// The provided count conversions round to odd for formats narrower than
// `f32`; `F32` converts directly.
impl_float_element!(F32, F32, F32, |val| F32(val as f32), |x: F32| x.0, {
    #[inline]
    fn from_count(n: usize) -> Self {
        F32::from_count(n)
    }
    #[inline]
    fn from_integer(k: i64) -> Self {
        F32::from_integer(k)
    }
    #[inline]
    fn from_count_reciprocal(n: usize) -> Self {
        F32::from_count_reciprocal(n)
    }
});
// F64 wraps native `f64`, so it is emitted from the shared native-`f64` macro
// (the same table the primitive impl uses) — the `impl_float_element!` default
// would widen-narrow it and discard f64 precision. (F32 routes through f32 =
// native; F16/Bf16/F8/F4/Bf8/Bf4 have no hardware transcendentals, so the f32
// default is their correct reduced-precision path.)
impl_float_element_native_f64!(F64, |x: F64| x.0, |v: f64| F64(v));
impl_float_element!(Bf16, f32, Bf16::from_f32, Bf16::from_f64, Bf16::to_f32);
impl_float_element!(
    Bf8,
    f32,
    Bf8::from_f32,
    |val| Bf8::from_f32(odd_rounded(val)),
    |x: Bf8| x.to_f32()
);
impl_float_element!(
    Bf4,
    f32,
    Bf4::from_f32,
    |val| Bf4::from_f32(odd_rounded(val)),
    |x: Bf4| x.to_f32()
);
impl_float_element!(
    F8,
    f32,
    F8::from_f32,
    |val| F8::from_f32(odd_rounded(val)),
    |x: F8| x.to_f32()
);
impl_float_element!(
    F4,
    f32,
    F4::from_f32,
    |val| F4::from_f32(odd_rounded(val)),
    |x: F4| x.to_f32()
);

#[cfg(test)]
mod tests {
    use crate::traits::{FloatElement, NumericElement};
    use crate::types::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8};

    fn normalized_round_trip<T: FloatElement>(value: T, expected_exponent: i32) {
        assert_eq!(value.binary_exponent(), Some(expected_exponent));
        let normalized = value.scale_binary(-expected_exponent);
        let magnitude = <T as NumericElement>::abs(normalized).to_f64();
        assert!((1.0..2.0).contains(&magnitude));
        assert_eq!(normalized.scale_binary(expected_exponent), value);
    }

    fn signed_zero_and_nan<T: FloatElement>(negative_zero: T) {
        assert_eq!(negative_zero.binary_exponent(), None);
        let scaled_zero = negative_zero.scale_binary(i32::MAX);
        assert_eq!(
            scaled_zero.to_f64().to_bits(),
            negative_zero.to_f64().to_bits()
        );

        let nan = T::from_f32(f32::NAN);
        assert_eq!(nan.binary_exponent(), None);
        assert!(nan.scale_binary(0).is_nan());

        let infinity = <T as NumericElement>::INFINITY;
        if !infinity.is_finite() {
            assert_eq!(infinity.binary_exponent(), None);
        }
    }

    #[test]
    fn f64_wrapper_transcendentals_are_native_precision() {
        // Native f64 agrees with std f64 to ~machine epsilon; the f32-routed
        // default would only be accurate to ~1e-7, failing these bounds.
        assert!((F64(1.0).exp().0 - core::f64::consts::E).abs() < 1e-15);
        assert!((F64(0.1).ln().0 - 0.1_f64.ln()).abs() < 1e-15);
        assert!((F64(0.7).sin().0 - 0.7_f64.sin()).abs() < 1e-15);
        assert!((F64(0.25).acos().0 - 0.25_f64.acos()).abs() < 1e-15);
        assert!((F64(2.0).powf(F64(10.0)).0 - 1024.0).abs() < 1e-12);
        assert!((F64(-8.0).cbrt().0 + 2.0).abs() < 1e-15);
        assert!((F64(27.0).cbrt().0 - 3.0).abs() < 1e-15);
        // nth_root composes pow(|x|, 1/n) + copysign, so it is ~1 ulp rather
        // than the exact-perfect-cube cbrt; bounds are looser accordingly.
        assert!((F64(27.0).nth_root(3).0 - 3.0).abs() < 1e-12);
        assert!((F64(-27.0).nth_root(3).0 + 3.0).abs() < 1e-12);
        assert!((F64(64.0).nth_root(6).0 - 2.0).abs() < 1e-12);
    }

    #[test]
    fn rounding_and_powi_surface() {
        // UFCS calls the FloatElement impl explicitly (concrete f64 would
        // otherwise resolve to std's inherent methods, not eunomia's).
        assert_eq!(FloatElement::floor(2.7_f64), 2.0);
        assert_eq!(FloatElement::ceil(2.2_f64), 3.0);
        assert_eq!(FloatElement::round(2.5_f64), 3.0);
        assert_eq!(FloatElement::trunc(-2.7_f64), -2.0);
        assert!((FloatElement::acos(0.25_f64) - 0.25_f64.acos()).abs() < 1e-15);
        // signum: ±1 with the sign (num_traits / std semantics), NaN→NaN.
        assert_eq!(FloatElement::signum(-3.5_f64), -1.0);
        assert_eq!(FloatElement::signum(0.0_f64), 1.0);
        // powi: exact integer power, correct for negative base + exponent.
        assert_eq!(FloatElement::powi(2.0_f64, 10), 1024.0);
        assert_eq!(FloatElement::powi(-2.0_f64, 3), -8.0);
        assert!((FloatElement::powi(2.0_f64, -2) - 0.25).abs() < 1e-15);
        // F64 wrapper (native impl).
        assert_eq!(FloatElement::floor(F64(2.7)).0, 2.0);
        assert_eq!(FloatElement::powi(F64(2.0), 10).0, 1024.0);
    }

    #[test]
    fn round_ties_even_rounds_to_nearest_ties_to_even() {
        // Ties resolve to the nearest EVEN integer, unlike `round` (half away
        // from zero): 2.5 and 3.5 are equidistant from their neighbors, and
        // -0.5 is equidistant from 0 and -1.
        assert_eq!(FloatElement::round_ties_even(2.5_f64), 2.0);
        assert_eq!(FloatElement::round_ties_even(3.5_f64), 4.0);
        assert_eq!(
            FloatElement::round_ties_even(-0.5_f64).to_bits(),
            (-0.0_f64).to_bits(),
            "-0.5 ties to -0, not 0 or -1"
        );
        // Non-tie values round to the nearest integer exactly as `round` does.
        assert_eq!(FloatElement::round_ties_even(2.3_f64), 2.0);
        assert_eq!(FloatElement::round_ties_even(2.7_f64), 3.0);
        assert_eq!(FloatElement::round_ties_even(-2.5_f64), -2.0);
        // Special values pass through unchanged.
        assert!(FloatElement::round_ties_even(f64::NAN).is_nan());
        assert_eq!(FloatElement::round_ties_even(f64::INFINITY), f64::INFINITY);
        assert_eq!(
            FloatElement::round_ties_even(f64::NEG_INFINITY),
            f64::NEG_INFINITY
        );
        // Cross-check against std's own `f64::round_ties_even` at every case.
        for &x in &[2.5, 3.5, -0.5, -2.5, 0.5, 4.5, 2.3, 2.7] {
            assert_eq!(
                FloatElement::round_ties_even(x),
                x.round_ties_even(),
                "round_ties_even({x}) vs std"
            );
        }
        // f32 default (routes through f32 libm `roundevenf`) and F64 wrapper
        // (native `libm::roundeven`) share the same contract.
        assert_eq!(FloatElement::round_ties_even(2.5_f32), 2.0);
        assert_eq!(FloatElement::round_ties_even(3.5_f32), 4.0);
        assert_eq!(FloatElement::round_ties_even(F64(2.5)).0, 2.0);
        assert_eq!(FloatElement::round_ties_even(F64(3.5)).0, 4.0);
        assert_eq!(
            FloatElement::round_ties_even(F64(-0.5)).0.to_bits(),
            (-0.0_f64).to_bits()
        );
    }

    #[test]
    fn binary_scaling_normalizes_every_float_implementation() {
        normalized_round_trip(2.0_f32, 1);
        normalized_round_trip(-2.0_f32, 1);
        normalized_round_trip(2.0_f64, 1);
        normalized_round_trip(-2.0_f64, 1);
        normalized_round_trip(F16::from_f32(2.0), 1);
        normalized_round_trip(F16::from_f32(-2.0), 1);
        normalized_round_trip(F32(2.0), 1);
        normalized_round_trip(F32(-2.0), 1);
        normalized_round_trip(F64(2.0), 1);
        normalized_round_trip(F64(-2.0), 1);
        normalized_round_trip(Bf16::from_f32(2.0), 1);
        normalized_round_trip(Bf16::from_f32(-2.0), 1);
        normalized_round_trip(Bf8::from_f32(2.0), 1);
        normalized_round_trip(Bf8::from_f32(-2.0), 1);
        normalized_round_trip(Bf4::from_f32(2.0), 1);
        normalized_round_trip(Bf4::from_f32(-2.0), 1);
        normalized_round_trip(F8::from_f32(2.0), 1);
        normalized_round_trip(F8::from_f32(-2.0), 1);
        normalized_round_trip(F4::from_f32(2.0), 1);
        normalized_round_trip(F4::from_f32(-2.0), 1);

        // E2M1 represents 1.5 exactly; its normalized significand must not be
        // forced into the unrepresentable [0.5, 1) range.
        normalized_round_trip(Bf4::from_f32(1.5), 0);
    }

    #[test]
    fn binary_scaling_preserves_subnormal_exponents() {
        normalized_round_trip(f32::from_bits(1), -149);
        normalized_round_trip(f64::from_bits(1), -1074);
        normalized_round_trip(F32(f32::from_bits(1)), -149);
        normalized_round_trip(F64(f64::from_bits(1)), -1074);
        normalized_round_trip(F16::from_bits(1), -24);
        normalized_round_trip(Bf16::from_bits(1), -133);
        normalized_round_trip(Bf8(1), -16);
        normalized_round_trip(Bf4(1), -1);
        normalized_round_trip(F8(1), -9);
        normalized_round_trip(F4(1), -2);
    }

    #[test]
    fn binary_scaling_keeps_native_float_boundaries_and_special_values() {
        assert_eq!(f32::MAX.binary_exponent(), Some(127));
        normalized_round_trip(f32::MAX, 127);
        let one_f32 = <f32 as NumericElement>::ONE;
        assert_eq!(one_f32.scale_binary(128), f32::INFINITY);
        assert_eq!(one_f32.scale_binary(-149).to_bits(), 1);
        assert_eq!(one_f32.scale_binary(-150).to_bits(), 0);

        assert_eq!(f64::MAX.binary_exponent(), Some(1023));
        normalized_round_trip(f64::MAX, 1023);
        let one_f64 = <f64 as NumericElement>::ONE;
        assert_eq!(one_f64.scale_binary(1024), f64::INFINITY);
        assert_eq!(one_f64.scale_binary(-1074).to_bits(), 1);
        assert_eq!(one_f64.scale_binary(-1075).to_bits(), 0);

        signed_zero_and_nan(-0.0_f32);
        signed_zero_and_nan(-0.0_f64);
        signed_zero_and_nan(F16::from_f32(-0.0));
        signed_zero_and_nan(F32(-0.0));
        signed_zero_and_nan(F64(-0.0));
        signed_zero_and_nan(Bf16::from_f32(-0.0));
        signed_zero_and_nan(Bf8::from_f32(-0.0));
        signed_zero_and_nan(Bf4::from_f32(-0.0));
        signed_zero_and_nan(F8::from_f32(-0.0));
        signed_zero_and_nan(F4::from_f32(-0.0));
    }
}
