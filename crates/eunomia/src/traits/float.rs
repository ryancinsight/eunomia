//! The [`FloatElement`] trait — float-specific conversions and the real
//! transcendental surface (libm-backed, native precision).

use super::{private, NumericElement};

/// Float-specific capabilities.
pub trait FloatElement: private::Sealed + NumericElement {
    /// The type a reduction over `Self` accumulates in.
    ///
    /// This is the crate's **only** sanctioned widening route. Any other way of
    /// computing in a type other than `T` inside a `T`-generic body is a fake
    /// generic — cast `T` to something concrete, work there, cast back — so the
    /// widening is declared here instead: per type, in the contract, where it
    /// carries a justification and where the caller can see it in the signature.
    ///
    /// # Numerical analysis
    ///
    /// The worst-case relative error of a sum of `n` values is `(n − 1)·ε` for
    /// sequential accumulation and `⌈log₂ n⌉·ε` for pairwise (tree)
    /// accumulation, where `ε` is the **accumulator's** machine epsilon — not
    /// the element type's. Each choice below follows from where `n·ε` turns
    /// destructive for that format:
    ///
    /// - **Identity for `f32`, `f64`, [`F32`](crate::F32), [`F64`](crate::F64).**
    ///   `ε₃₂ = 2⁻²³ ≈ 1.2e-7` and `ε₆₄ = 2⁻⁵² ≈ 2.2e-16` leave headroom at
    ///   realistic `n`, and widening `f32` to `f64` is a cost rather than a
    ///   correctness fix: it doubles traffic on a reduction that is usually
    ///   bandwidth-bound and halves the SIMD lane count. A caller who wants
    ///   double-precision accumulation over single-precision data expresses that
    ///   by instantiating the algorithm at `f64`, which keeps the choice visible.
    ///
    /// - **`f32` for [`F16`](crate::F16), [`Bf16`](crate::Bf16) and the sub-byte
    ///   formats.** These are storage and bandwidth formats, not accumulation
    ///   formats. The binding limit is not accuracy but *stagnation*: a running
    ///   sum stops advancing once it exceeds `1/ε` times the next addend, since
    ///   the addend then rounds away entirely. For `bf16` (8-bit significand,
    ///   `ε ≈ 3.9e-3`) that happens at `n ≈ 256`; for `f16` (11-bit,
    ///   `ε ≈ 4.9e-4`) at `n ≈ 2048`; for the 4- and 8-bit formats far sooner.
    ///   Summing 10⁵ `bf16` values in `bf16` therefore does not merely lose
    ///   digits, it converges to a wrong answer. Widening to `f32` is **exact**
    ///   for every one of these types — each has at most an 11-bit significand
    ///   and an exponent range inside `f32`'s — so the widening contributes no
    ///   error of its own and the reduction inherits `ε₃₂`.
    ///
    /// Widening these past `f32` buys little further: the inputs were already
    /// quantized to ≤11 significand bits, so `f64` would shrink only the
    /// summation error, which `f32` already holds well below that quantization.
    ///
    /// Reductions must still declare their accumulation *order* — pairwise and
    /// sequential differ by the factor above, and the bound is only meaningful
    /// alongside the order that produced it.
    type Accumulator: FloatElement;

    /// Convert from f32.
    fn from_f32(val: f32) -> Self;
    /// Convert from f64.
    fn from_f64(val: f64) -> Self;
    /// Cast to f32.
    fn to_f32(self) -> f32;

    /// Widen `self` into its [`Accumulator`](Self::Accumulator).
    ///
    /// Exact for every implementor: the accumulator is either `Self` or a
    /// strictly wider format, and the `f64` route below represents both without
    /// rounding (`f32`→`f64` and `f64`→`f64` are exact, as is any significand of
    /// ≤11 bits). Routing through `f32` instead would silently narrow the `f64`
    /// case — precisely the defect this seam exists to prevent.
    #[inline]
    fn to_accumulator(self) -> Self::Accumulator {
        Self::Accumulator::from_f64(<Self as NumericElement>::to_f64(self))
    }

    /// Narrow an accumulated value back to `Self`, rounding to nearest.
    ///
    /// The inverse of [`to_accumulator`](Self::to_accumulator), lossy exactly
    /// where `Self` is narrower than its accumulator — which is the point: the
    /// result re-enters the storage format only once, at the end of the
    /// reduction, instead of once per element.
    #[inline]
    fn from_accumulator(acc: Self::Accumulator) -> Self {
        Self::from_f64(<Self::Accumulator as NumericElement>::to_f64(acc))
    }

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
    /// Sign-preserving cube root — defined for all reals (`cbrt(-8) == -2`),
    /// unlike `powf(self, 1/3)` which is NaN for negative operands.
    ///
    /// Default routes through `f32` via `libm`; `f64` (and the `F64` wrapper)
    /// override with the native double-precision path.
    #[inline]
    fn cbrt(self) -> Self {
        Self::from_f32(libm::cbrtf(self.to_f32()))
    }
    /// Reciprocal `1 / self`.
    #[inline]
    fn recip(self) -> Self {
        Self::from_f32(1.0 / self.to_f32())
    }
    /// Reciprocal square root `1 / √self`, computed as `sqrt(self).recip()` at
    /// native precision (no f32 widen-narrow). `NaN` for negative inputs.
    #[inline]
    fn rsqrt(self) -> Self {
        self.sqrt().recip()
    }
    /// Real `n`th root, sign-preserving for odd `n` (`nth_root(-8, 3) == -2`),
    /// `NaN` for negative operands with even `n`. `n == 0` is undefined and
    /// returns `NaN`.
    ///
    /// libm has no general-root primitive, so the default composes
    /// `powf(|x|, 1/n)` with `copysign` for the odd case. `f64` (and the `F64`
    /// wrapper) override with the native double-precision path.
    #[inline]
    fn nth_root(self, n: u32) -> Self {
        if n == 0 {
            return <Self as NumericElement>::NAN;
        }
        let x = self.to_f32();
        let root = if n % 2 == 1 {
            libm::copysignf(libm::powf(libm::fabsf(x), 1.0 / n as f32), x)
        } else {
            libm::powf(x, 1.0 / n as f32)
        };
        Self::from_f32(root)
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

    /// Base-10 logarithm, `log₁₀(self)`.
    ///
    /// Default routes through `f32` via `libm`; `f64` overrides with the
    /// native double-precision path.
    #[inline]
    fn log10(self) -> Self {
        Self::from_f32(libm::log10f(self.to_f32()))
    }

    /// Base-2 logarithm, `log₂(self)`.
    ///
    /// Default routes through `f32` via `libm`; `f64` overrides with the
    /// native double-precision path.
    #[inline]
    fn log2(self) -> Self {
        Self::from_f32(libm::log2f(self.to_f32()))
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
