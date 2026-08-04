//! Round-trip and validation coverage for the packed-container archives.
//!
//! The 0.7 implementation these replace had no tests at all, so the archives
//! were unverified through a major-version migration. Each test below asserts a
//! value-semantic property: that every element survives the round trip, that a
//! borrowed view reads the same elements without copying, and that a corrupted
//! archive is rejected rather than read.

use alloc::vec::Vec;

use crate::packed::cow::Packed4Cow;
use crate::packed::vec::Packed4Vec;
use crate::{Bf4, Packable4};
use rkyv::rancor::Error;

/// Every representable `Bf4` code, so the round trip covers the whole domain
/// rather than a sample of it.
fn every_code() -> Packed4Vec<Bf4> {
    let mut vec = Packed4Vec::with_capacity(16);
    for code in 0..16u8 {
        vec.push(Bf4(code));
    }
    vec
}

fn elements<T: Packable4 + PartialEq + core::fmt::Debug>(vec: &Packed4Vec<T>) -> Vec<T> {
    (0..vec.len())
        .map(|i| vec.get(i).expect("index below len"))
        .collect()
}

#[test]
fn vec_round_trips_every_element() {
    let original = every_code();
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    let restored: Packed4Vec<Bf4> = rkyv::from_bytes::<_, Error>(&bytes).expect("deserialize");

    assert_eq!(restored.len(), original.len());
    assert_eq!(elements(&restored), elements(&original));
}

#[test]
fn cow_round_trips_every_element() {
    let original = Packed4Cow::Owned(every_code());
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    let restored: Packed4Cow<'static, Bf4> =
        rkyv::from_bytes::<_, Error>(&bytes).expect("deserialize");

    assert_eq!(restored.len(), original.len());
    for i in 0..original.len() {
        assert_eq!(restored.get(i), original.get(i), "element {i}");
    }
}

#[test]
fn odd_length_round_trips() {
    // An odd element count leaves a half-used final nibble; the length field,
    // not the byte count, is what makes it recoverable.
    let mut original = Packed4Vec::<Bf4>::new();
    for code in [1u8, 7, 15] {
        original.push(Bf4(code));
    }
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    let restored: Packed4Vec<Bf4> = rkyv::from_bytes::<_, Error>(&bytes).expect("deserialize");

    assert_eq!(restored.len(), 3);
    assert_eq!(elements(&restored), elements(&original));
}

#[test]
fn empty_container_round_trips() {
    let original = Packed4Vec::<Bf4>::new();
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    let restored: Packed4Vec<Bf4> = rkyv::from_bytes::<_, Error>(&bytes).expect("deserialize");
    assert_eq!(restored.len(), 0);
    assert!(restored.is_empty());
}

#[test]
fn archived_view_reads_without_deserializing() {
    // The point of the archive: read elements straight out of the buffer.
    let original = every_code();
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    let archived =
        rkyv::access::<super::ArchivedPacked4Vec<Bf4>, Error>(&bytes).expect("validated access");

    assert_eq!(archived.len(), original.len());
    let view = archived.as_view().expect("validated archive yields a view");
    for i in 0..original.len() {
        assert_eq!(view.get(i), original.get(i), "element {i}");
    }
}

#[test]
fn truncated_archive_is_rejected() {
    // Validated access is the guard the advisory was about: a short buffer must
    // fail, not produce a view over memory the archive does not own.
    let original = every_code();
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    let truncated = &bytes[..bytes.len() / 2];
    let result = rkyv::access::<super::ArchivedPacked4Vec<Bf4>, Error>(truncated);
    assert!(result.is_err(), "truncated archive must not validate");
}

#[test]
fn corrupted_length_is_rejected_or_bounded() {
    // Flipping bytes in the archive must never yield a view that reads past the
    // buffer. Either validation rejects it, or the recovered length stays
    // within what the data section can hold.
    let original = every_code();
    let bytes = rkyv::to_bytes::<Error>(&original).expect("serialize");
    for index in 0..bytes.len() {
        let mut corrupted = bytes.to_vec();
        corrupted[index] ^= 0xFF;
        if let Ok(archived) = rkyv::access::<super::ArchivedPacked4Vec<Bf4>, Error>(&corrupted) {
            // Validation admitted it, so the length invariant must hold and a
            // view must be constructible without reading past the buffer.
            assert!(
                archived.as_view().is_some(),
                "byte {index}: validated archive yielded an out-of-bounds length"
            );
        }
    }
}
