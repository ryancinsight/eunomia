//! The [`FloatElement`] trait — float-specific conversions and the real
//! transcendental surface (libm-backed, native precision).

use super::{private, NumericElement};

/// Float-specific capabilities.
pub trait FloatElement: private::Sealed + NumericElement {
    /// Convert from f32.
    fn from_f32(val: f32) -> Self;
    /// Convert from f64.
    fn from_f64(val: f64) -> Self;
    /// Cast to f32.
    fn to_f32(self) -> f32;

    // ── Transcendental / real-math surface ──
    //
    // Default implementations route through `f32` via `libm` — native for `f32`
    // and the correct reduced-precision path for `f16`/`bf16` (no hardware
    // transcendentals). `f64` overrides each with the native double-precision
    // `libm` function so it is not widen-narrowed.

    /// `e^self`.
    #[inline]
    fn exp(self) -> Self {
        Self::from_f32(libm::expf(self.to_f32()))
    }
    /// Natural logarithm.
    #[inline]
    fn ln(self) -> Self {
        Self::from_f32(libm::logf(self.to_f32()))
    }
    /// Sine (radians).
    #[inline]
    fn sin(self) -> Self {
        Self::from_f32(libm::sinf(self.to_f32()))
    }
    /// Cosine (radians).
    #[inline]
    fn cos(self) -> Self {
        Self::from_f32(libm::cosf(self.to_f32()))
    }
    /// Tangent (radians).
    #[inline]
    fn tan(self) -> Self {
        Self::from_f32(libm::tanf(self.to_f32()))
    }
    /// Hyperbolic sine.
    #[inline]
    fn sinh(self) -> Self {
        Self::from_f32(libm::sinhf(self.to_f32()))
    }
    /// Hyperbolic cosine.
    #[inline]
    fn cosh(self) -> Self {
        Self::from_f32(libm::coshf(self.to_f32()))
    }
    /// Hyperbolic tangent.
    #[inline]
    fn tanh(self) -> Self {
        Self::from_f32(libm::tanhf(self.to_f32()))
    }
    /// Four-quadrant arctangent of `self / other`.
    #[inline]
    fn atan2(self, other: Self) -> Self {
        Self::from_f32(libm::atan2f(self.to_f32(), other.to_f32()))
    }
    /// `self` raised to the power `n`.
    #[inline]
    fn powf(self, n: Self) -> Self {
        Self::from_f32(libm::powf(self.to_f32(), n.to_f32()))
    }
    /// Reciprocal `1 / self`.
    #[inline]
    fn recip(self) -> Self {
        Self::from_f32(1.0 / self.to_f32())
    }
}
