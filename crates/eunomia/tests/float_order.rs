//! Ordering and equality contract for the bit-pattern float wrappers.
//!
//! `F16`/`Bf16`/`Bf8`/`Bf4`/`F8`/`F4` store a raw sign-magnitude code, so a
//! *derived* `PartialEq`/`PartialOrd` orders lexicographically on those bits and
//! is numerically wrong: the sign bit is the most significant bit of the code,
//! so every negative value sorts above every positive one, `-0` compares unequal
//! to `+0`, and NaN compares ordered.
//!
//! This is not cosmetic. [`NumericElement::min_scalar`]/[`max_scalar`] are
//! defaults over exactly this `PartialOrd` and are not overridden for these
//! types, so `MIN_VALUE`/`MAX_VALUE`-seeded Min/Max reductions, `clamp`, and
//! every sort taken over them anywhere in the Atlas stack inherit whatever the
//! comparison says. This file is the regression that keeps it IEEE 754.
//!
//! [`max_scalar`]: NumericElement::max_scalar

use core::cmp::Ordering;

use eunomia::{Bf16, Bf4, Bf8, NumericElement, F16, F4, F8};

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
/// `SIGN_MASK` bit set. Under a derived (bitwise) ordering every one of these
/// assertions inverts, because setting the sign bit raises the raw `u8`.
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
            // A Max reduction seeds its accumulator with MIN_VALUE; with a
            // bitwise ordering that seed (sign bit set) dominates every finite
            // operand and the reduction returns the identity element forever.
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

/// The 16-bit wrappers already compared float-semantically; their impls now come
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
