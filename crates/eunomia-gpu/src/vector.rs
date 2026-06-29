//! The [`Vector`] device-buffer element type and its dimension aliases.

use bytemuck::{Pod, Zeroable};
use core::ops::{Index, IndexMut};

/// A fixed-width vector of `N` components of scalar `T`.
///
/// `#[repr(C)]` over `[T; N]` — see the [crate] docs for the layout contract.
/// Generic over both component type and width (const generic), so one
/// definition serves every GPU vector arity; [`Vec2`]/[`Vec3`]/[`Vec4`] are
/// dimension aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Vector<T, const N: usize> {
    /// Component array.
    pub data: [T; N],
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

impl<T: Default + Copy, const N: usize> Default for Vector<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::splat(T::default())
    }
}

// SAFETY: `Vector<T, N>` is `#[repr(C)]` wrapping `[T; N]`, so it is zeroable and
// plain-old-data exactly when `T` is (`[T; N]: Pod` for `T: Pod`).
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
    fn pod_roundtrip() {
        let v = Vec2::new([3.0_f32, -1.5]);
        let bytes: &[u8] = bytemuck::bytes_of(&v);
        assert_eq!(bytes.len(), 8);
        let back: Vec2<f32> = *bytemuck::from_bytes(bytes);
        assert_eq!(back, v);
    }
}
