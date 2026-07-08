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
    /// Inverse cosine (radians).
    #[inline]
    fn acos(self) -> Self {
        Self::from_f32(libm::acosf(self.to_f32()))
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

    // ── Rounding / sign ──

    /// Largest integer `≤ self`.
    #[inline]
    fn floor(self) -> Self {
        Self::from_f32(libm::floorf(self.to_f32()))
    }
    /// Smallest integer `≥ self`.
    #[inline]
    fn ceil(self) -> Self {
        Self::from_f32(libm::ceilf(self.to_f32()))
    }
    /// Nearest integer, half away from zero.
    #[inline]
    fn round(self) -> Self {
        Self::from_f32(libm::roundf(self.to_f32()))
    }
    /// Integer part (toward zero).
    #[inline]
    fn trunc(self) -> Self {
        Self::from_f32(libm::truncf(self.to_f32()))
    }
    /// Sign of `self`: `1` for positive/`+0`, `-1` for negative/`-0`, NaN for
    /// NaN (matching `f64::signum` / `num_traits::Float::signum`).
    #[inline]
    fn signum(self) -> Self {
        let x = self.to_f32();
        if x.is_nan() {
            Self::from_f32(x)
        } else {
            Self::from_f32(libm::copysignf(1.0, x))
        }
    }

    /// `self` raised to an integer power via exponentiation by squaring.
    ///
    /// Exact for integer exponents and correct for negative bases (unlike
    /// [`powf`](Self::powf)). A default over the [`NumericElement`] arithmetic —
    /// no per-type implementation needed, so no precision is lost.
    #[inline]
    fn powi(self, mut n: i32) -> Self {
        let mut base = self;
        if n < 0 {
            base = <Self as NumericElement>::ONE / base;
            n = -n;
        }
        let mut acc = <Self as NumericElement>::ONE;
        while n > 0 {
            if n & 1 == 1 {
                acc *= base;
            }
            base *= base;
            n >>= 1;
        }
        acc
    }

    /// Error function `erf(self) = 2/√π ∫₀ˢᵉˡᶠ e^(-t²) dt`.
    ///
    /// Default routes through `f32`; native-precision types override.
    #[inline]
    fn erf(self) -> Self {
        Self::from_f32(libm::erff(self.to_f32()))
    }

    /// Complementary error function `erfc(self) = 1 - erf(self)`, computed without
    /// the cancellation error of `1 - erf` for large `self`.
    #[inline]
    fn erfc(self) -> Self {
        Self::from_f32(libm::erfcf(self.to_f32()))
    }

    /// Natural logarithm of the absolute value of the gamma function, `ln|Γ(self)|`.
    #[inline]
    fn lgamma(self) -> Self {
        Self::from_f32(libm::lgammaf(self.to_f32()))
    }

    /// Machine epsilon (`nalgebra::RealField::default_epsilon` compatibility alias).
    ///
    /// Returns the smallest positive value `ε` such that `1.0 + ε != 1.0`.
    /// Prefer `<T as eunomia::RealField>::EPSILON` in new code.
    #[inline]
    fn default_epsilon() -> Self
    where
        Self: crate::RealField,
    {
        <Self as crate::RealField>::EPSILON
    }

    /// Returns `π` as this float type (nalgebra compatibility alias for `RealField::PI`).
    #[inline]
    fn pi() -> Self
    where
        Self: crate::RealField,
    {
        <Self as crate::RealField>::PI
    }

    /// Componentwise maximum (method form; prefer `NumericElement::max_scalar` in new code).
    #[inline]
    fn max(self, other: Self) -> Self {
        <Self as NumericElement>::max_scalar(self, other)
    }

    /// Componentwise minimum (method form; prefer `NumericElement::min_scalar` in new code).
    #[inline]
    fn min(self, other: Self) -> Self {
        <Self as NumericElement>::min_scalar(self, other)
    }

    /// L2 (Euclidean) norm — for scalars this is the absolute value.
    #[inline]
    fn norm(self) -> Self {
        <Self as NumericElement>::abs(self)
    }
}
