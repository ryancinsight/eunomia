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
    ///
    /// The operation inherits [`NumericElement::min_scalar`] and
    /// [`NumericElement::max_scalar`]: a single NaN bound is ignored, a NaN
    /// input is reduced to the lower bound, and the result preserves the
    /// signed-zero convention of the scalar operations.
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

    /// Bessel function of the first kind, order zero, `J₀(self)`.
    ///
    /// Default routes through `f32` via `libm::j0f`; native-precision types
    /// override with `libm::j0` (the
    /// [`FloatElement::erf`](super::FloatElement::erf) pattern). `J₀` is even:
    /// `bessel_j0(-x) == bessel_j0(x)`.
    #[inline]
    fn bessel_j0(self) -> Self {
        Self::from_f32(libm::j0f(self.to_f32()))
    }

    /// Bessel function of the first kind, order one, `J₁(self)`.
    ///
    /// Default routes through `f32` via `libm::j1f`; native-precision types
    /// override with `libm::j1`. `J₁` is odd: `bessel_j1(-x) == -bessel_j1(x)`.
    #[inline]
    fn bessel_j1(self) -> Self {
        Self::from_f32(libm::j1f(self.to_f32()))
    }

    /// Modified Bessel function of the second kind, order zero, `K₀(self)`
    /// for `self > 0`.
    ///
    /// Abramowitz & Stegun 9.8.5 for `0 < x ≤ 2` and 9.8.6 for `x > 2`,
    /// evaluated in the precision of `Self` (monomorphized per implementor,
    /// not widen-narrowed; libm 0.2.16 provides no `k0`). Both polynomial
    /// forms carry an absolute error below `1e-7` on their stated domains.
    /// `K₀` diverges at the origin and is undefined for `x ≤ 0`, so
    /// non-positive and non-finite arguments return `NaN`.
    #[inline]
    fn bessel_k0(self) -> Self {
        if !(self.is_finite() && self > <Self as NumericElement>::ZERO) {
            return <Self as NumericElement>::NAN;
        }
        if self <= Self::from_f64(2.0) {
            // A&S 9.8.5: K₀(x) = −ln(x/2)·I₀(x) + Σ cₙ (x/2)^{2n}.
            let t1 = (self / Self::from_f64(3.75)) * (self / Self::from_f64(3.75));
            let i0 = <Self as NumericElement>::ONE
                + t1 * (Self::from_f64(3.515_622_9)
                    + t1 * (Self::from_f64(3.089_942_4)
                        + t1 * (Self::from_f64(1.206_749_2)
                            + t1 * (Self::from_f64(0.265_973_2)
                                + t1 * (Self::from_f64(0.036_076_8)
                                    + t1 * Self::from_f64(0.004_581_3))))));
            let t2 = (self * Self::from_f64(0.5)) * (self * Self::from_f64(0.5));
            let correction = Self::from_f64(-0.577_215_66)
                + t2 * (Self::from_f64(0.422_784_20)
                    + t2 * (Self::from_f64(0.230_697_56)
                        + t2 * (Self::from_f64(0.034_885_90)
                            + t2 * (Self::from_f64(0.002_626_98)
                                + t2 * (Self::from_f64(0.000_107_50)
                                    + t2 * Self::from_f64(7.4e-6))))));
            (self * Self::from_f64(0.5)).ln() * -i0 + correction
        } else {
            // A&S 9.8.6: K₀(x) = e^{-x}/√x · Σ dₙ (2/x)^n.
            let t = Self::from_f64(2.0) / self;
            let series = Self::from_f64(1.253_314_14)
                + t * (Self::from_f64(-0.078_323_58)
                    + t * (Self::from_f64(0.021_895_68)
                        + t * (Self::from_f64(-0.010_624_46)
                            + t * (Self::from_f64(0.005_878_72)
                                + t * (Self::from_f64(-0.002_515_40)
                                    + t * Self::from_f64(0.000_532_08))))));
            (-self).exp() / self.sqrt() * series
        }
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
