//! Bulk `binary16`/`bfloat16` ↔ `f32` slice conversion — the vectorized
//! companion to the scalar [`widen`](super::widen)/[`narrow`](super::narrow)
//! kernel.
//!
//! `binary16` uses F16C on x86-64 (one `vcvtph2ps`/`vcvtps2ph` per eight lanes),
//! falling back to the scalar kernel elsewhere. `bfloat16` is the high 16 bits
//! of an `f32`, so its widen is a shift and its narrow a round-and-truncate —
//! both branch-light enough that the plain loops autovectorize, no intrinsics
//! needed. Every path rounds `f32`→reduced to nearest, ties to even, and is
//! verified bit-for-bit against the kernel and the `half` reference.

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

/// Widen a slice of `bfloat16` bit patterns into `f32`. `bfloat16` is the high
/// 16 bits of the `f32`, so widening is a left shift — exact for every value
/// (normals, subnormals, infinity, NaN) and the loop autovectorizes to a plain
/// unpack/shift, no F16C needed.
pub(crate) fn widen_bf16(src: &[u16], dst: &mut [f32]) {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = f32::from_bits(u32::from(src[i]) << 16);
    }
}

/// Narrow a slice of `f32` into `bfloat16` bit patterns (round to nearest, ties
/// to even). Branchless per element so the loop autovectorizes.
pub(crate) fn narrow_bf16(src: &[f32], dst: &mut [u16]) {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = round_f32_to_bf16(src[i].to_bits());
    }
}

/// Round an `f32` bit pattern to a `bfloat16` bit pattern, ties to even —
/// bit-identical to `narrow::<8, 7>`.
#[inline]
fn round_f32_to_bf16(bits: u32) -> u16 {
    // Round to nearest, ties to even: add `0x7FFF` plus the retained LSB, then
    // truncate. `wrapping_add` avoids a debug overflow panic on the (discarded)
    // NaN path; no wrap occurs for finite/infinite inputs.
    let rounded = bits.wrapping_add(0x7FFF).wrapping_add((bits >> 16) & 1) >> 16;
    // A NaN must keep a nonzero mantissa (else it collapses to infinity), forcing
    // the low mantissa bit when the retained payload is empty — the kernel's rule.
    let high = bits >> 16;
    let nan = high | u32::from(high & 0x7F == 0);
    let is_nan = (bits & 0x7F80_0000 == 0x7F80_0000) & (bits & 0x007F_FFFF != 0);
    (if is_nan { nan } else { rounded }) as u16
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
