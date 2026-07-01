//! `numpy::Element` for [`Complex`] (feature `numpy`), making
//! [`Complex32`](super::Complex32)/[`Complex64`](super::Complex64) usable as
//! the element type of `rust-numpy` arrays directly — no `num_complex` at the
//! Python boundary.
//!
//! The implementation is sound because `Complex<T>` is `#[repr(C)] { re, im }`,
//! bit-identical in layout and alignment to `num_complex::Complex<T>` (which
//! `numpy` re-exports as `numpy::Complex32`/`numpy::Complex64`). The NumPy dtype
//! (`complex64`/`complex128`) is therefore the same descriptor; we delegate to
//! the canonical `num_complex` impl to obtain it rather than reconstructing it.

use super::Complex;
use numpy::{Element, PyArrayDescr};
use pyo3::{Bound, Python};

// SAFETY: `Complex<f64>` is `#[repr(C)] { re: f64, im: f64 }`, identical in size,
// alignment, and field order to `numpy::Complex64` (`num_complex::Complex<f64>`),
// which maps to the NumPy `complex128` dtype. A `complex128` buffer is thus a
// valid `[Complex<f64>]` and vice versa, satisfying the `Element` contract.
unsafe impl Element for Complex<f64> {
    const IS_COPY: bool = true;

    #[inline]
    fn get_dtype(py: Python<'_>) -> Bound<'_, PyArrayDescr> {
        <numpy::Complex64 as Element>::get_dtype(py)
    }

    #[inline]
    fn clone_ref(&self, _py: Python<'_>) -> Self {
        *self
    }
}

// SAFETY: `Complex<f32>` is `#[repr(C)] { re: f32, im: f32 }`, identical in size,
// alignment, and field order to `numpy::Complex32` (`num_complex::Complex<f32>`),
// which maps to the NumPy `complex64` dtype. The reinterpret is valid in both
// directions, satisfying the `Element` contract.
unsafe impl Element for Complex<f32> {
    const IS_COPY: bool = true;

    #[inline]
    fn get_dtype(py: Python<'_>) -> Bound<'_, PyArrayDescr> {
        <numpy::Complex32 as Element>::get_dtype(py)
    }

    #[inline]
    fn clone_ref(&self, _py: Python<'_>) -> Self {
        *self
    }
}
