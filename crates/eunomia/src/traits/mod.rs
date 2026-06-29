//! Element trait surface: numeric/float capabilities and cast helpers.
//!
//! One trait family per leaf module ([`numeric`], [`float`], [`cast`]); the
//! `private::Sealed` supertrait stays here so `crate::traits::private` remains
//! the single sealing point for the whole crate.

pub(crate) mod private {
    pub trait Sealed {}
}

mod cast;
mod field;
mod float;
mod numeric;

pub use cast::{CastFrom, CastTo};
pub use field::{ComplexField, RealField};
pub use float::FloatElement;
pub use numeric::NumericElement;
