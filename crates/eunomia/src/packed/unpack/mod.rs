//! Unpacking functions for low-precision data representations.

#![allow(clippy::missing_safety_doc)]

mod arch;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[allow(missing_docs)]
#[path = "intrinsics/mod.rs"]
pub mod unsafe_intrinsics;

mod dispatch;
pub use dispatch::*;
