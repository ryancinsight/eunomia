//! The floating-point [`Complex`] surface — the `num_complex::Complex`
//! replacement (norm/conj/arg/exp/ln/sqrt/trig/…), bounded on [`FloatElement`].

use super::Complex;
use crate::traits::{FloatElement, NumericElement};
use core::ops::Neg;

/// Floating-point complex surface — the `num_complex::Complex` replacement.
///
/// Bounded on [`FloatElement`], which supplies the real transcendentals
/// (`exp`/`ln`/`sin`/`cos`/`atan2`/…) and the `NumericElement` arithmetic,
/// constants, `sqrt`, and `abs`. Every operation evaluates in `T`'s native
/// precision (no widen-narrow).
impl<T: FloatElement + Neg<Output = T>> Complex<T> {
    /// The imaginary unit `i` (`0 + 1i`).
    #[inline]
    pub fn i() -> Self {
        Self {
            re: <T as NumericElement>::ZERO,
            im: <T as NumericElement>::ONE,
        }
    }

    /// Squared magnitude `re² + im²`.
    #[inline]
    pub fn norm_sqr(self) -> T {
        self.re * self.re + self.im * self.im
    }

    /// Magnitude `√(re² + im²)`.
    #[inline]
    pub fn norm(self) -> T {
        self.norm_sqr().sqrt()
    }

    /// L1 (taxicab) norm `|re| + |im|`.
    #[inline]
    pub fn l1_norm(self) -> T {
        self.re.abs() + self.im.abs()
    }

    /// Argument (phase) `atan2(im, re)` in radians.
    #[inline]
    pub fn arg(self) -> T {
        self.im.atan2(self.re)
    }

    /// Complex conjugate `re − im·i`.
    #[inline]
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Scale by a real factor.
    #[inline]
    pub fn scale(self, t: T) -> Self {
        Self {
            re: self.re * t,
            im: self.im * t,
        }
    }

    /// Polar decomposition `(magnitude, argument)`.
    #[inline]
    pub fn to_polar(self) -> (T, T) {
        (self.norm(), self.arg())
    }

    /// Construct from polar form `r·(cos θ + i·sin θ)`.
    #[inline]
    pub fn from_polar(r: T, theta: T) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// `cos θ + i·sin θ` (unit-magnitude phasor).
    #[inline]
    pub fn cis(theta: T) -> Self {
        Self {
            re: theta.cos(),
            im: theta.sin(),
        }
    }

    /// Complex exponential `e^self = e^re·(cos im + i·sin im)`.
    #[inline]
    pub fn exp(self) -> Self {
        let r = self.re.exp();
        Self {
            re: r * self.im.cos(),
            im: r * self.im.sin(),
        }
    }

    /// Principal complex natural logarithm `ln|z| + i·arg z`.
    #[inline]
    pub fn ln(self) -> Self {
        Self {
            re: self.norm().ln(),
            im: self.arg(),
        }
    }

    /// Principal complex square root.
    #[inline]
    pub fn sqrt(self) -> Self {
        // √z = √r·(cos(θ/2) + i·sin(θ/2))
        let r = self.norm().sqrt();
        let half_theta = self.arg() * T::from_f32(0.5);
        Self {
            re: r * half_theta.cos(),
            im: r * half_theta.sin(),
        }
    }

    /// `self` raised to a real power via `z^n = r^n·e^{i·n·θ}`.
    #[inline]
    pub fn powf(self, n: T) -> Self {
        let r_n = self.norm().powf(n);
        let n_theta = self.arg() * n;
        Self {
            re: r_n * n_theta.cos(),
            im: r_n * n_theta.sin(),
        }
    }

    /// Complex sine.
    #[inline]
    pub fn sin(self) -> Self {
        Self {
            re: self.re.sin() * self.im.cosh(),
            im: self.re.cos() * self.im.sinh(),
        }
    }

    /// Complex cosine.
    #[inline]
    pub fn cos(self) -> Self {
        Self {
            re: self.re.cos() * self.im.cosh(),
            im: -(self.re.sin() * self.im.sinh()),
        }
    }

    /// Complex hyperbolic tangent.
    #[inline]
    pub fn tanh(self) -> Self {
        let two = T::from_f32(2.0);
        let x2 = self.re * two;
        let y2 = self.im * two;
        let denom = x2.cosh() + y2.cos();
        Self {
            re: x2.sinh() / denom,
            im: y2.sin() / denom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Complex;

    #[test]
    fn new_and_fields() {
        let c = Complex::new(2.0_f64, -3.0);
        assert_eq!(c.re, 2.0);
        assert_eq!(c.im, -3.0);
    }

    #[test]
    fn arithmetic_matches_definition() {
        let a = Complex::new(1.0_f64, 2.0);
        let b = Complex::new(3.0_f64, -1.0);
        assert_eq!(a + b, Complex::new(4.0, 1.0));
        assert_eq!(a - b, Complex::new(-2.0, 3.0));
        // (1+2i)(3-i) = 3 - i + 6i - 2i² = 5 + 5i
        assert_eq!(a * b, Complex::new(5.0, 5.0));
        // (1+2i)/(3-i) = (1+2i)(3+i)/10 = (1+7i)/10
        let q = a / b;
        assert!((q.re - 0.1).abs() < 1e-12 && (q.im - 0.7).abs() < 1e-12);
        assert_eq!(-a, Complex::new(-1.0, -2.0));
    }

    #[test]
    fn float_surface_matches_analytic() {
        let z = Complex::new(3.0_f64, 4.0);
        assert!((z.norm() - 5.0).abs() < 1e-12);
        assert!((z.norm_sqr() - 25.0).abs() < 1e-12);
        assert_eq!(z.conj(), Complex::new(3.0, -4.0));
        // arg(1 + i) = π/4
        let a = Complex::new(1.0_f64, 1.0);
        assert!((a.arg() - core::f64::consts::FRAC_PI_4).abs() < 1e-12);
        // e^{iπ} = -1 + 0i  (Euler's identity)
        let euler = Complex::new(0.0_f64, core::f64::consts::PI).exp();
        assert!((euler.re + 1.0).abs() < 1e-12 && euler.im.abs() < 1e-12);
        // √(2i) = 1 + i   since (1+i)² = 2i
        let s = Complex::new(0.0_f64, 2.0).sqrt();
        assert!((s.re - 1.0).abs() < 1e-12 && (s.im - 1.0).abs() < 1e-12);
        // ln(e) = 1 + 0i
        let l = Complex::new(core::f64::consts::E, 0.0).ln();
        assert!((l.re - 1.0).abs() < 1e-12 && l.im.abs() < 1e-12);
        assert_eq!(Complex::<f64>::i(), Complex::new(0.0, 1.0));
        // from_polar(2, π/2) = 0 + 2i
        let p = Complex::from_polar(2.0_f64, core::f64::consts::FRAC_PI_2);
        assert!(p.re.abs() < 1e-12 && (p.im - 2.0).abs() < 1e-12);
    }
}
