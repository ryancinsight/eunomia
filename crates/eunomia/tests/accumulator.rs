//! Contract for [`FloatElement::Accumulator`] — the crate's only sanctioned
//! widening route.
//!
//! The associated type is only worth having if accumulating through it is
//! measurably better than accumulating in the element type. That is asserted
//! here against a derived error bound rather than a tuned epsilon: the
//! accumulator path must land inside the bound and the in-`T` path must land
//! outside it, so the test fails if either the bound or the widening is wrong.

use eunomia::{Bf16, Bf4, Bf8, FloatElement, NumericElement, F16, F32, F4, F64, F8};

/// Pairwise (binary-tree) summation.
///
/// The recursion order is the one the error bound in
/// [`bf16_accumulator_beats_in_type_summation`] is derived for, so it is fixed
/// here rather than left to the optimizer: sequential and pairwise accumulation
/// differ by `n` versus `log₂ n` in the bound, and a bound quoted without its
/// order is meaningless.
fn pairwise_sum<T: FloatElement>(values: &[T]) -> T {
    match values {
        [] => <T as NumericElement>::ZERO,
        [single] => *single,
        _ => {
            let mid = values.len() / 2;
            let (left, right) = values.split_at(mid);
            pairwise_sum(left) + pairwise_sum(right)
        }
    }
}

/// Number of elements summed. `10⁵` is far past `bf16`'s stagnation point
/// (`n ≈ 1/ε_bf16 ≈ 256`), which is the regime the accumulator exists for.
const N: usize = 100_000;

/// Deterministic `bf16` operands, all exactly representable by construction.
///
/// Each element is the `bf16` code `0x3F80 | m` for a 7-bit mantissa `m`, i.e. a
/// value in `[1, 2)` that lands exactly on the format's grid. Nothing is rounded
/// on the way in, so the whole measured error belongs to the summation rather
/// than to input quantization. The mantissas come from a fixed LCG so the
/// sequence is scrambled (a monotonic order would make pairwise summation
/// unrepresentatively favourable) and identical on every run and platform.
fn operands() -> Vec<Bf16> {
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    (0..N)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            Bf16(0x3F80 | ((state >> 33) as u16 & 0x007F))
        })
        .collect()
}

#[test]
fn bf16_accumulator_beats_in_type_summation() {
    let elements = operands();
    assert_eq!(elements.len(), N);

    // Reference sum in f64. Its own error is bounded by n·u₆₄·Σ|xᵢ| ≈
    // 10⁵ · 5.6e-17 · 1.5e5 ≈ 8e-7 — five orders below the f32 bound derived
    // below, so it is a valid oracle for both paths.
    let exact: f64 = elements.iter().map(|&x| NumericElement::to_f64(x)).sum();
    let abs_sum: f64 = elements
        .iter()
        .map(|&x| NumericElement::to_f64(x).abs())
        .sum();

    // ── Error bound derivation ──
    //
    // For pairwise summation of n values the standard forward error bound is
    //     |Ŝ − S| ≤ ⌈log₂ n⌉ · u · Σ|xᵢ| / (1 − ⌈log₂ n⌉·u)
    // (Higham, *Accuracy and Stability of Numerical Algorithms*, 2nd ed., §4.2),
    // where u is the accumulator's unit roundoff. The denominator is 1 − 1e-6
    // here, so it is dropped and the linear form used directly.
    //
    // u depends only on the ACCUMULATOR, which is the entire point of the
    // associated type: binary32 has a 24-bit significand, so u₃₂ = 2⁻²⁴.
    // Widening bf16 → f32 is exact (8-bit significand into 24), so it
    // contributes no error term of its own.
    const U_F32: f64 = 1.0 / 16_777_216.0; // 2⁻²⁴
    let depth = (N as f64).log2().ceil(); // ⌈log₂ 10⁵⌉ = 17
    let bound = depth * U_F32 * abs_sum;

    // ── Accumulator path: widen to f32, reduce, narrow once at the end ──
    let widened: Vec<<Bf16 as FloatElement>::Accumulator> =
        elements.iter().map(|&x| x.to_accumulator()).collect();
    let via_accumulator = pairwise_sum(&widened);
    let accumulator_error = (f64::from(via_accumulator) - exact).abs();

    // ── In-type path: reduce in bf16 itself ──
    let in_type = pairwise_sum(&elements);
    let in_type_error = (NumericElement::to_f64(in_type) - exact).abs();

    assert!(
        accumulator_error <= bound,
        "accumulator path must satisfy the pairwise bound: \
         error {accumulator_error:.6e} > bound {bound:.6e} (exact {exact}, n {N})"
    );

    // The in-type path is outside the same bound by construction, not by luck:
    // the exact sum is ≈1.5e5, where one bf16 ulp is 2^17 · 2⁻⁷ = 1024. No bf16
    // value within `bound` (≈0.15) of the exact sum exists unless the sum lands
    // on a grid point, so the representation alone puts this path roughly four
    // orders outside — before any accumulated rounding is counted.
    assert!(
        in_type_error > bound,
        "in-type bf16 path must fall outside the f32 bound: \
         error {in_type_error:.6e} <= bound {bound:.6e}"
    );

    // The separation is the claim, so state it as one, with the margin derived
    // rather than picked: bf16's grid spacing at |S| ≈ 1.5e5 is 2¹⁷·2⁻⁷ = 1024,
    // i.e. ~6700 bounds wide, so the in-type result misses by at least a few
    // hundred unless it happens to land on a grid point. A 100× floor sits an
    // order below even the half-spacing figure (512 ≈ 3400 bounds), so it
    // cannot be met by a near-miss yet cannot fail on representable luck.
    assert!(
        in_type_error > bound * 100.0,
        "widening must buy orders of magnitude: in-type {in_type_error:.6e} vs \
         bound {bound:.6e} (accumulator {accumulator_error:.6e})"
    );
}

/// The accumulator choice per type, pinned so a future edit cannot silently
/// re-route a reduced-precision reduction back into its own format.
#[test]
fn accumulator_is_identity_for_full_precision_and_f32_for_reduced() {
    fn assert_accumulator<T, A>()
    where
        T: FloatElement<Accumulator = A>,
        A: FloatElement,
    {
    }

    assert_accumulator::<f32, f32>();
    assert_accumulator::<f64, f64>();
    assert_accumulator::<F32, F32>();
    assert_accumulator::<F64, F64>();

    assert_accumulator::<F16, f32>();
    assert_accumulator::<Bf16, f32>();
    assert_accumulator::<Bf8, f32>();
    assert_accumulator::<Bf4, f32>();
    assert_accumulator::<F8, f32>();
    assert_accumulator::<F4, f32>();
}

/// Widening into the accumulator must be exact for every implementor, and the
/// `f64` route must not silently narrow the double-precision types — the
/// specific hazard a naive `f32`-based conversion would introduce.
#[test]
fn widening_into_the_accumulator_is_exact() {
    // f64 keeps every digit: an f32-routed conversion would truncate here.
    let precise = core::f64::consts::PI;
    assert_eq!(
        FloatElement::to_accumulator(precise),
        precise,
        "f64 widening"
    );
    assert_eq!(
        FloatElement::to_accumulator(F64(precise)).0,
        precise,
        "F64 widening"
    );
    assert_eq!(
        <f64 as FloatElement>::from_accumulator(precise),
        precise,
        "f64 narrowing"
    );

    // Reduced precision widens exactly, so the widened value re-narrows to the
    // identical bit pattern (a genuine round trip, not an approximate one).
    for code in 0u16..=u16::MAX {
        let value = Bf16(code);
        if NumericElement::is_nan(value) {
            continue;
        }
        let widened = value.to_accumulator();
        assert_eq!(
            f64::from(widened),
            NumericElement::to_f64(value),
            "Bf16({code:#06X}) widening is exact"
        );
        assert_eq!(
            <Bf16 as FloatElement>::from_accumulator(widened).0,
            code,
            "Bf16({code:#06X}) round trips through its accumulator"
        );
    }
}
