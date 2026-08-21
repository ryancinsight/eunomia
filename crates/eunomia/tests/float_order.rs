//! Ordering and equality contract for the bit-pattern float wrappers.
//!
//! `F16`/`Bf16`/`Bf8`/`Bf4`/`F8`/`F4` store a raw sign-magnitude code, so a
//! *derived* `PartialEq`/`PartialOrd` orders lexicographically on those bits and
//! is numerically wrong: the sign bit is the most significant bit of the code,
//! so every negative value sorts above every positive one, `-0` compares unequal
//! to `+0`, and NaN compares ordered.
//!
//! This is not cosmetic. `PartialOrd` remains unordered for NaN, while
//! [`NumericElement::min_scalar`]/[`max_scalar`] define the separate value
//! contract used by reductions and `clamp`: one NaN is ignored and signed-zero
//! ties select the IEEE minimum or maximum independent of operand order. This
//! file verifies both surfaces against independent encoding and value oracles.
//!
//! [`max_scalar`]: NumericElement::max_scalar

use core::cmp::Ordering;

use eunomia::{Bf16, Bf4, Bf8, NumericElement, RealField, F16, F32, F4, F64, F8};

/// Independent sign-magnitude ordering oracle.
///
/// Derived from the **format layout**, not from `to_f32()`: each type is
/// sign-magnitude with a biased exponent field above the mantissa, so the
/// magnitude bits compare monotonically with the absolute value. Mapping a code
/// to a signed key (`+magnitude` when the sign bit is clear, `-magnitude` when
/// set) therefore reproduces IEEE 754 ordering — including `-0 == +0`, since
/// both keys are `0` — without traversing the widening path the implementation
/// uses. Agreement is independent evidence rather than a restatement of the
/// implementation.
fn sign_magnitude_order(
    a: u8,
    b: u8,
    sign: u8,
    magnitude: u8,
    is_nan: impl Fn(u8) -> bool,
) -> Option<Ordering> {
    if is_nan(a) || is_nan(b) {
        // IEEE 754: any comparison involving NaN is unordered.
        return None;
    }
    let key = |code: u8| -> i32 {
        let mag = i32::from(code & magnitude);
        if code & sign == 0 {
            mag
        } else {
            -mag
        }
    };
    Some(key(a).cmp(&key(b)))
}

/// Exhaustive comparison conformance over a type's whole encoding domain.
///
/// `$domain` is the set of valid codes: the full byte for the 8-bit formats,
/// the low nibble for the 4-bit ones (whose upper nibble is not part of the
/// encoding). Every ordered pair is checked, so no bit pattern escapes.
macro_rules! exhaustive_order_conformance {
    ($name:ident, $t:ident, $domain:expr, $sign:expr, $magnitude:expr, $is_nan:expr) => {
        #[test]
        fn $name() {
            for a in $domain {
                for b in $domain {
                    let got = $t(a).partial_cmp(&$t(b));
                    assert_eq!(
                        got,
                        sign_magnitude_order(a, b, $sign, $magnitude, $is_nan),
                        concat!(stringify!($t), " order: {:#04X} vs {:#04X}"),
                        a,
                        b
                    );
                    // Widening to f32 is exact for these formats, so the widened
                    // comparison is the same relation by a second route.
                    assert_eq!(
                        got,
                        $t(a).to_f32().partial_cmp(&$t(b).to_f32()),
                        concat!(stringify!($t), " widened order: {:#04X} vs {:#04X}"),
                        a,
                        b
                    );
                    // Equality must agree with the ordering it is derived from.
                    assert_eq!(
                        $t(a) == $t(b),
                        got == Some(Ordering::Equal),
                        concat!(stringify!($t), " eq/cmp agreement: {:#04X} vs {:#04X}"),
                        a,
                        b
                    );
                }
            }
        }
    };
}

// Bf8 (E5M2, IEEE-style): exponent all-ones with a non-zero mantissa is NaN;
// a zero mantissa there is infinity, which is ordered.
exhaustive_order_conformance!(bf8_orders_as_ieee754, Bf8, 0u8..=u8::MAX, 0x80, 0x7F, |c| {
    c & 0x7C == 0x7C && c & 0x03 != 0
});
// F8 (E4M3, finite-only): the top exponent is reserved entirely for NaN.
exhaustive_order_conformance!(f8_orders_as_ieee754, F8, 0u8..=u8::MAX, 0x80, 0x7F, |c| {
    c & 0x78 == 0x78
});
// Bf4 (E2M1, finite-only): top exponent reserved for NaN; 4-bit domain.
exhaustive_order_conformance!(bf4_orders_as_ieee754, Bf4, 0u8..16, 0x08, 0x07, |c| {
    c & 0x06 == 0x06
});
// F4 (E3M0, finite-only): top exponent reserved for NaN; 4-bit domain.
exhaustive_order_conformance!(f4_orders_as_ieee754, F4, 0u8..16, 0x08, 0x07, |c| {
    c & 0x07 == 0x07
});

/// Sign inversion, stated on hand-derived encodings rather than on anything the
/// crate computes.
///
/// Each `-1.0` code is the crate's own `ONE` constant with the format's
/// `SIGN_MASK` bit set. The value-semantic comparison must keep the negative
/// value below the positive one even though setting the sign bit raises the
/// raw encoding.
macro_rules! negative_sorts_below_positive {
    ($name:ident, $t:ident) => {
        #[test]
        fn $name() {
            let one = <$t as NumericElement>::ONE;
            let zero = <$t as NumericElement>::ZERO;
            let minus_one = $t(one.0 | <$t as NumericElement>::SIGN_MASK.0);
            let minus_zero = $t(zero.0 | <$t as NumericElement>::SIGN_MASK.0);

            assert!(minus_one < one, "-1 < 1");
            assert!(minus_one < zero, "-1 < 0");
            assert!(zero < one, "0 < 1");
            // Signed zeros are distinct encodings but the same value.
            assert_eq!(minus_zero, zero, "-0 == +0");
            assert_ne!(minus_zero.0, zero.0, "-0 and +0 keep distinct encodings");

            // The reductions that consume this ordering.
            assert_eq!(
                NumericElement::min_scalar(minus_one, one).0,
                minus_one.0,
                "min(-1, 1) == -1"
            );
            assert_eq!(
                NumericElement::max_scalar(minus_one, one).0,
                one.0,
                "max(-1, 1) == 1"
            );
            // A Max reduction seeds its accumulator with MIN_VALUE; the
            // value-semantic ordering must replace that identity with the first
            // finite operand.
            assert_eq!(
                NumericElement::max_scalar(<$t as NumericElement>::MIN_VALUE, one).0,
                one.0,
                "max(MIN_VALUE, 1) == 1"
            );
            assert_eq!(
                NumericElement::min_scalar(<$t as NumericElement>::MAX_VALUE, minus_one).0,
                minus_one.0,
                "min(MAX_VALUE, -1) == -1"
            );
        }
    };
}

negative_sorts_below_positive!(bf8_negatives_sort_below_positives, Bf8);
negative_sorts_below_positive!(f8_negatives_sort_below_positives, F8);
negative_sorts_below_positive!(bf4_negatives_sort_below_positives, Bf4);
negative_sorts_below_positive!(f4_negatives_sort_below_positives, F4);
negative_sorts_below_positive!(f16_negatives_sort_below_positives, F16);
negative_sorts_below_positive!(bf16_negatives_sort_below_positives, Bf16);

/// The 16-bit wrappers already compare float-semantically; their impls now come
/// from the shared `float_semantic_cmp!` expansion, so this pins that the
/// consolidation preserved the behaviour. `2^32` ordered pairs is out of budget,
/// so every one of the `65_536` codes is checked against a representative set
/// spanning both signs, both zeros, the infinities, and a NaN.
macro_rules! widened_order_conformance_16 {
    ($name:ident, $t:ident) => {
        #[test]
        fn $name() {
            let probes = [
                <$t as NumericElement>::ZERO,
                $t(<$t as NumericElement>::ZERO.0 | <$t as NumericElement>::SIGN_MASK.0),
                <$t as NumericElement>::ONE,
                $t(<$t as NumericElement>::ONE.0 | <$t as NumericElement>::SIGN_MASK.0),
                <$t as NumericElement>::INFINITY,
                <$t as NumericElement>::MIN_VALUE,
                <$t as NumericElement>::NAN,
            ];
            for bits in 0u16..=u16::MAX {
                let a = $t(bits);
                for &b in &probes {
                    assert_eq!(
                        a.partial_cmp(&b),
                        a.to_f32().partial_cmp(&b.to_f32()),
                        concat!(stringify!($t), " order: {:#06X} vs {:#06X}"),
                        bits,
                        b.0
                    );
                    assert_eq!(
                        a == b,
                        a.to_f32() == b.to_f32(),
                        concat!(stringify!($t), " eq: {:#06X} vs {:#06X}"),
                        bits,
                        b.0
                    );
                }
            }
        }
    };
}

widened_order_conformance_16!(f16_orders_as_ieee754, F16);
widened_order_conformance_16!(bf16_orders_as_ieee754, Bf16);

/// NaN is unordered and non-reflexive for every one of these types — the
/// property that makes `Eq`/`Ord`/`Hash` unimplementable for them, and that a
/// derived `PartialOrd` would violate.
#[test]
fn nan_is_unordered_and_non_reflexive() {
    macro_rules! assert_nan_unordered {
        ($($t:ident),+) => {$(
            let nan = <$t as NumericElement>::NAN;
            let one = <$t as NumericElement>::ONE;
            assert_ne!(nan, nan, concat!(stringify!($t), ": NaN != NaN"));
            assert_eq!(
                nan.partial_cmp(&one),
                None,
                concat!(stringify!($t), ": NaN vs 1 is unordered")
            );
            assert_eq!(
                nan.partial_cmp(&nan),
                None,
                concat!(stringify!($t), ": NaN vs NaN is unordered")
            );
            // Every comparison operator, not just `partial_cmp`: all five must
            // be false, which is what "unordered" means at the call site.
            assert_eq!(
                [nan < one, nan > one, nan <= one, nan >= one, nan == one],
                [false; 5],
                concat!(stringify!($t), ": every relation against NaN is false")
            );
        )+};
    }
    assert_nan_unordered!(F16, Bf16, Bf8, Bf4, F8, F4);
}

/// Assert the shared min/max special-value contract for one real scalar type.
fn assert_real_min_max_contract<T: NumericElement>() {
    let nan = T::NAN;
    let one = T::ONE;
    let positive_zero = T::ZERO;
    let negative_zero = <T as NumericElement>::bitor(T::ZERO, T::SIGN_MASK);

    assert_eq!(
        <T as NumericElement>::min_scalar(nan, one),
        one,
        "min(NaN, 1) ignores NaN"
    );
    assert_eq!(
        <T as NumericElement>::min_scalar(one, nan),
        one,
        "min(1, NaN) ignores NaN"
    );
    assert_eq!(
        <T as NumericElement>::max_scalar(nan, one),
        one,
        "max(NaN, 1) ignores NaN"
    );
    assert_eq!(
        <T as NumericElement>::max_scalar(one, nan),
        one,
        "max(1, NaN) ignores NaN"
    );
    assert!(
        <T as NumericElement>::min_scalar(nan, nan).is_nan(),
        "min(NaN, NaN) remains NaN"
    );
    assert!(
        <T as NumericElement>::max_scalar(nan, nan).is_nan(),
        "max(NaN, NaN) remains NaN"
    );

    let min_zero = <T as NumericElement>::min_scalar(positive_zero, negative_zero);
    let min_zero_reversed = <T as NumericElement>::min_scalar(negative_zero, positive_zero);
    assert_eq!(min_zero, negative_zero, "min(+0, -0) returns exactly -0");
    assert_eq!(
        min_zero_reversed, negative_zero,
        "min(-0, +0) returns exactly -0"
    );
    assert_eq!(
        min_zero.bitand(T::SIGN_MASK),
        T::SIGN_MASK,
        "min(+0, -0) returns -0"
    );
    assert_eq!(
        min_zero_reversed.bitand(T::SIGN_MASK),
        T::SIGN_MASK,
        "min(-0, +0) returns -0"
    );

    let max_zero = <T as NumericElement>::max_scalar(positive_zero, negative_zero);
    let max_zero_reversed = <T as NumericElement>::max_scalar(negative_zero, positive_zero);
    assert_eq!(max_zero, positive_zero, "max(+0, -0) returns exactly +0");
    assert_eq!(
        max_zero_reversed, positive_zero,
        "max(-0, +0) returns exactly +0"
    );
    assert_eq!(
        max_zero.bitand(T::SIGN_MASK),
        T::ZERO,
        "max(+0, -0) returns +0"
    );
    assert_eq!(
        max_zero_reversed.bitand(T::SIGN_MASK),
        T::ZERO,
        "max(-0, +0) returns +0"
    );

    let clamp = |value: T, min: T, max: T| {
        <T as NumericElement>::min_scalar(<T as NumericElement>::max_scalar(value, min), max)
    };
    assert_eq!(clamp(nan, T::ZERO, one), T::ZERO, "clamp(NaN, 0, 1) = 0");
    assert_eq!(clamp(one, nan, T::ONE), one, "clamp(1, NaN, 1) = 1");
    assert_eq!(
        clamp(T::ZERO, T::ZERO, nan),
        T::ZERO,
        "clamp(0, 0, NaN) = 0"
    );
}

#[test]
fn real_scalars_share_nan_and_signed_zero_min_max() {
    assert_real_min_max_contract::<f32>();
    assert_real_min_max_contract::<f64>();
    assert_real_min_max_contract::<F16>();
    assert_real_min_max_contract::<F32>();
    assert_real_min_max_contract::<F64>();
    assert_real_min_max_contract::<Bf16>();
    assert_real_min_max_contract::<Bf8>();
    assert_real_min_max_contract::<Bf4>();
    assert_real_min_max_contract::<F8>();
    assert_real_min_max_contract::<F4>();
}

#[test]
fn real_field_clamp_uses_the_scalar_special_value_contract() {
    assert_eq!(<f32 as RealField>::clamp(f32::NAN, 0.0, 1.0), 0.0);
    assert_eq!(<f64 as RealField>::clamp(1.0, f64::NAN, 1.0), 1.0);
    assert_eq!(<f64 as RealField>::clamp(0.0, 0.0, f64::NAN), 0.0);
}
