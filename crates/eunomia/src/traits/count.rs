//! The [`TryFromCount`] trait — checked construction of any numeric element
//! from an element count.

use super::private;

/// Checked construction of a numeric element from a count (a length, a
/// dimension, a position along an axis).
///
/// This is the one conversion every [`NumericElement`](super::NumericElement)
/// accepts from `usize`: integer elements take the count exactly or refuse it,
/// and float elements always accept it, rounding to nearest as
/// [`FloatElement::from_count`](super::FloatElement::from_count) documents.
/// Float-only code calls `from_count` directly and needs no error path.
///
/// # Examples
///
/// ```
/// use eunomia::TryFromCount;
///
/// assert_eq!(i8::try_from_count(127), Ok(127));
/// assert!(i8::try_from_count(128).is_err());
/// assert_eq!(f32::try_from_count(3), Ok(3.0));
/// ```
pub trait TryFromCount: private::Sealed + Sized {
    /// Converts the count `n` into `Self`.
    ///
    /// # Errors
    ///
    /// Returns [`CountRangeError`] when `Self` is an integer element whose
    /// range does not contain `n`. Float elements never fail.
    fn try_from_count(n: usize) -> Result<Self, CountRangeError>;
}

/// A count lies outside the range of the integer element it was converted to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountRangeError {
    count: usize,
    target: &'static str,
}

impl CountRangeError {
    /// The error for the count `count` not fitting the element type `T`.
    #[must_use]
    pub fn new<T>(count: usize) -> Self {
        Self {
            count,
            target: core::any::type_name::<T>(),
        }
    }

    /// The count that did not fit.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// The name of the element type the count did not fit.
    #[must_use]
    pub const fn target(&self) -> &'static str {
        self.target
    }
}

impl core::fmt::Display for CountRangeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "count {} lies outside the range of `{}`",
            self.count, self.target
        )
    }
}

impl core::error::Error for CountRangeError {}
