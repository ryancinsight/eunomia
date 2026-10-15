//! Arithmetic implementations for the float representations.

mod integer;
mod primitive;
mod reduced;

// The target-format floor kernel is the one family surface the wrapper
// `FloatElement` impls override; escalated for that call site only.
pub(crate) use integer::floor;
