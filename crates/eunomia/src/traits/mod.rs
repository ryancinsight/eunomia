//! Element trait surface: numeric, float, field, count, and unit capabilities.
//!
//! One trait family per leaf module ([`numeric`], [`float`], [`count`]); the
//! `private::Sealed` supertrait stays here so `crate::traits::private` remains
//! the single sealing point for the whole crate.

//! ## Accumulator widening
//!
//! `FloatElement::Accumulator` is the crate's **only** sanctioned widening
//! route. Any other way of computing in a type other than `T` inside a
//! `T`-generic body is a fake generic -- cast `T` to something concrete, work
//! there, cast back -- so the widening is declared in the contract instead:
//! per type, where it carries a justification and where a caller can see it
//! in the signature.
//! ### Numerical analysis
//!
//! The worst-case relative error of a sum of `n` values is `(n - 1)*e` for
//! sequential accumulation and `ceil(log2 n)*e` for pairwise (tree)
//! accumulation, where `e` is the **accumulator's** machine epsilon -- not
//! the element type's. Each choice below follows from where `n*e` turns
//! destructive for that format.
//!
//! The type a reduction over `Self` accumulates in.
//!
//! This is the crate's **only** sanctioned widening route. Any other way of
//! computing in a type other than `T` inside a `T`-generic body is a fake
//! generic — cast `T` to something concrete, work there, cast back — so the
//! widening is declared here instead: per type, in the contract, where it
//! carries a justification and where the caller can see it in the signature.
//!
//! # Numerical analysis
//!
//! The worst-case relative error of a sum of `n` values is `(n − 1)·ε` for
//! sequential accumulation and `⌈log₂ n⌉·ε` for pairwise (tree)
//! accumulation, where `ε` is the **accumulator's** machine epsilon — not
//! the element type's. Each choice below follows from where `n·ε` turns
//! destructive for that format:
//!
//! - **Identity for `f32`, `f64`, [`F32`](crate::F32), [`F64`](crate::F64).**
//!   `ε₃₂ = 2⁻²³ ≈ 1.2e-7` and `ε₆₄ = 2⁻⁵² ≈ 2.2e-16` leave headroom at
//!   realistic `n`, and widening `f32` to `f64` is a cost rather than a
//!   correctness fix: it doubles traffic on a reduction that is usually
//!   bandwidth-bound and halves the SIMD lane count. A caller who wants
//!   double-precision accumulation over single-precision data expresses that
//!   by instantiating the algorithm at `f64`, which keeps the choice visible.
//!
//! - **`f32` for [`F16`](crate::F16), [`Bf16`](crate::Bf16) and the sub-byte
//!   formats.** These are storage and bandwidth formats, not accumulation
//!   formats. The binding limit is not accuracy but *stagnation*: a running
//!   sum stops advancing once it exceeds `1/ε` times the next addend, since
//!   the addend then rounds away entirely. For `bf16` (8-bit significand,
//!   `ε ≈ 3.9e-3`) that happens at `n ≈ 256`; for `f16` (11-bit,
//!   `ε ≈ 4.9e-4`) at `n ≈ 2048`; for the 4- and 8-bit formats far sooner.
//!   Summing 10⁵ `bf16` values in `bf16` therefore does not merely lose
//!   digits, it converges to a wrong answer. Widening to `f32` is **exact**
//!   for every one of these types — each has at most an 11-bit significand
//!   and an exponent range inside `f32`'s — so the widening contributes no
//!   error of its own and the reduction inherits `ε₃₂`.
//!
//! Widening these past `f32` buys little further: the inputs were already
//! quantized to ≤11 significand bits, so `f64` would shrink only the
//! summation error, which `f32` already holds well below that quantization.
//!
//! Reductions must still declare their accumulation *order* — pairwise and
//! sequential differ by the factor above, and the bound is only meaningful
//! alongside the order that produced it.
//!
//! [`FloatElement::Accumulator`]: crate::FloatElement::Accumulator

pub(crate) mod private {
    pub trait Sealed {}
}

mod count;
mod field;
mod float;
mod numeric;
mod unit;

pub use count::{CountRangeError, TryFromCount};
pub use field::{ComplexField, RealField};
pub use float::FloatElement;
pub use numeric::NumericElement;
pub use unit::UnitScalar;
