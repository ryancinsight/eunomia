//! Unpacking functions for low-precision data representations.

mod arch;

/// Raw, ISA-specific unpack kernels.
///
/// Each submodule exposes the same six operations as the safe dispatch layer,
/// but unconditionally and without a runtime feature probe, so a caller that
/// already runs inside a matching `#[target_feature]` scope can invoke the
/// kernel directly and keep the converted lanes in registers. Every function
/// is `unsafe`: the caller must have established the required ISA is available.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[path = "intrinsics/mod.rs"]
pub mod unsafe_intrinsics;

mod dispatch;
pub use dispatch::*;
