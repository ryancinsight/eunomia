//! Component-wise arithmetic for [`Vector`].
//!
//! Element-wise operators build the result array with [`core::array::from_fn`]
//! — one const-generic expression per operator, no hand-rolled loops — which
//! monomorphizes and inlines to the same code as a fixed-width implementation.

use crate::Vector;
use core::ops::{Add, Mul, Neg, Sub};

impl<T: Add<Output = T> + Copy, const N: usize> Add for Vector<T, N> {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            data: core::array::from_fn(|i| self.data[i] + rhs.data[i]),
        }
    }
}

impl<T: Sub<Output = T> + Copy, const N: usize> Sub for Vector<T, N> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self {
            data: core::array::from_fn(|i| self.data[i] - rhs.data[i]),
        }
    }
}

impl<T: Neg<Output = T> + Copy, const N: usize> Neg for Vector<T, N> {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self {
            data: core::array::from_fn(|i| -self.data[i]),
        }
    }
}

impl<T: Mul<Output = T> + Copy, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Self;
    /// Scalar multiplication (`vector * scalar`).
    #[inline(always)]
    fn mul(self, scalar: T) -> Self {
        Self {
            data: core::array::from_fn(|i| self.data[i] * scalar),
        }
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Copy, const N: usize> Vector<T, N> {
    /// Euclidean dot product `Σ aᵢ·bᵢ`. Requires `N ≥ 1`.
    #[inline]
    pub fn dot(self, rhs: Self) -> T {
        let mut acc = self.data[0] * rhs.data[0];
        for i in 1..N {
            acc = acc + self.data[i] * rhs.data[i];
        }
        acc
    }
}

/// Euclidean geometry over a real scalar field.
impl<T: eunomia::RealField, const N: usize> Vector<T, N> {
    /// Squared Euclidean norm `Σ aᵢ²`.
    #[inline]
    pub fn norm_squared(self) -> T {
        self.dot(self)
    }

    /// Euclidean norm (length) `√(Σ aᵢ²)`.
    #[inline]
    pub fn norm(self) -> T {
        self.norm_squared().sqrt()
    }

    /// Unit vector in the same direction, `self / ‖self‖`.
    ///
    /// Matches `nalgebra`'s `normalize`: division by a zero norm yields a
    /// non-finite result rather than panicking; callers that may pass a zero
    /// vector should guard the norm themselves.
    #[inline]
    pub fn normalize(self) -> Self {
        self * self.norm().recip()
    }
}

/// 3-vector cross product.
impl<T: eunomia::RealField> Vector<T, 3> {
    /// Cross product `self × rhs`.
    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        let [ax, ay, az] = self.data;
        let [bx, by, bz] = rhs.data;
        Self {
            data: [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Vec2, Vec4, Vector};

    #[test]
    fn geometry_norm_normalize_cross() {
        let x = Vector::<f64, 3>::new([1.0, 0.0, 0.0]);
        let y = Vector::<f64, 3>::new([0.0, 1.0, 0.0]);
        // right-handed: x × y = z
        assert_eq!(x.cross(y).data, [0.0, 0.0, 1.0]);
        assert_eq!(x.dot(y), 0.0);
        let v = Vector::<f64, 3>::new([3.0, 4.0, 0.0]);
        assert_eq!(v.norm_squared(), 25.0);
        assert_eq!(v.norm(), 5.0);
        assert!((v.normalize().norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn arithmetic_and_dot() {
        let a = Vec4::new([1.0_f32, 2.0, 3.0, 4.0]);
        let b = Vec4::splat(2.0);
        assert_eq!((a + b).data, [3.0, 4.0, 5.0, 6.0]);
        assert_eq!((a - b).data, [-1.0, 0.0, 1.0, 2.0]);
        assert_eq!((a * 2.0).data, [2.0, 4.0, 6.0, 8.0]);
        assert_eq!((-a).data, [-1.0, -2.0, -3.0, -4.0]);
        // 1·2 + 2·2 + 3·2 + 4·2 = 20
        assert_eq!(a.dot(b), 20.0);
    }

    #[test]
    fn scalar_mul_on_pair() {
        let v = Vec2::new([1.5_f32, -2.0]);
        assert_eq!((v * 2.0).data, [3.0, -4.0]);
    }
}
