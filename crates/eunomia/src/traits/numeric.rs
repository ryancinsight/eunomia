//! The core [`NumericElement`] trait — the monomorphization extension point
//! for operations across all numeric precisions.

use super::{private, CastFrom};

/// Core numeric element trait. The main extension point for monomorphized operations across all precisions.
///
/// # Source constructors
///
/// [`from_f64`](super::FloatElement::from_f64) lives on `FloatElement` (precision-correct for
/// half-precision types via `half::f{16,bf16}::from_f64`). Integer callers
/// use the literal `v as Self` truncating cast natively. There is no generic
/// `from_usize` on `NumericElement` for the same reason — the per-type route
/// is selected explicitly so callers express precision-correct construction
/// (literal cast for ints, `FloatElement::from_f64(v as f64)` round-trip for
/// floats).
pub trait NumericElement:
    private::Sealed
    + Copy
    + Default
    + Send
    + Sync
    + 'static
    + PartialOrd
    + PartialEq
    + core::fmt::Debug
    + core::ops::Add<Output = Self>
    + core::ops::AddAssign
    + core::ops::Sub<Output = Self>
    + core::ops::SubAssign
    + core::ops::Mul<Output = Self>
    + core::ops::MulAssign
    + core::ops::Div<Output = Self>
    + CastFrom<i32>
{
    /// Additive identity.
    const ZERO: Self;
    /// Multiplicative identity.
    const ONE: Self;
    /// IEEE 754 not-a-number sentinel.
    const NAN: Self;
    /// IEEE 754 positive infinity.
    const INFINITY: Self;
    /// Number of bytes per element.
    const BYTE_WIDTH: usize;
    /// Bitwise representation with all bits set to 1.
    const ALL_ONES: Self;
    /// IEEE 754 sign-bit mask: only the most-significant bit is set.
    ///
    /// XOR-ing any value with this mask negates it (flips the sign bit).
    /// Used by the default `SimdKernel::neg` implementation (hermes-simd-core) to
    /// avoid subtraction, which is not universally available across SIMD backends.
    const SIGN_MASK: Self;

    /// Absolute value.
    fn abs(self) -> Self;
    /// Scalar fused multiply-add: (self * b) + c.
    fn scalar_fmadd(self, b: Self, c: Self) -> Self;
    /// Square root. Floats follow IEEE 754 (`NaN` for negative inputs); integers
    /// return the exact floor integer square root (`isqrt`), with negative signed
    /// inputs defined to return 0 (integers have no `NaN` to signal the domain
    /// error). No `f64` round-trip, so the integer result is exact for all operands.
    fn sqrt(self) -> Self;
    /// Returns true if finite.
    fn is_finite(self) -> bool;
    /// Returns true if NaN.
    fn is_nan(self) -> bool;
    /// Cast to f64.
    fn to_f64(self) -> f64;
    /// Bitwise AND.
    fn bitand(self, rhs: Self) -> Self;
    /// Bitwise OR.
    fn bitor(self, rhs: Self) -> Self;
    /// Bitwise XOR.
    fn bitxor(self, rhs: Self) -> Self;
    /// Count set bits (population count).
    fn count_ones(self) -> u32;

    /// Elementwise minimum: returns `self` if `self <= other`, else `other`.
    ///
    /// Default: uses `PartialOrd` comparison. Concrete impls (e.g. `f32`, `f64`) may
    /// override with a hardware intrinsic.
    #[inline(always)]
    fn min_scalar(self, other: Self) -> Self
    where
        Self: PartialOrd,
    {
        if self <= other {
            self
        } else {
            other
        }
    }

    /// Elementwise maximum: returns `self` if `self >= other`, else `other`.
    ///
    /// Default: uses `PartialOrd` comparison. Concrete impls (e.g. `f32`, `f64`) may
    /// override with a hardware intrinsic.
    #[inline(always)]
    fn max_scalar(self, other: Self) -> Self
    where
        Self: PartialOrd,
    {
        if self >= other {
            self
        } else {
            other
        }
    }

    /// The minimum representable finite value (negative infinity or `i32::MIN`).
    ///
    /// Used as the identity element for `Max` reductions.
    const MIN_VALUE: Self;

    /// The maximum representable finite value (positive infinity or `i32::MAX`).
    ///
    /// Used as the identity element for `Min` reductions.
    const MAX_VALUE: Self;
}
