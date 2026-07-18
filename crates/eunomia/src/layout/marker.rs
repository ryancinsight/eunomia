//! Byte-layout marker traits — eunomia's native `Zeroable`/`Pod` vocabulary.
//!
//! These are the datatype-law statement of "which representations are safe to
//! reinterpret as bytes", owned here rather than borrowed from `bytemuck`. They
//! are `unsafe` marker traits (the compiler cannot verify the layout facts they
//! assert); every impl carries a `// SAFETY:` justification, and the scalar
//! wrappers' `const _` size/alignment assertions ([`crate::types`]) pin the
//! layout the impls rely on. The `bytemuck` feature bridges these to
//! `bytemuck::{Pod, Zeroable}` for GPU/FFI boundaries that fix that contract.

use crate::types::{Bf16, Bf4, Bf8, Complex, F16, F32, F4, F64, F8, I16, I32, I8};

/// A type whose all-zero bit pattern is a valid, inhabited value.
///
/// # Safety
/// The all-zeroes bit pattern must be a valid value of `Self`. This excludes
/// types with a validity niche at zero (e.g. `NonZeroU32`, `&T`).
pub unsafe trait Zeroable: Sized {
    /// Returns a value of `Self` with every byte set to zero.
    #[inline]
    #[must_use]
    fn zeroed() -> Self {
        // SAFETY: `Self: Zeroable` guarantees the all-zero pattern is valid.
        unsafe { core::mem::zeroed() }
    }
}

/// Plain-old-data: any bit pattern of `size_of::<Self>()` bytes is a valid
/// `Self`, and the type carries no padding or invalid representations, so it can
/// be freely reinterpreted to and from bytes.
///
/// # Safety
/// - `Self` is inhabited and `Copy`.
/// - Every bit pattern of `size_of::<Self>()` bytes is a valid `Self`.
/// - `Self` contains no padding bytes, no interior mutability, and no pointers
///   or references.
pub unsafe trait Pod: Zeroable + Copy + 'static {}

macro_rules! impl_pod {
    ($($t:ty),* $(,)?) => {$(
        // SAFETY: a primitive or `#[repr(transparent)]`/`#[repr(C)]` wrapper over
        // one with no invalid bit patterns, no padding, and no interior
        // mutability; all-zeroes is a valid value.
        unsafe impl Zeroable for $t {}
        // SAFETY: see the `Zeroable` impl; additionally `Copy + 'static`.
        unsafe impl Pod for $t {}
    )*};
}

// Primitive scalars. `bool`/`char` are intentionally excluded — they have
// invalid bit patterns and are therefore not `Pod`.
impl_pod!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

// eunomia scalar wrappers (`#[repr(transparent)]`, layout pinned in `types`).
impl_pod!(F16, F32, F64, Bf16, Bf8, Bf4, F8, F4, I8, I16, I32);

// SAFETY: `Complex<T>` is `#[repr(C)]` with two `T` fields, so its all-zero
// pattern is valid, and it is padding-free plain-old-data, exactly when `T` is.
unsafe impl<T: Zeroable> Zeroable for Complex<T> {}
unsafe impl<T: Pod> Pod for Complex<T> {}
