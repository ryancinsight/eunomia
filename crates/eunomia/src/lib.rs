#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Foundational numeric types, traits, and mixed-precision helpers for the Atlas ecosystem.
//! Single Source of Truth (SSOT) for all numeric representations.

extern crate alloc;

mod casts;
pub mod convert;
mod impls;
pub mod layout;
mod ops;
mod packed;
pub mod relative_eq;
mod traits;
mod types;

// Re-export core traits
pub use traits::{CastFrom, CastTo, ComplexField, FloatElement, NumericElement, RealField};

// Re-export the relative-equality trait at the crate root so the macro can
// resolve `eunomia::RelativeEq` and users can opt into the trait if needed.
pub use relative_eq::RelativeEq;

// Re-export byte-layout markers (the reinterpretation fns live under `layout`)
pub use layout::{Pod, Zeroable};

// Re-export wrapper types
pub use types::{
    Bf16, Bf4, Bf8, Complex, Complex32, Complex64, F16, F32, F4, F64, F8, I16, I32, I8,
};

// Re-export packed layout structures and functions
pub use packed::{
    unpack_bf4_to_bf16, unpack_bf4_to_bf16_packed, unpack_bf8_to_bf16, unpack_f4_to_f32,
    unpack_f4_to_f32_packed, unpack_f8_to_f32, Packable4, Packed4Cow, Packed4Iter, Packed4Slice,
    Packed4SliceMut, Packed4Vec, PackedBf4Cow, PackedBf4Slice, PackedBf4SliceMut, PackedBf4Vec,
    PackedF4Cow, PackedF4Slice, PackedF4SliceMut, PackedF4Vec,
};

#[cfg(feature = "rkyv")]
pub use packed::{ArchivedPacked4Cow, ArchivedPacked4Vec, Packed4CowResolver, Packed4VecResolver};

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub use packed::unsafe_intrinsics;
