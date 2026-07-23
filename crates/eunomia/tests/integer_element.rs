//! Value-semantic contract tests for the integer `NumericElement` impls.
//!
//! Covers the signed (`i8`/`i16`/`i32`/`i64`) and unsigned (`u8`/`u16`/`u32`/
//! `u64`) implementations, cross-checking every trait operation against `std`
//! semantics rather than asserting mere existence.

use eunomia::{CastFrom, NumericElement};

/// Assert the full integer `NumericElement` contract for one type.
///
/// `$abs_in`/`$abs_out` parameterize the single sign-dependent case (`abs`):
/// signed invocations pass a negative input, unsigned a non-negative one.
macro_rules! integer_element_contract {
    ($name:ident, $t:ty, abs($abs_in:expr) == $abs_out:expr) => {
        #[test]
        fn $name() {
            // Identity / boundary constants match std.
            assert_eq!(<$t as NumericElement>::ZERO, 0 as $t);
            assert_eq!(<$t as NumericElement>::ONE, 1 as $t);
            assert_eq!(<$t as NumericElement>::MIN_VALUE, <$t>::MIN);
            assert_eq!(<$t as NumericElement>::MAX_VALUE, <$t>::MAX);
            assert_eq!(<$t as NumericElement>::ALL_ONES, !(0 as $t));
            assert_eq!(
                <$t as NumericElement>::BYTE_WIDTH,
                core::mem::size_of::<$t>()
            );

            // Identity arithmetic.
            let a: $t = 7;
            assert_eq!(a + <$t as NumericElement>::ZERO, a);
            assert_eq!(a * <$t as NumericElement>::ONE, a);

            // Bitwise ops and popcount match the std operators bit-for-bit.
            let x: $t = 0b1010;
            let y: $t = 0b0110;
            assert_eq!(NumericElement::bitand(x, y), x & y);
            assert_eq!(NumericElement::bitor(x, y), x | y);
            assert_eq!(NumericElement::bitxor(x, y), x ^ y);
            assert_eq!(NumericElement::count_ones(x), x.count_ones());

            // Ordering reductions return the correct operand.
            assert_eq!(NumericElement::min_scalar(a, 3 as $t), 3 as $t);
            assert_eq!(NumericElement::max_scalar(a, 3 as $t), 7 as $t);

            // Fused multiply-add follows the documented wrapping contract.
            let (b, c): ($t, $t) = (3, 4);
            assert_eq!(a.scalar_fmadd(b, c), a.wrapping_mul(b).wrapping_add(c));
            assert_eq!(a.scalar_fmadd(b, c), 25 as $t);

            // Integers are always finite, never NaN.
            assert!(NumericElement::is_finite(a));
            assert!(!NumericElement::is_nan(a));

            // Lossless widening to f64.
            assert_eq!(NumericElement::to_f64(a), 7.0_f64);

            // Integer (floor) square root is exact (no f64 round-trip). 16 fits
            // every width including i8/u8.
            assert_eq!(NumericElement::sqrt(16 as $t), 4 as $t);
            assert_eq!(NumericElement::sqrt(15 as $t), 3 as $t);
            assert_eq!(NumericElement::sqrt(1 as $t), 1 as $t);
            assert_eq!(
                NumericElement::sqrt(<$t as NumericElement>::ZERO),
                <$t as NumericElement>::ZERO
            );

            // Sign-dependent absolute value.
            assert_eq!(NumericElement::abs($abs_in as $t), $abs_out as $t);

            // CastFrom<i32> maps an in-range value exactly.
            assert_eq!(<$t as CastFrom<i32>>::cast_from(5_i32), 5 as $t);
        }
    };
}

integer_element_contract!(i8_element_contract, i8, abs(-5) == 5);
integer_element_contract!(i16_element_contract, i16, abs(-5) == 5);
integer_element_contract!(i32_element_contract, i32, abs(-5) == 5);
integer_element_contract!(i64_element_contract, i64, abs(-5) == 5);
integer_element_contract!(u8_element_contract, u8, abs(5) == 5);
integer_element_contract!(u16_element_contract, u16, abs(5) == 5);
integer_element_contract!(u32_element_contract, u32, abs(5) == 5);
integer_element_contract!(u64_element_contract, u64, abs(5) == 5);

/// Integer `sqrt` is exact for large operands where the former
/// `(self as f64).sqrt() as Self` path lost precision (the operand rounds to f64
/// before the root above 2^53). These are the regression cases.
#[test]
fn integer_sqrt_is_exact_for_large_operands() {
    // u64::MAX = 2^64−1; floor(sqrt) = 2^32−1 = 4_294_967_295. The f64 path
    // returned 2^32 = 4_294_967_296, whose square overflows u64 — provably wrong.
    assert_eq!(NumericElement::sqrt(u64::MAX), 4_294_967_295_u64);
    assert_eq!(
        u64::MAX.isqrt(),
        4_294_967_295_u64,
        "std isqrt is the oracle"
    );

    // i64::MAX = 2^63−1; floor(sqrt) = 3_037_000_499.
    assert_eq!(NumericElement::sqrt(i64::MAX), 3_037_000_499_i64);

    // Defining property r² ≤ n < (r+1)² on an operand above 2^53 (no overflow in
    // the check: r ≈ 2^20, (r+1)² ≪ u64::MAX).
    let n: u64 = (1u64 << 40) + 7;
    let r = NumericElement::sqrt(n);
    assert!(
        r * r <= n && (r + 1) * (r + 1) > n,
        "isqrt invariant for {n}"
    );
}

/// Negative signed inputs have no real root and no NaN; the documented contract
/// returns 0 (total, non-panicking — `isqrt` itself would panic on negatives).
#[test]
fn signed_integer_sqrt_of_negative_is_zero() {
    assert_eq!(NumericElement::sqrt(-4_i32), 0_i32);
    assert_eq!(NumericElement::sqrt(-1_i64), 0_i64);
    assert_eq!(NumericElement::sqrt(i64::MIN), 0_i64);
}

/// Cross-width `CastFrom` round-trips for an in-range value preserve it exactly.
#[test]
fn cross_width_cast_round_trip() {
    let v: u8 = 200;
    let widened = u32::cast_from(v);
    assert_eq!(widened, 200_u32);
    assert_eq!(u8::cast_from(widened), v);

    let s: i64 = -42;
    let narrowed = i32::cast_from(s);
    assert_eq!(narrowed, -42_i32);
    assert_eq!(i64::cast_from(narrowed), s);
}

/// Overflow regressions for every integer `NumericElement` impl (primitive
/// and wrapper) added by ATLAS-EUNOMIA-044. Before the patch, the trait's
/// float default `Some(self + rhs)` / `Some(self * rhs)` was inherited by
/// unsigned primitives and by I8/I16/I32 wrappers, which would silently wrap
/// on integer overflow in release builds (and panic under debug overflow
/// checks). Every affected type now matches `std`'s checked / saturating
/// semantics exactly:
///   * `checked_add`/`checked_mul` return `None` on overflow.
///   * `saturating_add`/`saturating_mul` cap at MIN_VALUE (signed underflow)
///     and at MAX_VALUE (any overflow).
///   * In-range operands pass through unchanged.
///   * For signed types, MIN.saturating_add(-1) = MIN (i.e. no panic); for
///     unsigned, MIN = 0 and -1 is not representable, so the underflow
///     assertion is gated behind `$is_signed`.
///
/// Parameter `$is_signed:ident` is `signed` or `unsigned`; the macro emits the
/// underflow assertion only when signed.
macro_rules! integer_overflow_contract {
    ($name:ident, $t:ty, $is_signed:ident) => {
        #[test]
        fn $name() {
            assert_eq!(
                NumericElement::checked_add(<$t>::MAX, 1 as $t),
                None,
                "checked_add must return None on MAX + 1 overflow"
            );
            assert_eq!(
                NumericElement::checked_mul(<$t>::MAX, 2 as $t),
                None,
                "checked_mul must return None on MAX * 2 overflow"
            );
            assert_eq!(
                NumericElement::saturating_add(<$t>::MAX, 1 as $t),
                <$t>::MAX,
                "saturating_add must cap at MAX"
            );
            assert_eq!(
                NumericElement::saturating_mul(<$t>::MAX, 2 as $t),
                <$t>::MAX,
                "saturating_mul must cap at MAX"
            );
            // In-range match std.
            assert_eq!(NumericElement::checked_add(3 as $t, 4 as $t), Some(7 as $t));
            assert_eq!(NumericElement::checked_mul(3 as $t, 4 as $t), Some(12 as $t));
            // Signed underflow only: unsigned MIN = 0 and -1 is not representable.
            integer_overflow_contract!(@underflow $t, $is_signed);
        }
    };
    // Signed underflow assertion: -1 is well-formed and the result must cap at MIN.
    (@underflow $t:ty, signed) => {
        assert_eq!(
            NumericElement::saturating_add(<$t>::MIN, -1 as $t),
            <$t>::MIN,
            "saturating_add must cap at MIN (signed underflow)"
        );
    };
    // Unsigned: -1 isn't a $t value, so the underflow check is vacuous.
    (@underflow $t:ty, unsigned) => {};
}

integer_overflow_contract!(u8_overflow_contract, u8, unsigned);
integer_overflow_contract!(u16_overflow_contract, u16, unsigned);
integer_overflow_contract!(u32_overflow_contract, u32, unsigned);
integer_overflow_contract!(u64_overflow_contract, u64, unsigned);
integer_overflow_contract!(usize_overflow_contract, usize, unsigned);
integer_overflow_contract!(i8_overflow_contract, i8, signed);
integer_overflow_contract!(i16_overflow_contract, i16, signed);
integer_overflow_contract!(i32_overflow_contract, i32, signed);
integer_overflow_contract!(i64_overflow_contract, i64, signed);
integer_overflow_contract!(isize_overflow_contract, isize, signed);

/// Wrapper I8/I16/I32 must mirror their primitive siblings by ATLAS-EUNOMIA-044:
/// the wrapper-level NumericElement must expose the same overflow semantics as
/// `std` (saturating to MIN/MAX, `None` on overflow), and integer-wrapper
/// `sqrt` must route through `isqrt` (not the legacy f32/f64 round-trip).
mod wrapper_integer_overflow {
    use eunomia::{NumericElement, I16, I32, I8};

    #[test]
    fn i8_wrapper_checked_overflow() {
        assert_eq!(
            <I8 as NumericElement>::checked_add(I8(i8::MAX), I8(1)),
            None,
            "I8::checked_add must return None on overflow"
        );
        assert_eq!(
            <I8 as NumericElement>::checked_mul(I8(i8::MAX), I8(2)),
            None
        );
        assert_eq!(
            <I8 as NumericElement>::saturating_add(I8(i8::MAX), I8(1)),
            I8(i8::MAX)
        );
        assert_eq!(
            <I8 as NumericElement>::saturating_mul(I8(i8::MAX), I8(2)),
            I8(i8::MAX)
        );
        assert_eq!(
            <I8 as NumericElement>::saturating_add(I8(i8::MIN), I8(-1)),
            I8(i8::MIN)
        );
    }

    #[test]
    fn i16_wrapper_checked_overflow() {
        assert_eq!(
            <I16 as NumericElement>::checked_add(I16(i16::MAX), I16(1)),
            None
        );
        assert_eq!(
            <I16 as NumericElement>::checked_mul(I16(i16::MAX), I16(2)),
            None
        );
        assert_eq!(
            <I16 as NumericElement>::saturating_add(I16(i16::MAX), I16(1)),
            I16(i16::MAX)
        );
        assert_eq!(
            <I16 as NumericElement>::saturating_mul(I16(i16::MAX), I16(2)),
            I16(i16::MAX)
        );
        assert_eq!(
            <I16 as NumericElement>::saturating_add(I16(i16::MIN), I16(-1)),
            I16(i16::MIN)
        );
    }

    #[test]
    fn i32_wrapper_checked_overflow() {
        assert_eq!(
            <I32 as NumericElement>::checked_add(I32(i32::MAX), I32(1)),
            None
        );
        assert_eq!(
            <I32 as NumericElement>::checked_mul(I32(i32::MAX), I32(2)),
            None
        );
        assert_eq!(
            <I32 as NumericElement>::saturating_add(I32(i32::MAX), I32(1)),
            I32(i32::MAX)
        );
        assert_eq!(
            <I32 as NumericElement>::saturating_mul(I32(i32::MAX), I32(2)),
            I32(i32::MAX)
        );
        assert_eq!(
            <I32 as NumericElement>::saturating_add(I32(i32::MIN), I32(-1)),
            I32(i32::MIN)
        );
    }

    #[test]
    fn i32_wrapper_sqrt_routes_through_isqrt() {
        // Small operands; pins parity with std `i32::isqrt`.
        assert_eq!(<I32 as NumericElement>::sqrt(I32(16)), I32(4));
        assert_eq!(<I32 as NumericElement>::sqrt(I32(15)), I32(3));
        assert_eq!(<I32 as NumericElement>::sqrt(I32(0)), I32(0));
        // Documented contract: signed negative inputs have no real root and
        // integers have no NaN sentinel; the call returns 0 rather than
        // panicking (which `i32::isqrt` itself would do).
        assert_eq!(<I32 as NumericElement>::sqrt(I32(-4)), I32(0));
        assert_eq!(<I32 as NumericElement>::sqrt(I32(i32::MIN)), I32(0));
        // Cross-check: every small in-range value matches std `i32::isqrt`
        // exactly (no f64 round-trip this time — that was the path the patch
        // removed; tying the oracle to `isqrt` proves the new path is correct).
        for n in 0..=100_000_i32 {
            let expected = n.isqrt();
            let got = <I32 as NumericElement>::sqrt(I32(n)).0;
            assert_eq!(got, expected, "I32 sqrt mismatch at n={n}");
        }
        // The regression case from the patch docstring. For i32 the f64
        // round-trip is exact, but we pin floor(sqrt(i32::MAX)) = 46340 as a
        // permanent operation-invariant (46340² = 2,147,395,600 ≤ i32::MAX;
        // 46341² = 2,147,488,281 > i32::MAX). The only way this stays exact
        // across future refactors is via the exact `i32::isqrt` primitive
        // rather than via any ((self as f64).sqrt() as i32) replacement.
        assert_eq!(<I32 as NumericElement>::sqrt(I32(i32::MAX)).0, 46340);
    }

    #[test]
    fn i8_i16_wrapper_sqrt_routes_through_isqrt() {
        // I8 / I16 also route through isqrt() for parity with I32 (and the
        // signed primitive). Negative inputs return 0.
        assert_eq!(<I8 as NumericElement>::sqrt(I8(9)), I8(3));
        assert_eq!(<I8 as NumericElement>::sqrt(I8(-1)), I8(0));
        assert_eq!(<I16 as NumericElement>::sqrt(I16(81)), I16(9));
        assert_eq!(<I16 as NumericElement>::sqrt(I16(-1)), I16(0));
    }
}
