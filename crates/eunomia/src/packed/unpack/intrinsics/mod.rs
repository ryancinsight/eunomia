//! Raw unpack kernels per instruction-set extension.

/// AVX2 kernels (x86-64).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod avx2;
/// AVX-512 kernels (x86-64).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod avx512;

/// NEON kernels (aarch64).
#[cfg(target_arch = "aarch64")]
pub mod neon;
