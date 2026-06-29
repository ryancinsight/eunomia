#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Atlas GPU datatype vocabulary.
//!
//! Device-buffer-friendly vector types over the [`eunomia`] scalar SSOT. These
//! are the element types GPU compute backends (hephaestus, coeus) place in
//! storage/vertex buffers; the scalar element types themselves (`F16`, `Bf16`,
//! `F32`, `Complex`, …) live in [`eunomia`] and are re-exported here as the
//! single GPU datatype surface.
//!
//! ## Layout contract
//!
//! [`Vector<T, N>`] is `#[repr(C)]` over `[T; N]`: size `N * size_of::<T>()`,
//! alignment `align_of::<T>()`. This is the **std430 / tightly-packed storage**
//! layout — correct for SSBO arrays, vertex attributes, and host↔device copies.
//! std140 uniform-buffer rules (vec3/vec4 padded to 16 bytes, array stride
//! rounding) are a *buffer-packing* concern enforced at the upload boundary, not
//! a property of the element type — keeping the vocabulary type honest and
//! `bytemuck::Pod`.

mod ops;
mod vector;

// Re-export the GPU element scalars from the eunomia SSOT so downstream code has
// one datatype surface to import.
pub use eunomia::{Bf16, Complex, F16, F32, F64};
pub use vector::{Vec2, Vec3, Vec4, Vector};
