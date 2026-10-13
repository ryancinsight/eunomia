//! Monomorphized arithmetic for reduced binary floating-point formats.

mod add_sub;
mod divide;
mod format;
mod multiply;
mod remainder;
mod unary;

pub(super) use add_sub::{add, sub};
pub(super) use divide::div;
pub(super) use multiply::mul;
pub(super) use remainder::rem;
pub(super) use unary::neg;
