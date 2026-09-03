//! Byte-layout law: eunomia's native `Zeroable`/`Pod` markers and the safe
//! reinterpretation surface (`bytes_of`, `cast_slice`, `from_bytes`, …) the
//! stack marshals GPU/FFI buffers through — owned here instead of borrowed from
//! `bytemuck`. Eunomia-owned types implement both marker vocabularies in the
//! canonical marker module; backend-owned ABI types remain the
//! responsibility of their backend crates.

mod bytes;
mod marker;

pub use bytes::{
    bytes_of, bytes_of_mut, cast_slice, cast_slice_mut, from_bytes, pod_read_unaligned,
    try_cast_slice, try_cast_slice_mut, try_from_bytes, try_pod_read_unaligned, PodCastError,
};
pub use marker::{Pod, Zeroable};
