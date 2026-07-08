//! [`RealField`]/[`ComplexField`] impls.
//!
//! `RealField` is implemented per primitive float; `ComplexField` is provided by
//! two blanket impls — one over every [`RealField`] (the real case) and one over
//! every `Complex<RealField>` (the complex case) — so the field surface is
//! defined once per case and fully monomorphizes.

use crate::traits::{ComplexField, FloatElement, NumericElement, RealField};
use crate::types::Complex;

impl RealField for f32 {
    const PI: Self = core::f32::consts::PI;
    const TAU: Self = core::f32::consts::TAU;
    const FRAC_PI_2: Self = core::f32::consts::FRAC_PI_2;
    const E: Self = core::f32::consts::E;
    const LN_2: Self = core::f32::consts::LN_2;
    const SQRT_2: Self = core::f32::consts::SQRT_2;
    const EPSILON: Self = f32::EPSILON;

    #[inline]
    fn copysign(self, sign: Self) -> Self {
        libm::copysignf(self, sign)
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        f32::is_sign_positive(self)
    }
}

impl RealField for f64 {
    const PI: Self = core::f64::consts::PI;
    const TAU: Self = core::f64::consts::TAU;
    const FRAC_PI_2: Self = core::f64::consts::FRAC_PI_2;
    const E: Self = core::f64::consts::E;
    const LN_2: Self = core::f64::consts::LN_2;
    const SQRT_2: Self = core::f64::consts::SQRT_2;
    const EPSILON: Self = f64::EPSILON;

    #[inline]
    fn copysign(self, sign: Self) -> Self {
        libm::copysign(self, sign)
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        f64::is_sign_positive(self)
    }
}

/// Real case: every [`RealField`] is a (degenerate) [`ComplexField`] with zero
/// imaginary part; the transcendentals are the real ones.
impl<T: RealField> ComplexField for T {
    type RealPart = T;

    #[inline]
    fn from_real(re: T) -> Self {
        re
    }
    #[inline]
    fn real(self) -> T {
        self
    }
    #[inline]
    fn imaginary(self) -> T {
        <T as NumericElement>::ZERO
    }
    #[inline]
    fn modulus(self) -> T {
        self.abs()
    }
    #[inline]
    fn modulus_squared(self) -> T {
        self * self
    }
    #[inline]
    fn argument(self) -> T {
        if self >= <T as NumericElement>::ZERO {
            <T as NumericElement>::ZERO
        } else {
            <T as RealField>::PI
        }
    }
    #[inline]
    fn conjugate(self) -> Self {
        self
    }
    #[inline]
    fn scale(self, factor: T) -> Self {
        self * factor
    }
    #[inline]
    fn sqrt(self) -> Self {
        <T as NumericElement>::sqrt(self)
    }
    #[inline]
    fn exp(self) -> Self {
        <T as FloatElement>::exp(self)
    }
    #[inline]
    fn ln(self) -> Self {
        <T as FloatElement>::ln(self)
    }
    #[inline]
    fn powf(self, n: T) -> Self {
        <T as FloatElement>::powf(self, n)
    }
    #[inline]
    fn sin(self) -> Self {
        <T as FloatElement>::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        <T as FloatElement>::cos(self)
    }
}

/// Complex case: `Complex<T>` over a real field `T` is a [`ComplexField`] with
/// `RealPart = T`, delegating to the native complex operations.
impl<T: RealField> ComplexField for Complex<T> {
    type RealPart = T;

    #[inline]
    fn from_real(re: T) -> Self {
        Complex::new(re, <T as NumericElement>::ZERO)
    }
    #[inline]
    fn real(self) -> T {
        self.re
    }
    #[inline]
    fn imaginary(self) -> T {
        self.im
    }
    #[inline]
    fn modulus(self) -> T {
        self.norm()
    }
    #[inline]
    fn modulus_squared(self) -> T {
        self.norm_sqr()
    }
    #[inline]
    fn argument(self) -> T {
        self.arg()
    }
    #[inline]
    fn conjugate(self) -> Self {
        self.conj()
    }
    #[inline]
    fn scale(self, factor: T) -> Self {
        Complex::scale(self, factor)
    }
    #[inline]
    fn sqrt(self) -> Self {
        Complex::sqrt(self)
    }
    #[inline]
    fn exp(self) -> Self {
        Complex::exp(self)
    }
    #[inline]
    fn ln(self) -> Self {
        Complex::ln(self)
    }
    #[inline]
    fn powf(self, n: T) -> Self {
        Complex::powf(self, n)
    }
    #[inline]
    fn sin(self) -> Self {
        Complex::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        Complex::cos(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::{ComplexField, NumericElement, RealField};
    use crate::types::Complex;

    #[test]
    fn real_field_constants_and_sign() {
        assert!((<f64 as RealField>::PI - core::f64::consts::PI).abs() < 1e-15);
        assert!((<f64 as RealField>::TAU - 2.0 * core::f64::consts::PI).abs() < 1e-15);
        assert!((-3.0_f64).is_sign_negative());
        assert_eq!(5.0_f64.copysign(-1.0), -5.0);
        assert_eq!(2.0_f64.clamp(0.0, 1.0), 1.0);
        assert!((180.0_f64.to_radians() - core::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn complex_field_over_real_scalar() {
        // f64 as a degenerate ComplexField.
        let x = 3.0_f64;
        assert_eq!(ComplexField::modulus(x), 3.0);
        assert_eq!(ComplexField::imaginary(x), 0.0);
        assert_eq!(ComplexField::conjugate(x), 3.0);
        assert_eq!(ComplexField::argument(-2.0_f64), core::f64::consts::PI);
        assert!((ComplexField::exp(1.0_f64) - core::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn complex_field_over_complex() {
        let z = Complex::new(3.0_f64, 4.0);
        assert_eq!(ComplexField::modulus(z), 5.0);
        assert_eq!(ComplexField::modulus_squared(z), 25.0);
        assert_eq!(ComplexField::real(z), 3.0);
        assert_eq!(ComplexField::imaginary(z), 4.0);
        assert_eq!(ComplexField::conjugate(z), Complex::new(3.0, -4.0));
        // e^{iπ} = -1
        let euler: Complex<f64> = ComplexField::exp(Complex::new(0.0, core::f64::consts::PI));
        assert!((euler.re + 1.0).abs() < 1e-12 && euler.im.abs() < 1e-12);
        // generic over the field: works for both f64 and Complex<f64>
        fn norm_sq<F: ComplexField>(x: F) -> F::RealPart {
            x.modulus_squared()
        }
        assert_eq!(norm_sq(z), 25.0);
        assert_eq!(norm_sq(2.0_f64), 4.0);
    }

    /// Cross-type default-equivalence: `Complex::<f64>::default()` MUST
    /// resolve to `{ re: 0.0, im: 0.0 }`. Pins the `derive(Default)`
    /// resolution at `crates/eunomia/src/types/complex/mod.rs:32-35`.
    /// Drift here means a sentinel-default sneaked into the `Default`
    /// derivation — that's exactly the silent-drift class this fixture exists
    /// to catch (CR-EUNOMIA-COMPLEX, ADR 0006 "Why not the alternatives").
    #[test]
    fn eunomia_complex64_default_is_zero() {
        let z = Complex::<f64>::default();
        assert_eq!(z.re, 0.0_f64);
        assert_eq!(z.im, 0.0_f64);
        assert_eq!(z, Complex::<f64>::new(0.0, 0.0));
    }

    /// f32 parity companion for the default-equivalence pin.
    #[test]
    fn eunomia_complex32_default_is_zero() {
        let z = Complex::<f32>::default();
        assert_eq!(z.re, 0.0_f32);
        assert_eq!(z.im, 0.0_f32);
        assert_eq!(z, Complex::<f32>::new(0.0, 0.0));
    }

    /// Pinned `modulus` of `Complex::<f64>::default()` returns `0.0`
    /// (NOT NaN). If `Complex::default()` is ever rewritten to a sentinel,
    /// this fails loudly: the post-ADR §1 `ComplexField::zero()` default
    /// MUST conform with this contract.
    #[test]
    fn eunomia_complex64_default_has_zero_modulus() {
        let z = Complex::<f64>::default();
        assert_eq!(<Complex::<f64> as ComplexField>::modulus(z), 0.0_f64,);
        let m = <Complex<f64> as ComplexField>::modulus(z);
        assert!(!m.is_nan(), "modulus of `default()` is unexpectedly NaN");
    }

    // ───── Post-ADR-0006 §1 fixtures (frozen via `#[cfg(any())]`) ────────────
    // Compile ONLY once ADR-0006 §1 ships (additive `ComplexField::zero()`
    // + `ComplexField::one()` defaults on `crate::traits::field::ComplexField`).
    // Activating these tests is a one-liner: replace `#[cfg(any())]` with
    // `#[cfg(test)]`. Their job is to lock §1's contract against later drift.

    /// `<f64 as ComplexField>::zero() == 0.0` — the degenerate-real case.#[cfg(test)]
    #[test]
    fn complex_field_zero_over_real_scalar_f64() {
        assert_eq!(<f64 as ComplexField>::zero(), 0.0_f64);
        assert_eq!(<f32 as ComplexField>::zero(), 0.0_f32);
    }

    /// `<f64 as ComplexField>::one() == 1.0` — the degenerate-real case.#[cfg(test)]
    #[test]
    fn complex_field_one_over_real_scalar_f64() {
        assert_eq!(<f64 as ComplexField>::one(), 1.0_f64);
        assert_eq!(<f32 as ComplexField>::one(), 1.0_f32);
    }

    /// `<Complex<f64> as ComplexField>::zero() == Complex::new(0.0, 0.0)`.#[cfg(test)]
    #[test]
    fn complex_field_zero_over_complex_via_default() {
        let z = <Complex<f64> as ComplexField>::zero();
        assert_eq!(z, Complex::<f64>::new(0.0, 0.0));
        let z32 = <Complex<f32> as ComplexField>::zero();
        assert_eq!(z32, Complex::<f32>::new(0.0, 0.0));
    }

    /// `<Complex<f64> as ComplexField>::one() == Complex::new(1.0, 0.0)`.#[cfg(test)]
    #[test]
    fn complex_field_one_over_complex_via_default() {
        let z = <Complex<f64> as ComplexField>::one();
        assert_eq!(z, Complex::<f64>::new(1.0, 0.0));
        let z32 = <Complex<f32> as ComplexField>::one();
        assert_eq!(z32, Complex::<f32>::new(1.0, 0.0));
    }

    /// `ComplexField::zero()` body MUST equal `from_real(<RealPart as
    /// NumericElement>::ZERO)`. This is the body ADR-0006 §1 prescribes.
    /// If anyone hand-rolls an override divergent from the additive default,
    /// this fixture surfaces it.#[cfg(test)]
    #[test]
    fn complex_field_zero_routes_through_numeric_element_zero() {
        assert_eq!(
            <Complex<f64> as ComplexField>::zero(),
            <Complex<f64> as ComplexField>::from_real(<f64 as NumericElement>::ZERO),
        );
        assert_eq!(
            <Complex<f32> as ComplexField>::one(),
            <Complex<f32> as ComplexField>::from_real(<f32 as NumericElement>::ONE),
        );
    }
}
