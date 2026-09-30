use eunomia::{Bf16, Bf4, Bf8, CastFrom, FloatElement, F16, F32, F4, F64, F8};

fn assert_index_cast<T>(expected: usize)
where
    T: FloatElement,
    usize: CastFrom<T>,
{
    assert_eq!(usize::cast_from(T::from_f64(1.75)), expected);
}

#[test]
fn float_families_convert_integral_coordinates_to_indices() {
    assert_index_cast::<f32>(1);
    assert_index_cast::<f64>(1);
    assert_index_cast::<F16>(1);
    assert_index_cast::<F32>(1);
    assert_index_cast::<F64>(1);
    assert_index_cast::<Bf16>(1);
    assert_index_cast::<Bf8>(1);
    // At 1.75, E2M1 and E3M0 are exactly halfway between 1.5/2 and 1/2,
    // respectively; ties-to-even quantization selects 2 before the index cast.
    assert_index_cast::<Bf4>(2);
    assert_index_cast::<F8>(1);
    assert_index_cast::<F4>(2);
}

#[test]
fn primitive_float_to_index_casts_follow_rust_saturation() {
    assert_eq!(usize::cast_from(-1.0_f32), 0);
    assert_eq!(usize::cast_from(f64::INFINITY), usize::MAX);
}

/// Compiles only when `T` converts from each of the ten fixed-width numeric
/// primitives, so a missing pair fails the build rather than a caller's.
fn casts_from_every_primitive<T>()
where
    T: CastFrom<u8>
        + CastFrom<i8>
        + CastFrom<u16>
        + CastFrom<i16>
        + CastFrom<u32>
        + CastFrom<i32>
        + CastFrom<u64>
        + CastFrom<i64>
        + CastFrom<f32>
        + CastFrom<f64>,
{
}

#[test]
fn every_fixed_width_primitive_converts_from_every_other() {
    casts_from_every_primitive::<u8>();
    casts_from_every_primitive::<i8>();
    casts_from_every_primitive::<u16>();
    casts_from_every_primitive::<i16>();
    casts_from_every_primitive::<u32>();
    casts_from_every_primitive::<i32>();
    casts_from_every_primitive::<u64>();
    casts_from_every_primitive::<i64>();
    casts_from_every_primitive::<f32>();
    casts_from_every_primitive::<f64>();
}

/// Signed sources reinterpret into unsigned targets modulo 2^bits after sign
/// extension, and widen into `i64` exactly — Rust's `as` semantics.
#[test]
fn signed_primitives_cast_into_unsigned_and_i64_as_rust_does() {
    assert_eq!(u8::cast_from(-1_i8), u8::MAX);
    assert_eq!(u16::cast_from(-1_i8), u16::MAX);
    assert_eq!(u32::cast_from(i8::MIN), 0xFFFF_FF80);
    assert_eq!(u64::cast_from(-2_i8), u64::MAX - 1);
    assert_eq!(i64::cast_from(i8::MIN), -128);
    assert_eq!(u8::cast_from(0x0181_i16), 0x81);
    assert_eq!(u16::cast_from(-1_i16), u16::MAX);
    assert_eq!(u32::cast_from(i16::MIN), 0xFFFF_8000);
    assert_eq!(u64::cast_from(-1_i16), u64::MAX);
    assert_eq!(i64::cast_from(i16::MIN), -32_768);
    assert_eq!(u8::cast_from(0x1_0000_0102_i64), 0x02);
    assert_eq!(u16::cast_from(-1_i64), u16::MAX);
    assert_eq!(u32::cast_from((1_i64 << 32) + 7), 7);
    assert_eq!(u64::cast_from(i64::MIN), 1_u64 << 63);
}
