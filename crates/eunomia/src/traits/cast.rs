//! Generic scalar [`CastFrom`]/[`CastTo`] conversion helpers.

/// Helper trait for generic casting between SIMD scalar types.
pub trait CastFrom<T>: Copy {
    /// Cast from type `T` to `Self`.
    fn cast_from(val: T) -> Self;
}

/// Helper trait for generic casting to another SIMD scalar type.
pub trait CastTo: Copy {
    /// Cast `self` to type `U`.
    #[inline(always)]
    fn cast_to<U>(self) -> U
    where
        U: CastFrom<Self>,
    {
        U::cast_from(self)
    }
}

impl<T: Copy> CastTo for T {}
