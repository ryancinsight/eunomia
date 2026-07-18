//! Safe reinterpretation between [`Pod`] values, slices, and bytes.
//!
//! The stack's byte-marshalling surface (GPU uniform upload via [`bytes_of`],
//! slice readback/upload via [`cast_slice`], header parsing via [`from_bytes`]),
//! owned natively over [`Pod`] instead of borrowed from `bytemuck`. Each `unsafe`
//! block is justified by the [`Pod`] contract (no padding, every bit pattern
//! valid) plus an explicit length/alignment check.

use super::Pod;
use core::mem::{align_of, size_of};

/// Why a fallible reinterpretation could not be performed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PodCastError {
    /// The source pointer is not aligned for the destination type.
    TargetAlignmentMismatch,
    /// The source byte length is not an exact multiple of the destination size.
    SizeMismatch,
}

impl core::fmt::Display for PodCastError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::TargetAlignmentMismatch => "source is misaligned for the target type",
            Self::SizeMismatch => "source byte length is not a multiple of the target size",
        };
        f.write_str(msg)
    }
}

impl core::error::Error for PodCastError {}

/// View a [`Pod`] value as its raw bytes.
#[inline]
#[must_use]
pub fn bytes_of<T: Pod>(value: &T) -> &[u8] {
    // SAFETY: `T: Pod` has no padding and every bit is initialized, so the
    // `size_of::<T>()` bytes behind `value` are a valid `[u8]` of that length.
    unsafe { core::slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) }
}

/// View a [`Pod`] value as its raw bytes, mutably.
#[inline]
pub fn bytes_of_mut<T: Pod>(value: &mut T) -> &mut [u8] {
    // SAFETY: as [`bytes_of`]; `T: Pod` accepts any byte pattern written back.
    unsafe { core::slice::from_raw_parts_mut((value as *mut T).cast::<u8>(), size_of::<T>()) }
}

/// Compute the destination length for reinterpreting `src_len` elements of
/// `size_of::<A>()` bytes as `B`, or the reason it cannot be done.
#[inline]
fn cast_len<A, B>(src_len: usize) -> Result<usize, PodCastError> {
    let src_bytes = size_of::<A>() * src_len;
    if size_of::<B>() == 0 {
        // A slice of ZSTs carries no bytes; the only sound target length is 0.
        return Ok(0);
    }
    if !src_bytes.is_multiple_of(size_of::<B>()) {
        return Err(PodCastError::SizeMismatch);
    }
    Ok(src_bytes / size_of::<B>())
}

/// Reinterpret a slice of one [`Pod`] type as a slice of another, fallibly.
///
/// # Errors
/// [`PodCastError::SizeMismatch`] if the source byte length is not a multiple of
/// `size_of::<B>()`; [`PodCastError::TargetAlignmentMismatch`] if the source
/// pointer is not aligned for `B`.
pub fn try_cast_slice<A: Pod, B: Pod>(a: &[A]) -> Result<&[B], PodCastError> {
    let new_len = cast_len::<A, B>(a.len())?;
    if !(a.as_ptr() as usize).is_multiple_of(align_of::<B>()) {
        return Err(PodCastError::TargetAlignmentMismatch);
    }
    // SAFETY: the byte length divides evenly into `new_len` `B`s, the source is
    // aligned for `B`, and both `A` and `B` are `Pod` (no padding, every bit
    // pattern valid), so the elements are valid `B`s over the same allocation.
    Ok(unsafe { core::slice::from_raw_parts(a.as_ptr().cast::<B>(), new_len) })
}

/// Reinterpret a mutable slice of one [`Pod`] type as another, fallibly.
///
/// # Errors
/// As [`try_cast_slice`].
pub fn try_cast_slice_mut<A: Pod, B: Pod>(a: &mut [A]) -> Result<&mut [B], PodCastError> {
    let new_len = cast_len::<A, B>(a.len())?;
    if !(a.as_ptr() as usize).is_multiple_of(align_of::<B>()) {
        return Err(PodCastError::TargetAlignmentMismatch);
    }
    // SAFETY: as [`try_cast_slice`]; the exclusive borrow is preserved because
    // the result spans the same bytes with no aliasing.
    Ok(unsafe { core::slice::from_raw_parts_mut(a.as_mut_ptr().cast::<B>(), new_len) })
}

/// Reinterpret a slice of one [`Pod`] type as a slice of another.
///
/// # Panics
/// If the source length or alignment is incompatible with `B` (see
/// [`try_cast_slice`]).
#[inline]
#[must_use]
pub fn cast_slice<A: Pod, B: Pod>(a: &[A]) -> &[B] {
    match try_cast_slice(a) {
        Ok(b) => b,
        Err(e) => panic!("cast_slice: {e}"),
    }
}

/// Reinterpret a mutable slice of one [`Pod`] type as a slice of another.
///
/// # Panics
/// If the source length or alignment is incompatible with `B`.
#[inline]
pub fn cast_slice_mut<A: Pod, B: Pod>(a: &mut [A]) -> &mut [B] {
    match try_cast_slice_mut(a) {
        Ok(b) => b,
        Err(e) => panic!("cast_slice_mut: {e}"),
    }
}

/// Reinterpret a byte slice as a reference to a single [`Pod`] value, fallibly.
///
/// # Errors
/// [`PodCastError::SizeMismatch`] unless `bytes.len() == size_of::<T>()`;
/// [`PodCastError::TargetAlignmentMismatch`] if the bytes are not aligned for `T`.
pub fn try_from_bytes<T: Pod>(bytes: &[u8]) -> Result<&T, PodCastError> {
    if bytes.len() != size_of::<T>() {
        return Err(PodCastError::SizeMismatch);
    }
    if !(bytes.as_ptr() as usize).is_multiple_of(align_of::<T>()) {
        return Err(PodCastError::TargetAlignmentMismatch);
    }
    // SAFETY: length equals `size_of::<T>()`, the pointer is aligned for `T`, and
    // `T: Pod` accepts any bit pattern, so the bytes are a valid `T`.
    Ok(unsafe { &*bytes.as_ptr().cast::<T>() })
}

/// Reinterpret a byte slice as a reference to a single [`Pod`] value.
///
/// # Panics
/// If `bytes.len() != size_of::<T>()` or the bytes are misaligned for `T`.
#[inline]
#[must_use]
pub fn from_bytes<T: Pod>(bytes: &[u8]) -> &T {
    match try_from_bytes(bytes) {
        Ok(t) => t,
        Err(e) => panic!("from_bytes: {e}"),
    }
}

/// Read a [`Pod`] value out of a byte slice without any alignment requirement.
///
/// Copies `size_of::<T>()` bytes out by value, so the source need not be aligned.
///
/// # Panics
/// If `bytes.len() < size_of::<T>()`.
#[inline]
#[must_use]
pub fn pod_read_unaligned<T: Pod>(bytes: &[u8]) -> T {
    assert!(
        bytes.len() >= size_of::<T>(),
        "pod_read_unaligned: buffer shorter than the target type",
    );
    // SAFETY: the length is checked, `read_unaligned` needs no alignment, and
    // `T: Pod` makes any `size_of::<T>()`-byte pattern a valid owned `T`.
    unsafe { bytes.as_ptr().cast::<T>().read_unaligned() }
}
