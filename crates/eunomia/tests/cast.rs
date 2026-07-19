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
