//! Bulk `binary16` ↔ `f32` slice conversion — the hardware-accelerated companion
//! to the scalar [`widen`](super::widen)/[`narrow`](super::narrow) kernel.
//!
//! On x86-64 with F16C the conversion is one instruction per eight lanes
//! (`vcvtph2ps`/`vcvtps2ph`); everywhere else it falls back to the scalar
//! kernel. F16C rounds `f32`→`f16` to nearest, ties to even — identical to the
//! kernel and the `half` reference — so the SIMD path is bit-for-bit equal to
//! the scalar one (verified differentially). Selection follows the same runtime
//! dispatch pattern as `packed::unpack`.

use super::{narrow, widen};

/// Widen a slice of `binary16` bit patterns into `f32`, writing
/// `min(src.len(), dst.len())` elements.
pub(crate) fn widen_f16(src: &[u16], dst: &mut [f32]) {
    #[cfg(target_arch = "x86_64")]
    {
        if has_f16c() {
            // SAFETY: `has_f16c()` confirmed F16C support at runtime; the kernel
            // bounds its reads/writes to the shorter of the two slices.
            unsafe {
                widen_f16_x86(src, dst);
            }
            return;
        }
    }
    widen_f16_scalar(src, dst);
}

/// Narrow a slice of `f32` into `binary16` bit patterns (round to nearest, ties
/// to even), writing `min(src.len(), dst.len())` elements.
pub(crate) fn narrow_f16(src: &[f32], dst: &mut [u16]) {
    #[cfg(target_arch = "x86_64")]
    {
        if has_f16c() {
            // SAFETY: `has_f16c()` confirmed F16C support at runtime; the kernel
            // bounds its reads/writes to the shorter of the two slices.
            unsafe {
                narrow_f16_x86(src, dst);
            }
            return;
        }
    }
    narrow_f16_scalar(src, dst);
}

#[inline]
fn widen_f16_scalar(src: &[u16], dst: &mut [f32]) {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = f32::from_bits(widen::<5, 10>(u32::from(src[i])));
    }
}

#[inline]
fn narrow_f16_scalar(src: &[f32], dst: &mut [u16]) {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = narrow::<5, 10>(src[i].to_bits()) as u16;
    }
}

/// Runtime F16C detection (mirrors `packed::unpack::arch`).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn has_f16c() -> bool {
    #[cfg(feature = "std")]
    {
        std::is_x86_feature_detected!("f16c")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "f16c")
    }
}

/// # Safety
/// The running CPU must support F16C. Handles eight lanes per iteration with a
/// scalar remainder; reads/writes stay within the shorter of the two slices.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "f16c")]
unsafe fn widen_f16_x86(src: &[u16], dst: &mut [f32]) {
    use core::arch::x86_64::{_mm256_cvtph_ps, _mm256_storeu_ps, _mm_loadu_si128};

    let n = src.len().min(dst.len());
    let mut i = 0;
    while i + 8 <= n {
        let packed = _mm_loadu_si128(src.as_ptr().add(i).cast());
        let widened = _mm256_cvtph_ps(packed);
        _mm256_storeu_ps(dst.as_mut_ptr().add(i), widened);
        i += 8;
    }
    while i < n {
        dst[i] = f32::from_bits(widen::<5, 10>(u32::from(src[i])));
        i += 1;
    }
}

/// # Safety
/// The running CPU must support F16C. Rounds to nearest (ties to even) via the
/// immediate; scalar remainder; reads/writes stay within the shorter slice.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "f16c")]
unsafe fn narrow_f16_x86(src: &[f32], dst: &mut [u16]) {
    use core::arch::x86_64::{
        _mm256_cvtps_ph, _mm256_loadu_ps, _mm_storeu_si128, _MM_FROUND_TO_NEAREST_INT,
    };

    let n = src.len().min(dst.len());
    let mut i = 0;
    while i + 8 <= n {
        let values = _mm256_loadu_ps(src.as_ptr().add(i));
        // `_MM_FROUND_TO_NEAREST_INT` (0) rounds to nearest, ties to even —
        // matching the scalar kernel and `half`.
        let narrowed = _mm256_cvtps_ph::<{ _MM_FROUND_TO_NEAREST_INT }>(values);
        _mm_storeu_si128(dst.as_mut_ptr().add(i).cast(), narrowed);
        i += 8;
    }
    while i < n {
        dst[i] = narrow::<5, 10>(src[i].to_bits()) as u16;
        i += 1;
    }
}
