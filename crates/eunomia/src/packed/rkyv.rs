//! rkyv zero-copy serialization support for packed 4-bit containers.

use crate::packed::cow::Packed4Cow;
use crate::packed::slice::{Packable4, Packed4Slice};
use crate::packed::vec::Packed4Vec;
use rkyv::bytecheck::CheckBytes;
use rkyv::munge::munge;
use rkyv::rancor::{fail, Fallible, Source};
use rkyv::ser::{Allocator, Writer};
use rkyv::vec::{ArchivedVec, VecResolver};
use rkyv::{Archive, Deserialize, Place, Portable, Serialize};

/// An archive whose element count exceeds the nibbles its data section holds.
///
/// The derived byte checks validate each field in isolation; nothing in them
/// relates `len` to the length of `data`. That relationship is the container's
/// actual invariant, and an archive violating it would hand out a view reading
/// past the buffer — the failure class RUSTSEC-2026-0235 describes. It is
/// therefore checked explicitly, not assumed.
#[derive(Debug)]
struct LengthExceedsData {
    len: usize,
    capacity: usize,
}

impl core::fmt::Display for LengthExceedsData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "archived packed container declares {} elements but its data section holds {} nibbles",
            self.len, self.capacity
        )
    }
}

impl core::error::Error for LengthExceedsData {}

/// Shared invariant check for both archived containers.
fn verify_length<E: Source>(len: usize, data: &ArchivedVec<u8>) -> Result<(), E> {
    let capacity = data.len() * 2;
    if len > capacity {
        fail!(LengthExceedsData { len, capacity });
    }
    Ok(())
}

/// Archived representation of a `Packed4Cow` for zero-copy deserialization.
///
/// `Portable` is derived rather than hand-asserted: the derive checks that
/// every field is itself portable, which is the property that makes reading
/// this struct straight out of an archive sound.
#[derive(Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck, verify)]
#[repr(C)]
pub struct ArchivedPacked4Cow<T: Packable4> {
    pub(crate) data: ArchivedVec<u8>,
    pub(crate) len: rkyv::Archived<usize>,
    pub(crate) _marker: core::marker::PhantomData<T>,
}

/// Resolver type for `Packed4Cow`.
///
/// Only the data vector carries resolver state; `usize` resolves without any,
/// so there is no length field to thread through.
pub struct Packed4CowResolver {
    pub(crate) data_resolver: VecResolver,
}

impl<T: Packable4> Archive for Packed4Cow<'_, T> {
    type Archived = ArchivedPacked4Cow<T>;
    type Resolver = Packed4CowResolver;

    #[inline]
    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        let view = self.as_view();
        // 0.8 projects fields through `Place` instead of raw pointer offsets,
        // so the position arithmetic the 0.7 implementation did by hand is now
        // the macro's job and cannot drift from the struct layout.
        munge!(let ArchivedPacked4Cow { data, len, _marker: _ } = out);
        ArchivedVec::resolve_from_slice(view.as_packed_slice(), resolver.data_resolver, data);
        // `usize`'s resolver is the unit type: the length is written directly,
        // in the archive's endianness, by rkyv's own integer impl.
        view.len().resolve((), len);
    }
}

impl<T: Packable4, S> Serialize<S> for Packed4Cow<'_, T>
where
    S: Fallible + Allocator + Writer + ?Sized,
{
    #[inline]
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let view = self.as_view();
        let data_resolver = ArchivedVec::serialize_from_slice(view.as_packed_slice(), serializer)?;
        Ok(Packed4CowResolver { data_resolver })
    }
}

impl<T: Packable4, D> Deserialize<Packed4Cow<'static, T>, D> for ArchivedPacked4Cow<T>
where
    D: Fallible + ?Sized,
{
    #[inline]
    fn deserialize(&self, _deserializer: &mut D) -> Result<Packed4Cow<'static, T>, D::Error> {
        let mut vec = Packed4Vec::with_capacity(self.len());
        vec.data.extend_from_slice(self.data.as_slice());
        vec.len = self.len();
        Ok(Packed4Cow::Owned(vec))
    }
}

impl<T: Packable4> ArchivedPacked4Cow<T> {
    /// Returns the logical length of the archived packed container.
    #[inline]
    pub fn len(&self) -> usize {
        // `Archived<usize>` is an endian-aware integer in 0.8, so this is a
        // plain conversion rather than a deserializer round trip.
        self.len.to_native() as usize
    }

    /// Returns `true` if the archived packed container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Zero-copy conversion of the archived container to a borrowed `Packed4Cow`.
    #[inline]
    pub fn as_borrowed(&self) -> Option<Packed4Cow<'_, T>> {
        let len = self.len();
        Packed4Slice::new(self.data.as_slice(), len).map(Packed4Cow::Borrowed)
    }
}

/// Archived representation of a `Packed4Vec`.
#[derive(Portable, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck, verify)]
#[repr(C)]
pub struct ArchivedPacked4Vec<T: Packable4> {
    pub(crate) data: ArchivedVec<u8>,
    pub(crate) len: rkyv::Archived<usize>,
    pub(crate) _marker: core::marker::PhantomData<T>,
}

/// Resolver type for `Packed4Vec`.
///
/// Only the data vector carries resolver state; `usize` resolves without any,
/// so there is no length field to thread through.
pub struct Packed4VecResolver {
    pub(crate) data_resolver: VecResolver,
}

impl<T: Packable4> Archive for Packed4Vec<T> {
    type Archived = ArchivedPacked4Vec<T>;
    type Resolver = Packed4VecResolver;

    #[inline]
    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        munge!(let ArchivedPacked4Vec { data, len, _marker: _ } = out);
        ArchivedVec::resolve_from_slice(self.as_packed_slice(), resolver.data_resolver, data);
        self.len.resolve((), len);
    }
}

impl<T: Packable4, S> Serialize<S> for Packed4Vec<T>
where
    S: Fallible + Allocator + Writer + ?Sized,
{
    #[inline]
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let data_resolver = ArchivedVec::serialize_from_slice(self.as_packed_slice(), serializer)?;
        Ok(Packed4VecResolver { data_resolver })
    }
}

impl<T: Packable4, D> Deserialize<Packed4Vec<T>, D> for ArchivedPacked4Vec<T>
where
    D: Fallible + ?Sized,
{
    #[inline]
    fn deserialize(&self, _deserializer: &mut D) -> Result<Packed4Vec<T>, D::Error> {
        let mut vec = Packed4Vec::with_capacity(self.len());
        vec.data.extend_from_slice(self.data.as_slice());
        vec.len = self.len();
        Ok(vec)
    }
}

impl<T: Packable4> ArchivedPacked4Vec<T> {
    /// Returns the logical length of the archived vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.to_native() as usize
    }

    /// Returns `true` if the archived vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert the archived vector to a borrowed `Packed4Slice` view.
    ///
    /// Returns `None` when the declared length exceeds the data section, which
    /// validated access already rejects; the check is repeated here so an
    /// archive reached through `access_unchecked` cannot produce a view that
    /// reads past the buffer.
    #[inline]
    pub fn as_view(&self) -> Option<Packed4Slice<'_, T>> {
        Packed4Slice::new(self.data.as_slice(), self.len())
    }
}

// SAFETY-adjacent contract: `verify` runs during validated access, before any
// borrowed view is handed out, so a container reaching `as_view` through
// `rkyv::access` has already had its length bounded against its data.
unsafe impl<T: Packable4, C> rkyv::bytecheck::Verify<C> for ArchivedPacked4Cow<T>
where
    C: Fallible + ?Sized,
    C::Error: Source,
{
    fn verify(&self, _context: &mut C) -> Result<(), C::Error> {
        verify_length(self.len(), &self.data)
    }
}

unsafe impl<T: Packable4, C> rkyv::bytecheck::Verify<C> for ArchivedPacked4Vec<T>
where
    C: Fallible + ?Sized,
    C::Error: Source,
{
    fn verify(&self, _context: &mut C) -> Result<(), C::Error> {
        verify_length(self.len(), &self.data)
    }
}

#[cfg(test)]
mod tests;
