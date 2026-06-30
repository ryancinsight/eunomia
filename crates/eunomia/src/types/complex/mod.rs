//! Complex number — the SSOT vocabulary type for `re + im·i`, replacing the
//! third-party `num_complex::Complex` across the Atlas stack
//! (leto, hephaestus, coeus).
//!
//! Layout-compatible (`#[repr(C)]` `{ re, im }`) and `bytemuck`-friendly, so
//! values round-trip through GPU device buffers and FFI boundaries identically
//! to `num_complex::Complex`. (The `Pod`/`Zeroable` impls live in the parent
//! `types` module alongside the other scalar types.)
//!
//! Operators live in [`ops`]; the floating-point surface in [`float`].

mod consts;
mod float;
mod ops;

/// A complex number `re + im·i`.
///
/// A minimal `#[repr(C)]` pair of real/imaginary components carrying the
/// arithmetic and trait surface the numeric stack relies on. Arithmetic is
/// defined field-wise (`Add`/`Sub`/`Neg`) and by the complex product/quotient
/// (`Mul`/`Div`), generic over the component scalar `T`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Complex<T> {
    /// Real part.
    pub re: T,
    /// Imaginary part.
    pub im: T,
}

/// A single-precision complex number, `Complex<f32>` (mirrors
/// `num_complex::Complex32`).
pub type Complex32 = Complex<f32>;

/// A double-precision complex number, `Complex<f64>` (mirrors
/// `num_complex::Complex64`).
pub type Complex64 = Complex<f64>;

impl<T> Complex<T> {
    /// Construct from real and imaginary parts.
    #[inline(always)]
    pub const fn new(re: T, im: T) -> Self {
        Self { re, im }
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Complex<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}+{}i", self.re, self.im)
    }
}
