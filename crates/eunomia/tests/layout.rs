//! Value-semantic contracts for the native byte-layout surface
//! ([`eunomia::layout`]).

use eunomia::layout::{
    bytes_of, bytes_of_mut, cast_slice, cast_slice_mut, from_bytes, pod_read_unaligned,
    try_cast_slice, try_cast_vec, try_from_bytes, try_pod_read_unaligned, PodCastError,
};
use eunomia::{Bf16, Complex32, Zeroable, F16};

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
    assert_eq!(try_pod_read_unaligned::<u32>(&buf[1..]), Ok(value));
    assert_eq!(
        try_pod_read_unaligned::<u32>(&buf[..3]),
        Err(PodCastError::SizeMismatch),
    );
}

#[test]
fn eunomia_types_satisfy_the_bytemuck_boundary_contract() {
    fn assert_bytemuck_pod<T: bytemuck::Pod>() {}

    assert_bytemuck_pod::<F16>();
    assert_bytemuck_pod::<Bf16>();
    assert_bytemuck_pod::<Complex32>();

    let value = F16::from_bits(0x3C00);
    assert_eq!(bytemuck::bytes_of(&value), bytes_of(&value));
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

/// An exactly sized byte vector: `From<Box<[T]>>` keeps capacity equal to length.
fn exact_bytes(bytes: &[u8]) -> Vec<u8> {
    Vec::from(Box::<[u8]>::from(bytes))
}

#[test]
fn try_cast_vec_adopts_the_allocation_and_scales_length_and_capacity() {
    let bytes = exact_bytes(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let address = bytes.as_ptr().addr();
    let pixels = try_cast_vec::<u8, [u8; 4]>(bytes).expect("eight bytes are two whole pixels");
    assert_eq!(pixels, [[1, 2, 3, 4], [5, 6, 7, 8]]);
    assert_eq!(pixels.capacity(), 2);
    // The same allocation backs the result: nothing was copied.
    assert_eq!(pixels.as_ptr().addr(), address);
}

#[test]
fn try_cast_vec_round_trips_and_writes_through_the_shared_allocation() {
    let mut pixels = try_cast_vec::<u8, [u8; 4]>(exact_bytes(&[0; 12]))
        .expect("twelve bytes are three whole pixels");
    pixels[1] = [9, 8, 7, 6];
    let bytes = try_cast_vec::<[u8; 4], u8>(pixels).expect("pixels are whole bytes");
    assert_eq!(bytes, [0, 0, 0, 0, 9, 8, 7, 6, 0, 0, 0, 0]);
    assert_eq!(bytes.capacity(), 12);
}

#[test]
fn try_cast_vec_rejects_a_length_that_is_not_a_whole_number_of_targets() {
    let bytes = exact_bytes(&[1, 2, 3, 4, 5, 6]);
    let address = bytes.as_ptr().addr();
    let (error, returned) =
        try_cast_vec::<u8, [u8; 4]>(bytes).expect_err("six bytes end in a partial pixel");
    assert_eq!(error, PodCastError::SizeMismatch);
    assert_eq!(returned, [1, 2, 3, 4, 5, 6]);
    assert_eq!(returned.as_ptr().addr(), address);
}

#[test]
fn try_cast_vec_rejects_a_capacity_that_is_not_a_whole_number_of_targets() {
    // Nine bytes of capacity hold eight initialized bytes: the length divides
    // into pixels but the allocation does not, so freeing it as two pixels
    // would use a different layout than it was allocated with.
    let mut bytes = exact_bytes(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    bytes.truncate(8);
    assert_eq!(bytes.capacity(), 9);
    let (error, returned) = try_cast_vec::<u8, [u8; 4]>(bytes)
        .expect_err("nine bytes of capacity end in a partial pixel");
    assert_eq!(error, PodCastError::SizeMismatch);
    assert_eq!(returned.len(), 8);
    assert_eq!(returned.capacity(), 9);
}

#[test]
fn try_cast_vec_rejects_differing_alignment_even_when_the_sizes_divide() {
    let bytes = exact_bytes(&[1, 2, 3, 4]);
    let (error, returned) =
        try_cast_vec::<u8, u16>(bytes).expect_err("a byte allocation is not aligned for u16");
    assert_eq!(error, PodCastError::TargetAlignmentMismatch);
    assert_eq!(returned, [1, 2, 3, 4]);
}

#[test]
fn try_cast_vec_of_an_empty_vector_stays_empty() {
    let pixels =
        try_cast_vec::<u8, [u8; 4]>(Vec::new()).expect("an empty vector has no partial pixel");
    assert!(pixels.is_empty());
    assert_eq!(pixels.capacity(), 0);
}
