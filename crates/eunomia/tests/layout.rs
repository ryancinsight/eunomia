//! Value-semantic contracts for the native byte-layout surface
//! ([`eunomia::layout`]).

use eunomia::layout::{
    bytes_of, bytes_of_mut, cast_slice, cast_slice_mut, from_bytes, pod_read_unaligned,
    try_cast_slice, try_from_bytes, PodCastError,
};
use eunomia::{Complex32, Zeroable};

#[test]
fn zeroed_is_all_zero_bytes() {
    assert_eq!(u32::zeroed(), 0);
    assert_eq!(f64::zeroed(), 0.0);
    assert_eq!(Complex32::zeroed(), Complex32::new(0.0, 0.0));
    assert_eq!(bytes_of(&u64::zeroed()), &[0u8; 8]);
}

#[test]
fn bytes_of_matches_native_representation() {
    let x = 0x0102_0304_u32;
    assert_eq!(bytes_of(&x), &x.to_ne_bytes());
    let f = core::f32::consts::PI;
    assert_eq!(bytes_of(&f), &f.to_ne_bytes());
    assert_eq!(bytes_of(&x).len(), core::mem::size_of::<u32>());
}

#[test]
fn from_bytes_round_trips_single_value() {
    let x = 0xDEAD_BEEF_u32;
    let bytes = bytes_of(&x);
    assert_eq!(*from_bytes::<u32>(bytes), x);
    assert_eq!(try_from_bytes::<u32>(bytes), Ok(&x));
}

#[test]
fn bytes_of_mut_writes_through() {
    let mut x = 0u32;
    bytes_of_mut(&mut x).copy_from_slice(&0x1122_3344_u32.to_ne_bytes());
    assert_eq!(x, 0x1122_3344);
}

#[test]
fn cast_slice_reinterprets_and_round_trips() {
    let floats = [1.0_f32, 2.0, 3.0, 4.0];
    let bytes: &[u8] = cast_slice(&floats);
    assert_eq!(bytes.len(), 16);
    // The byte view came from an `f32` slice, so it re-casts back exactly.
    assert_eq!(cast_slice::<u8, f32>(bytes), &floats[..]);
}

#[test]
fn cast_slice_widens_bytes_to_scalars() {
    // Source is `u32`-aligned, so re-casting its bytes to `u32` is valid.
    let words = [0x1111_1111_u32, 0x2222_2222];
    let bytes: &[u8] = cast_slice(&words);
    assert_eq!(cast_slice::<u8, u32>(bytes), &words[..]);
}

#[test]
fn cast_slice_mut_writes_through() {
    let mut words = [0u32; 2];
    {
        let bytes: &mut [u8] = cast_slice_mut(&mut words);
        bytes.fill(0xFF);
    }
    assert_eq!(words, [u32::MAX, u32::MAX]);
}

#[test]
fn try_cast_slice_reports_size_mismatch() {
    let bytes = [0u8; 3];
    assert_eq!(
        try_cast_slice::<u8, u32>(&bytes),
        Err(PodCastError::SizeMismatch),
    );
}

#[test]
fn try_cast_slice_reports_alignment_mismatch() {
    // A `u32`-aligned buffer; offset 1 is 4 bytes long but misaligned for `u32`.
    let words = [0u32; 2];
    let bytes: &[u8] = cast_slice(&words);
    assert_eq!(
        try_cast_slice::<u8, u32>(&bytes[1..5]),
        Err(PodCastError::TargetAlignmentMismatch),
    );
}

#[test]
fn pod_read_unaligned_reads_from_any_offset() {
    let mut buf = [0u8; 8];
    let value = 0x0A0B_0C0D_u32;
    buf[1..5].copy_from_slice(&value.to_ne_bytes());
    // Offset 1 is misaligned for `u32`; the unaligned read still succeeds.
    assert_eq!(pod_read_unaligned::<u32>(&buf[1..]), value);
}

#[test]
fn complex_round_trips_through_bytes() {
    let z = Complex32::new(1.5, -2.25);
    let bytes = bytes_of(&z);
    assert_eq!(bytes.len(), 8);
    assert_eq!(*from_bytes::<Complex32>(bytes), z);
    // Layout-identical to a packed `[re, im]` pair.
    assert_eq!(cast_slice::<Complex32, f32>(&[z]), &[1.5_f32, -2.25]);
}
