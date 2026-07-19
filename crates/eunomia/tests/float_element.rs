//! Value-semantic contract tests for the native float `NumericElement` /
//! `FloatElement` impls (`F16`, `Bf16`, `F32`, `F64`).
//!
//! The integer surface has `integer_element.rs`; this is its float analogue,
//! cross-checking every trait operation against analytic values rather than
//! asserting mere existence. The reduced-precision types `F16`/`Bf16` are the
//! ones the hermes SIMD kernels bind as `SimdKernel<F16>`/`<Bf16>`, so their
//! scalar contract is load-bearing across the Atlas stack — this file is the
//! regression that keeps them first-class scalars, not just storage wrappers.
//!
//! Every operand is exactly representable in `bfloat16` (≤3 significant bits),
//! hence in every wider type here, so each result is exact and no assertion
//! carries a tolerance. Calls are fully-qualified `NumericElement::…` /
//! `FloatElement::…`: std is stabilizing same-named inherent float methods
//! (`unstable_name_collisions`), and method syntax would silently rebind to
//! those, changing which implementation the test verifies.

use eunomia::{Bf16, CastFrom, FloatElement, NumericElement, F16, F32, F64};

/// Assert the full float `NumericElement` + `FloatElement` contract for one
/// native float wrapper type, entirely through exactly-representable operands.
macro_rules! float_element_contract {
    ($name:ident, $t:ty) => {
        #[test]
        fn $name() {
            // Construct from / observe as `f32` through the trait under test.
            let f = |v: f32| <$t as FloatElement>::from_f32(v);
            let g = |x: $t| FloatElement::to_f32(x);

            // ── Constants match their IEEE meaning ──
            assert_eq!(g(<$t as NumericElement>::ZERO), 0.0, "ZERO");
            assert_eq!(g(<$t as NumericElement>::ONE), 1.0, "ONE");
            assert_eq!(
                g(<$t as NumericElement>::INFINITY),
                f32::INFINITY,
                "INFINITY"
            );
            assert_eq!(
                <$t as NumericElement>::BYTE_WIDTH,
                core::mem::size_of::<$t>(),
                "BYTE_WIDTH == size_of"
            );

            // ── Classification: NaN vs infinity vs a normal value ──
            assert!(
                NumericElement::is_nan(<$t as NumericElement>::NAN),
                "is_nan(NAN)"
            );
            assert!(
                !NumericElement::is_finite(<$t as NumericElement>::NAN),
                "!is_finite(NAN)"
            );
            assert!(
                !NumericElement::is_finite(<$t as NumericElement>::INFINITY),
                "!is_finite(INFINITY)"
            );
            assert!(
                NumericElement::is_finite(<$t as NumericElement>::ONE),
                "is_finite(ONE)"
            );
            assert!(
                !NumericElement::is_nan(<$t as NumericElement>::ONE),
                "!is_nan(ONE)"
            );

            // ── Additive/multiplicative identities via the type's own ops ──
            let a = f(1.5);
            assert_eq!(a + <$t as NumericElement>::ZERO, a, "a + 0 == a");
            assert_eq!(a * <$t as NumericElement>::ONE, a, "a * 1 == a");

            // ── Exact arithmetic (operands + results representable in bf16) ──
            assert_eq!(g(f(1.5) + f(2.5)), 4.0, "1.5 + 2.5");
            assert_eq!(g(f(4.0) - f(2.5)), 1.5, "4.0 - 2.5");
            assert_eq!(g(f(2.0) * f(3.0)), 6.0, "2.0 * 3.0");
            assert_eq!(g(f(6.0) / f(2.0)), 3.0, "6.0 / 2.0");

            // ── abs / sqrt / signum ──
            assert_eq!(g(NumericElement::abs(f(-2.5))), 2.5, "abs(-2.5)");
            assert_eq!(g(NumericElement::sqrt(f(4.0))), 2.0, "sqrt(4)");
            assert_eq!(g(FloatElement::signum(f(-2.5))), -1.0, "signum(-2.5)");
            assert_eq!(g(FloatElement::signum(f(2.5))), 1.0, "signum(2.5)");

            // ── Fused multiply-add: 2*3 + 1 = 7 (exact) ──
            assert_eq!(
                g(NumericElement::scalar_fmadd(f(2.0), f(3.0), f(1.0))),
                7.0,
                "fmadd 2*3+1"
            );

            // ── Ordering reductions return the correct operand ──
            assert_eq!(g(NumericElement::min_scalar(f(1.5), f(2.5))), 1.5, "min");
            assert_eq!(g(NumericElement::max_scalar(f(1.5), f(2.5))), 2.5, "max");

            // ── FloatElement round-trip + integer power (exp-by-squaring) ──
            assert_eq!(g(<$t as FloatElement>::from_f64(1.5_f64)), 1.5, "from_f64");
            assert_eq!(g(FloatElement::powi(f(2.0), 3)), 8.0, "2^3");
            assert_eq!(g(FloatElement::powi(f(2.0), 0)), 1.0, "2^0");
            assert_eq!(g(FloatElement::powi(f(2.0), -1)), 0.5, "2^-1");

            // ── Transcendental defaults at exact points (0/1 land on values
            //    every precision here represents exactly) ──
            assert_eq!(
                g(FloatElement::exp(<$t as NumericElement>::ZERO)),
                1.0,
                "exp(0)"
            );
            assert_eq!(
                g(FloatElement::ln(<$t as NumericElement>::ONE)),
                0.0,
                "ln(1)"
            );
            assert_eq!(
                g(FloatElement::sin(<$t as NumericElement>::ZERO)),
                0.0,
                "sin(0)"
            );
            assert_eq!(
                g(FloatElement::cos(<$t as NumericElement>::ZERO)),
                1.0,
                "cos(0)"
            );

            // ── CastFrom<i32> maps an in-range integer to its float value ──
            assert_eq!(
                g(<$t as CastFrom<i32>>::cast_from(5_i32)),
                5.0,
                "cast_from(5)"
            );
        }
    };
}

float_element_contract!(f16_element_contract, F16);
float_element_contract!(bf16_element_contract, Bf16);
float_element_contract!(f32_wrapper_element_contract, F32);
float_element_contract!(f64_wrapper_element_contract, F64);

/// The reduced-precision types must also *round* through their native kernel,
/// not merely construct: a value between two representable grid points quantizes
/// to the nearest, ties to even. `0.1` is unrepresentable in every finite binary
/// float, so each type snaps it to its nearest grid point — and the finer
/// `binary16` grid (10 mantissa bits) lands strictly nearer than `bfloat16`'s
/// (7 bits), which is exactly why these are distinct types, not one alias.
#[test]
fn reduced_precision_rounds_to_the_native_grid() {
    let f16 = FloatElement::to_f32(<F16 as FloatElement>::from_f32(0.1));
    let bf16 = FloatElement::to_f32(<Bf16 as FloatElement>::from_f32(0.1));

    // Round-to-nearest relative error is bounded by 2^-mantissa (a loose but
    // valid envelope over the tight 0.5-ulp bound), so each snapped value stays
    // within that fraction of 0.1.
    assert!(
        (f16 - 0.1).abs() <= 0.1 * 2.0_f32.powi(-10),
        "F16 |err| ≤ rel 2^-10"
    );
    assert!(
        (bf16 - 0.1).abs() <= 0.1 * 2.0_f32.powi(-7),
        "Bf16 |err| ≤ rel 2^-7"
    );
    // The finer grid is strictly nearer, and the two disagree — not interchangeable.
    assert!(
        (f16 - 0.1).abs() < (bf16 - 0.1).abs(),
        "F16 grid finer than Bf16 at 0.1"
    );
    assert_ne!(f16, bf16, "F16 and Bf16 quantize 0.1 to distinct values");
}
