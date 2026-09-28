//! Bulk `binary16`/`bfloat16` ↔ `f32` slice conversion — the vectorized
//! companion to the scalar [`widen`](super::widen)/[`narrow`](super::narrow)
//! kernel.
//!
//! The conversion is parameterised by the **element type** through the
//! [`WidenToF32`]/[`NarrowFromF32`] traits rather than by a per-format free
//! function, so one trait method serves every reduced format and the format
//! lives in the type, not in the identifier. `binary16` uses F16C on x86-64
//! (one `vcvtph2ps`/`vcvtps2ph` per eight lanes), falling back to the scalar
//! kernel elsewhere. `bfloat16` is the high 16 bits of an `f32`, so its widen is
//! a shift and its narrow a round-and-truncate — both branch-light enough that
//! the plain loops autovectorize, no intrinsics needed. Every path rounds
//! `f32`→reduced to nearest, ties to even, and is verified bit-for-bit against
//! the kernel and the independent reference.

use super::{narrow, widen};
use crate::types::{Bf16, F16};

/// A reduced-precision float format with a bulk slice widening into `f32`.
pub(crate) trait WidenToF32: Sized {
    /// Widen `src` into `dst`, writing `min(src.len(), dst.len())` elements.
    fn widen_all(src: &[Self], dst: &mut [f32]);
}

/// A reduced-precision float format with a bulk slice narrowing from `f32`,
/// rounding to nearest with ties to even.
pub(crate) trait NarrowFromF32: Sized {
    /// Narrow `src` into `dst`, writing `min(src.len(), dst.len())` elements.
    fn narrow_all(src: &[f32], dst: &mut [Self]);
}

/// Scalar widening loop for any `(E, M)` format.
#[inline]
fn widen_scalar<const E: u32, const M: u32>(src: &[u16], dst: &mut [f32]) {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = f32::from_bits(widen::<E, M>(u32::from(src[i])));
    }
}

/// Scalar narrowing loop for any `(E, M)` format.
#[inline]
fn narrow_scalar<const E: u32, const M: u32>(src: &[f32], dst: &mut [u16]) {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = narrow::<E, M>(src[i].to_bits()) as u16;
    }
}

impl WidenToF32 for F16 {
    /// The dispatch is `#[inline]` on purpose: it is a feature probe and a call,
    /// and a consumer that already runs inside a matching `#[target_feature]`
    /// scope can then inline the accelerated body itself, keeping the converted
    /// lanes in registers instead of round-tripping them through the caller's
    /// buffer. Nothing about the conversion changes; only where it may be placed.
    #[inline]
    fn widen_all(src: &[Self], dst: &mut [f32]) {
        // `F16` is `#[repr(transparent)]` over `u16`, so the reinterpret is a
        // layout no-op (same size and alignment).
        let src = crate::layout::cast_slice::<F16, u16>(src);
        #[cfg(target_arch = "x86_64")]
        {
            if has_f16c() {
                // SAFETY: `has_f16c()` confirmed F16C support at runtime; the
                // kernel bounds its reads/writes to the shorter of the slices.
                unsafe {
                    widen_hardware_x86(src, dst);
                }
                return;
            }
        }
        widen_scalar::<5, 10>(src, dst);
    }
}

impl NarrowFromF32 for F16 {
    /// See [`F16::widen_all`](WidenToF32::widen_all) for why the dispatch is
    /// `#[inline]`.
    #[inline]
    fn narrow_all(src: &[f32], dst: &mut [Self]) {
        let dst = crate::layout::cast_slice_mut::<F16, u16>(dst);
        #[cfg(target_arch = "x86_64")]
        {
            if has_f16c() {
                // SAFETY: `has_f16c()` confirmed F16C support at runtime; the
                // kernel bounds its reads/writes to the shorter of the slices.
                unsafe {
                    narrow_hardware_x86(src, dst);
                }
                return;
            }
        }
        narrow_scalar::<5, 10>(src, dst);
    }
}

impl WidenToF32 for Bf16 {
    /// `bfloat16` is the high 16 bits of the `f32`, so widening is a left shift
    /// — exact for every value (normals, subnormals, infinity, NaN) and the loop
    /// autovectorizes to a plain unpack/shift, no F16C needed.
    #[inline]
    fn widen_all(src: &[Self], dst: &mut [f32]) {
        let n = src.len().min(dst.len());
        for i in 0..n {
            dst[i] = f32::from_bits(u32::from(src[i].0) << 16);
        }
    }
}

impl NarrowFromF32 for Bf16 {
    /// Branchless per element so the loop autovectorizes.
    #[inline]
    fn narrow_all(src: &[f32], dst: &mut [Self]) {
        let n = src.len().min(dst.len());
        for i in 0..n {
            dst[i] = Self(round_high_half(src[i].to_bits()));
        }
    }
}

/// Round an `f32` bit pattern to its high half, ties to even — bit-identical to
/// `narrow::<8, 7>`.
#[inline]
fn round_high_half(bits: u32) -> u16 {
    // Round to nearest, ties to even: add `0x7FFF` plus the retained LSB, then
    // truncate. `wrapping_add` avoids a debug overflow panic on the (discarded)
    // NaN path; no wrap occurs for finite/infinite inputs.
    let rounded = bits.wrapping_add(0x7FFF).wrapping_add((bits >> 16) & 1) >> 16;
    // A NaN must keep a nonzero mantissa (else it collapses to infinity), forcing
    // the low mantissa bit when the retained payload is empty — the kernel's rule.
    let high = bits >> 16;
    #[expect(
        clippy::verbose_bit_mask,
        reason = "the mask names the retained NaN payload bits; trailing_zeros() obscures it"
    )]
    let nan = high | u32::from(high & 0x7F == 0);
    let is_nan = (bits & 0x7F80_0000 == 0x7F80_0000) && (bits & 0x007F_FFFF != 0);
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
unsafe fn widen_hardware_x86(src: &[u16], dst: &mut [f32]) {
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
unsafe fn narrow_hardware_x86(src: &[f32], dst: &mut [u16]) {
    use core::arch::x86_64::{
        _mm256_cvtps_ph, _mm256_loadu_ps, _mm_storeu_si128, _MM_FROUND_TO_NEAREST_INT,
    };

    let n = src.len().min(dst.len());
    let mut i = 0;
    while i + 8 <= n {
        let values = _mm256_loadu_ps(src.as_ptr().add(i));
        // `_MM_FROUND_TO_NEAREST_INT` (0) rounds to nearest, ties to even —
        // matching the scalar kernel and the reference.
        let narrowed = _mm256_cvtps_ph::<{ _MM_FROUND_TO_NEAREST_INT }>(values);
        _mm_storeu_si128(dst.as_mut_ptr().add(i).cast(), narrowed);
        i += 8;
    }
    while i < n {
        dst[i] = narrow::<5, 10>(src[i].to_bits()) as u16;
        i += 1;
    }
}
