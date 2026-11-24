//! The [`RealElement`] marker — the element types that are not complex.

use super::NumericElement;

/// A [`NumericElement`] whose values are real numbers, never pairs of them.
///
/// [`Complex<T>`](crate::Complex) is a `NumericElement` only when `T:
/// RealElement`. Its `min_scalar`, `max_scalar` and `sqrt` order and
/// sign-test the components, and a `Complex` component has no order
/// (`PartialOrd` yields `None` for every pair), so a nested complex would
/// return the second argument from every fold and a wrong square root.
///
/// Every shipped element type except `Complex` carries the marker: the
/// primitive floats and integers, `F16`, `Bf16`, `F32`, `F64`, `Bf8`, `Bf4`,
/// `F8`, `F4`, `I8`, `I16` and `I32`. The unsigned primitives carry it
/// because they are real, although `Complex<u8>` and its siblings are still
/// excluded by the `Neg` bound of the `Complex` impl, so the marker does not
/// change which component types are admitted.
///
/// The trait is sealed through the `private::Sealed` supertrait of
/// [`NumericElement`]; downstream crates cannot implement it.
///
/// # Examples
///
/// ```
/// use eunomia::{Complex, NumericElement, RealElement};
///
/// fn real_part_identity<T: RealElement>() -> T {
///     T::MIN_VALUE
/// }
///
/// fn complex_identity<T: RealElement + core::ops::Neg<Output = T>>() -> Complex<T> {
///     <Complex<T> as NumericElement>::MIN_VALUE
/// }
///
/// assert_eq!(real_part_identity::<f32>(), f32::NEG_INFINITY);
/// assert_eq!(complex_identity::<i8>(), Complex::new(i8::MIN, i8::MIN));
/// ```
pub trait RealElement: NumericElement {}
