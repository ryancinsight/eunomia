//! Scalar **field** abstractions — the eunomia replacement for
//! `nalgebra::RealField` / `nalgebra::ComplexField`.
//!
//! These let generic numeric/geometry code be written once over "a real scalar"
//! ([`RealField`]) or "a real-or-complex scalar" ([`ComplexField`]) without
//! pulling in nalgebra. The *linear algebra* built on top (matrices,
//! decompositions) lives in leto (CPU) / hephaestus (GPU); eunomia owns only the
//! scalar field vocabulary.

use super::{FloatElement, NumericElement};
use core::ops::{Add, Div, Mul, Neg, Sub};

/// An ordered real scalar field: the [`FloatElement`] math surface plus a total
/// (partial) order, the standard mathematical constants, and sign helpers.
///
/// The `nalgebra::RealField` analogue. Implemented for `f32`/`f64`.
pub trait RealField: FloatElement + PartialOrd + Neg<Output = Self> {
    /// Archimedes' constant π.
    const PI: Self;
    /// The full turn, 2π.
    const TAU: Self;
    /// π/2.
    const FRAC_PI_2: Self;
    /// Euler's number e.
    const E: Self;
    /// Natural logarithm of 2.
    const LN_2: Self;
    /// √2.
    const SQRT_2: Self;
    /// Machine epsilon (the difference between 1 and the next representable value).
    const EPSILON: Self;

    /// Positive infinity.
    #[inline]
    fn infinity() -> Self {
        <Self as NumericElement>::INFINITY
    }
    /// Negative infinity.
    #[inline]
    fn neg_infinity() -> Self {
        -<Self as NumericElement>::INFINITY
    }
    /// Not-a-number.
    #[inline]
    fn nan() -> Self {
        <Self as NumericElement>::NAN
    }
    /// The smallest finite value.
    #[inline]
    fn min_value() -> Self {
        <Self as NumericElement>::MIN_VALUE
    }
    /// The largest finite value.
    #[inline]
    fn max_value() -> Self {
        <Self as NumericElement>::MAX_VALUE
    }

    /// A value with the magnitude of `self` and the sign of `sign`.
    fn copysign(self, sign: Self) -> Self;

    /// Whether `self` carries a positive sign bit (`+0`, `+∞`, positive finite).
    fn is_sign_positive(self) -> bool;

    /// Whether `self` carries a negative sign bit.
    #[inline]
    fn is_sign_negative(self) -> bool {
        !self.is_sign_positive()
    }

    /// Restrict `self` to `[min, max]` (assumes `min ≤ max`).
    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        self.max_scalar(min).min_scalar(max)
    }

    /// Convert an angle in degrees to radians.
    #[inline]
    fn to_radians(self) -> Self {
        self * (Self::PI / Self::from_f64(180.0))
    }

    /// Convert an angle in radians to degrees.
    #[inline]
    fn to_degrees(self) -> Self {
        self * (Self::from_f64(180.0) / Self::PI)
    }
}

/// A scalar **field** that may be real or complex — the `nalgebra::ComplexField`
/// analogue.
///
/// Generic code bounded on `T: ComplexField` runs uniformly for `f32`/`f64`
/// (real) and [`Complex<f32>`](crate::Complex)/`Complex<f64>` (complex); the
/// associated [`RealPart`](ComplexField::RealPart) is the underlying real field.
/// Provided by blanket impls over every [`RealField`] and every
/// `Complex<RealField>`, so there is one definition per case (no per-type
/// duplication) and all of it monomorphizes.
pub trait ComplexField:
    Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    /// The real scalar field underlying this field.
    type RealPart: RealField;

    /// Embed a real value as `re + 0i` (identity for a real field).
    fn from_real(re: Self::RealPart) -> Self;
    /// Real component.
    fn real(self) -> Self::RealPart;
    /// Imaginary component (`0` for a real field).
    fn imaginary(self) -> Self::RealPart;
    /// Magnitude `|z|`.
    fn modulus(self) -> Self::RealPart;
    /// Squared magnitude `|z|²` (no `sqrt`).
    fn modulus_squared(self) -> Self::RealPart;
    /// Argument (phase) in radians (`0`/`π` for non-negative/negative reals).
    fn argument(self) -> Self::RealPart;
    /// Complex conjugate (identity for a real field).
    fn conjugate(self) -> Self;
    /// Multiply by a real factor.
    fn scale(self, factor: Self::RealPart) -> Self;

    /// Principal square root.
    fn sqrt(self) -> Self;
    /// Exponential `e^self`.
    fn exp(self) -> Self;
    /// Principal natural logarithm.
    fn ln(self) -> Self;
    /// `self` raised to a real power.
    fn powf(self, n: Self::RealPart) -> Self;
    /// Sine.
    fn sin(self) -> Self;
    /// Cosine.
    fn cos(self) -> Self;

    /// Additive identity: `0 + 0i` for a complex field, `0` for a real field.
    /// Default body routes through `NumericElement::ZERO` via `from_real`.
    /// Per ADR 0006 \u00a71 (CR-EUNOMIA-COMPLEX): no per-implementor override is
    /// sound because the real blanket impl returns `re` from
    /// `from_real(re)` and the `Complex<T>` blanket impl returns
    /// `Complex::new(re, <T as NumericElement>::ZERO)`; both round-trip
    /// identically through `Self::from_real(<Self::RealPart as NumericElement>::ZERO)`.
    #[inline]
    #[must_use]
    fn zero() -> Self {
        Self::from_real(<Self::RealPart as NumericElement>::ZERO)
    }

    /// Multiplicative identity: `1 + 0i` for a complex field, `1` for a real field.
    /// Default body routes through `NumericElement::ONE` via `from_real`.
    /// See [`zero`](Self::zero) for the routing rationale.
    #[inline]
    #[must_use]
    fn one() -> Self {
        Self::from_real(<Self::RealPart as NumericElement>::ONE)
    }
}
