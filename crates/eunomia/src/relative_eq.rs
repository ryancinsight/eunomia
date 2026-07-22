//! Relative-equality assertion helpers for the Atlas numeric SSOT.
//!
//! These helpers mirror the common `approx::assert_relative_eq!` API while
//! remaining `no_std`-friendly.
//!
//! Combined `epsilon`/`max_relative` checks use OR semantics matching `approx`:
//! a value passes if it is within `epsilon` **or** within `max_relative * scale`.

use core::fmt;

/// Types that can be compared with a blended absolute/relative tolerance.
///
/// The macro [`assert_relative_eq!`](macro@crate::assert_relative_eq) is the
/// primary consumer of this trait; callers generally do not need to implement
/// it themselves unless they are introducing a new numeric scalar type into
/// the Atlas ecosystem.
pub trait RelativeEq {
    /// Native tolerance type for this value.
    type Epsilon: Copy + fmt::Debug;

    /// Returns the absolute tolerance to use when the caller does not specify
    /// an `epsilon`.
    fn default_epsilon() -> Self::Epsilon;

    /// Returns the relative tolerance to use when the caller does not specify
    /// a `max_relative`.
    fn default_max_relative() -> Self::Epsilon;

    /// Returns `true` if `self` and `other` are within the given tolerances.
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool;
}

impl RelativeEq for f32 {
    type Epsilon = Self;

    fn default_epsilon() -> Self::Epsilon {
        Self::EPSILON
    }

    fn default_max_relative() -> Self::Epsilon {
        Self::EPSILON
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if self == other {
            return true;
        }
        if self.is_infinite() || other.is_infinite() {
            return false;
        }
        let diff = (*self - *other).abs();
        if diff <= epsilon {
            return true;
        }
        let scale = self.abs().max(other.abs());
        diff <= max_relative * scale
    }
}

impl RelativeEq for f64 {
    type Epsilon = Self;

    fn default_epsilon() -> Self::Epsilon {
        Self::EPSILON
    }

    fn default_max_relative() -> Self::Epsilon {
        Self::EPSILON
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if self == other {
            return true;
        }
        if self.is_infinite() || other.is_infinite() {
            return false;
        }
        let diff = (*self - *other).abs();
        if diff <= epsilon {
            return true;
        }
        let scale = self.abs().max(other.abs());
        diff <= max_relative * scale
    }
}

impl<T: RelativeEq, const N: usize> RelativeEq for [T; N] {
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.iter()
            .zip(other.iter())
            .all(|(a, b)| a.relative_eq(b, epsilon, max_relative))
    }
}

impl<T: RelativeEq> RelativeEq for [T] {
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter()
            .zip(other.iter())
            .all(|(a, b)| a.relative_eq(b, epsilon, max_relative))
    }
}

/// Blanket reference impl so `assert_relative_eq!(xi, di)` works when the
/// caller already holds `&f64` (e.g. from `iter().zip()`). The macro wraps each
/// operand in `&($expr)`, producing `&&T` when `$expr: &T`; this impl
/// dereferences one layer so the underlying `T: RelativeEq` is consulted.
///
/// The comparison delegates through a fully qualified `<T as RelativeEq>::`
/// call to avoid recursion through this same `&T` impl.
impl<T: RelativeEq + ?Sized> RelativeEq for &T {
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        <T as RelativeEq>::relative_eq(*self, *other, epsilon, max_relative)
    }
}

/// Complex-number relative equality. Two `Complex<T>` are equal iff both the
/// real and imaginary parts satisfy `T: RelativeEq` with the same tolerances.
/// This mirrors `approx`'s `RelativeEq for Complex64` behaviour and lets the
/// Atlas transform tests (Apollo, Kwavers) drop `approx` entirely.
impl<T: RelativeEq> RelativeEq for crate::Complex<T> {
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.re.relative_eq(&other.re, epsilon, max_relative)
            && self.im.relative_eq(&other.im, epsilon, max_relative)
    }
}

/// Panic helper used by [`assert_relative_eq!`](macro@crate::assert_relative_eq).
#[doc(hidden)]
#[cold]
#[inline(never)]
pub fn __relative_eq_panic<T: fmt::Debug, E: fmt::Debug>(
    left: &T,
    right: &T,
    epsilon: E,
    max_relative: E,
) -> ! {
    panic!(
        "assert_relative_eq failed\n  left: {:?}\n right: {:?}\n epsilon: {:?}\n max_relative: {:?}",
        left, right, epsilon, max_relative
    )
}

/// Resolves a value's native default absolute tolerance for exported macros.
#[doc(hidden)]
pub fn __default_epsilon<T: RelativeEq + ?Sized>(_: &T) -> T::Epsilon {
    T::default_epsilon()
}

/// Resolves a value's native default relative tolerance for exported macros.
#[doc(hidden)]
pub fn __default_max_relative<T: RelativeEq + ?Sized>(_: &T) -> T::Epsilon {
    T::default_max_relative()
}

/// Asserts that two numeric values are approximately equal.
///
/// This is a drop-in replacement for `approx::assert_relative_eq!` that is
/// implemented entirely within the Atlas `eunomia` crate and works in
/// `no_std` environments.
///
/// # Supported call patterns
///
/// ```rust
/// use eunomia::assert_relative_eq;
///
/// assert_relative_eq!(1.0_f64, 1.0000000000000001);
/// assert_relative_eq!(1.0_f64, 1.1, epsilon = 0.2);
/// assert_relative_eq!(1.0_f64, 1.1, max_relative = 0.2);
/// assert_relative_eq!(1.0_f64, 1.00000000001, epsilon = 1e-10, max_relative = 1e-10);
/// assert_relative_eq!(1.0_f64, 1.00000000001, max_relative = 1e-10, epsilon = 1e-10);
/// ```
#[macro_export]
macro_rules! assert_relative_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $crate::relative_eq::__default_epsilon(left);
        let max_relative = $crate::relative_eq::__default_max_relative(left);
        if !$crate::RelativeEq::relative_eq(left, right, epsilon, max_relative) {
            $crate::relative_eq::__relative_eq_panic(left, right, epsilon, max_relative);
        }
    }};

    ($left:expr, $right:expr, epsilon = $epsilon:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $epsilon;
        let max_relative = $crate::relative_eq::__default_max_relative(left);
        if !$crate::RelativeEq::relative_eq(left, right, epsilon, max_relative) {
            $crate::relative_eq::__relative_eq_panic(left, right, epsilon, max_relative);
        }
    }};

    ($left:expr, $right:expr, max_relative = $max_relative:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $crate::relative_eq::__default_epsilon(left);
        let max_relative = $max_relative;
        if !$crate::RelativeEq::relative_eq(left, right, epsilon, max_relative) {
            $crate::relative_eq::__relative_eq_panic(left, right, epsilon, max_relative);
        }
    }};

    ($left:expr, $right:expr, epsilon = $epsilon:expr, max_relative = $max_relative:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $epsilon;
        let max_relative = $max_relative;
        if !$crate::RelativeEq::relative_eq(left, right, epsilon, max_relative) {
            $crate::relative_eq::__relative_eq_panic(left, right, epsilon, max_relative);
        }
    }};

    ($left:expr, $right:expr, max_relative = $max_relative:expr, epsilon = $epsilon:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $epsilon;
        let max_relative = $max_relative;
        if !$crate::RelativeEq::relative_eq(left, right, epsilon, max_relative) {
            $crate::relative_eq::__relative_eq_panic(left, right, epsilon, max_relative);
        }
    }};
}

/// Asserts that two numeric values are within an absolute tolerance of each
/// other.
///
/// This is a drop-in replacement for `approx::assert_abs_diff_eq!` that is
/// implemented entirely within the Atlas `eunomia` crate.
///
/// # Supported call patterns
///
/// ```rust
/// use eunomia::assert_abs_diff_eq;
///
/// assert_abs_diff_eq!(1.0_f64, 1.0000000000000001);
/// assert_abs_diff_eq!(1.0_f64, 1.1, epsilon = 0.2);
/// ```
#[macro_export]
macro_rules! assert_abs_diff_eq {
    ($left:expr, $right:expr $(,)?) => {{
        $crate::assert_relative_eq!($left, $right, max_relative = 0.0)
    }};

    ($left:expr, $right:expr, epsilon = $epsilon:expr $(,)?) => {{
        $crate::assert_relative_eq!($left, $right, epsilon = $epsilon, max_relative = 0.0)
    }};
}

/// Boolean relative-equality check (drop-in for `approx::relative_eq!`).
///
/// Returns `true` if `left` and `right` are within tolerance, `false` otherwise.
/// Unlike [`assert_relative_eq!`](macro@crate::assert_relative_eq), this macro
/// does **not** panic — callers use it inside
/// `assert!(relative_eq!(...), "custom message")` when they need a context-rich
/// failure message, matching the `approx::relative_eq!` API.
///
/// # Supported call patterns
///
/// ```rust
/// use eunomia::relative_eq;
///
/// assert!(relative_eq!(1.0_f64, 1.0000000000000001));
/// assert!(relative_eq!(1.0_f64, 1.1, epsilon = 0.2));
/// assert!(relative_eq!(1.0_f64, 1.1, max_relative = 0.2));
/// assert!(relative_eq!(1.0_f64, 1.00000000001, epsilon = 1e-10, max_relative = 1e-10));
/// ```
#[macro_export]
macro_rules! relative_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $crate::relative_eq::__default_epsilon(left);
        let max_relative = $crate::relative_eq::__default_max_relative(left);
        $crate::RelativeEq::relative_eq(left, right, epsilon, max_relative)
    }};

    ($left:expr, $right:expr, epsilon = $epsilon:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $epsilon;
        let max_relative = $crate::relative_eq::__default_max_relative(left);
        $crate::RelativeEq::relative_eq(left, right, epsilon, max_relative)
    }};

    ($left:expr, $right:expr, max_relative = $max_relative:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $crate::relative_eq::__default_epsilon(left);
        let max_relative = $max_relative;
        $crate::RelativeEq::relative_eq(left, right, epsilon, max_relative)
    }};

    ($left:expr, $right:expr, epsilon = $epsilon:expr, max_relative = $max_relative:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $epsilon;
        let max_relative = $max_relative;
        $crate::RelativeEq::relative_eq(left, right, epsilon, max_relative)
    }};

    ($left:expr, $right:expr, max_relative = $max_relative:expr, epsilon = $epsilon:expr $(,)?) => {{
        let left = &($left);
        let right = &($right);
        let epsilon = $epsilon;
        let max_relative = $max_relative;
        $crate::RelativeEq::relative_eq(left, right, epsilon, max_relative)
    }};
}

/// Boolean absolute-difference equality check (drop-in for
/// `approx::abs_diff_eq!`).
///
/// Returns `true` if `|left - right| <= epsilon`, `false` otherwise. Unlike
/// [`assert_abs_diff_eq!`](macro@crate::assert_abs_diff_eq), this macro does
/// **not** panic — callers use it inside
/// `assert!(abs_diff_eq!(...), "custom message")`.
///
/// # Supported call patterns
///
/// ```rust
/// use eunomia::abs_diff_eq;
///
/// assert!(abs_diff_eq!(1.0_f64, 1.0000000000000001));
/// assert!(abs_diff_eq!(1.0_f64, 1.1, epsilon = 0.2));
/// ```
#[macro_export]
macro_rules! abs_diff_eq {
    ($left:expr, $right:expr $(,)?) => {{
        $crate::relative_eq!($left, $right, max_relative = 0.0)
    }};

    ($left:expr, $right:expr, epsilon = $epsilon:expr $(,)?) => {{
        $crate::relative_eq!($left, $right, epsilon = $epsilon, max_relative = 0.0)
    }};
}

#[cfg(test)]
mod tests;
