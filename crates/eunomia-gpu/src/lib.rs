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

use bytemuck::{Pod, Zeroable};
use core::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

// Re-export the GPU element scalars from the eunomia SSOT so downstream code has
// one datatype surface to import.
pub use eunomia::{Bf16, Complex, F16, F32, F64};

/// A fixed-width vector of `N` components of scalar `T`.
///
/// `#[repr(C)]` over `[T; N]` — see the [module docs](crate) for the layout
/// contract. Generic over both component type and width (const generic), so one
/// definition serves every GPU vector arity; [`Vec2`]/[`Vec3`]/[`Vec4`] are
/// dimension aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Vector<T, const N: usize> {
    /// Component array.
    pub data: [T; N],
}

impl<T: Default + Copy, const N: usize> Default for Vector<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::splat(T::default())
    }
}

/// 2-component vector (e.g. UV coordinates, complex pairs).
pub type Vec2<T> = Vector<T, 2>;
/// 3-component vector (e.g. positions, RGB).
pub type Vec3<T> = Vector<T, 3>;
/// 4-component vector (e.g. homogeneous coordinates, RGBA).
pub type Vec4<T> = Vector<T, 4>;

impl<T, const N: usize> Vector<T, N> {
    /// Construct from a component array.
    #[inline(always)]
    pub const fn new(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<T: Copy, const N: usize> Vector<T, N> {
    /// Construct a vector with every component equal to `value`.
    #[inline(always)]
    pub const fn splat(value: T) -> Self {
        Self { data: [value; N] }
    }
}

// SAFETY: `Vector<T, N>` is `#[repr(C)]` wrapping `[T; N]`, so it is zeroable and
// plain-old-data exactly when `T` is. `[T; N]: Pod` for `T: Pod`.
unsafe impl<T: Zeroable, const N: usize> Zeroable for Vector<T, N> {}
unsafe impl<T: Pod, const N: usize> Pod for Vector<T, N> {}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    #[inline(always)]
    fn from(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<T, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;
    #[inline(always)]
    fn index(&self, i: usize) -> &T {
        &self.data[i]
    }
}

impl<T, const N: usize> IndexMut<usize> for Vector<T, N> {
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.data[i]
    }
}

impl<T: Add<Output = T> + Copy, const N: usize> Add for Vector<T, N> {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        let mut out = self;
        let mut i = 0;
        while i < N {
            out.data[i] = self.data[i] + rhs.data[i];
            i += 1;
        }
        out
    }
}

impl<T: Sub<Output = T> + Copy, const N: usize> Sub for Vector<T, N> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        let mut out = self;
        let mut i = 0;
        while i < N {
            out.data[i] = self.data[i] - rhs.data[i];
            i += 1;
        }
        out
    }
}

impl<T: Neg<Output = T> + Copy, const N: usize> Neg for Vector<T, N> {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        let mut out = self;
        let mut i = 0;
        while i < N {
            out.data[i] = -self.data[i];
            i += 1;
        }
        out
    }
}

impl<T: Mul<Output = T> + Copy, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Self;
    /// Scalar multiplication (`vector * scalar`).
    #[inline(always)]
    fn mul(self, scalar: T) -> Self {
        let mut out = self;
        let mut i = 0;
        while i < N {
            out.data[i] = self.data[i] * scalar;
            i += 1;
        }
        out
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Copy, const N: usize> Vector<T, N> {
    /// Euclidean dot product `Σ aᵢ·bᵢ`.
    #[inline]
    pub fn dot(self, rhs: Self) -> T {
        let mut acc = self.data[0] * rhs.data[0];
        let mut i = 1;
        while i < N {
            acc = acc + self.data[i] * rhs.data[i];
            i += 1;
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_tightly_packed() {
        assert_eq!(core::mem::size_of::<Vec4<f32>>(), 16);
        assert_eq!(core::mem::align_of::<Vec4<f32>>(), 4);
        assert_eq!(core::mem::size_of::<Vec2<f32>>(), 8);
        assert_eq!(core::mem::size_of::<Vec3<f32>>(), 12);
    }

    #[test]
    fn arithmetic_and_dot() {
        let a = Vec4::new([1.0_f32, 2.0, 3.0, 4.0]);
        let b = Vec4::splat(2.0);
        assert_eq!((a + b).data, [3.0, 4.0, 5.0, 6.0]);
        assert_eq!((a - b).data, [-1.0, 0.0, 1.0, 2.0]);
        assert_eq!((a * 2.0).data, [2.0, 4.0, 6.0, 8.0]);
        assert_eq!((-a).data, [-1.0, -2.0, -3.0, -4.0]);
        // 1*2 + 2*2 + 3*2 + 4*2 = 20
        assert_eq!(a.dot(b), 20.0);
    }

    #[test]
    fn pod_roundtrip() {
        let v = Vec2::new([3.0_f32, -1.5]);
        let bytes: &[u8] = bytemuck::bytes_of(&v);
        assert_eq!(bytes.len(), 8);
        let back: Vec2<f32> = *bytemuck::from_bytes(bytes);
        assert_eq!(back, v);
    }
}
