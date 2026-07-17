//! Generic scalar [`CastFrom`]/[`CastTo`] conversion helpers.

/// Helper trait for generic casting between SIMD scalar types.
///
/// Primitive numeric implementations follow Rust's `as` conversion semantics.
/// In particular, float-to-integer conversions truncate toward zero and
/// saturate values outside the destination range.
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
