//! Element trait surface: numeric, float, field, count, and unit capabilities.
//!
//! One trait family per leaf module ([`numeric`], [`float`], [`count`]); the
//! `private::Sealed` supertrait stays here so `crate::traits::private` remains
//! the single sealing point for the whole crate.

pub(crate) mod private {
    pub trait Sealed {}
}

mod count;
mod field;
mod float;
mod numeric;
mod unit;

pub use count::{CountRangeError, TryFromCount};
pub use field::{ComplexField, RealField};
pub use float::FloatElement;
pub use numeric::NumericElement;
pub use unit::UnitScalar;
