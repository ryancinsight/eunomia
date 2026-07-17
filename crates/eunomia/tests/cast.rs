use eunomia::{Bf16, Bf4, Bf8, CastFrom, FloatElement, F16, F32, F4, F64, F8};

fn assert_index_cast<T>()
where
    T: FloatElement,
    usize: CastFrom<T>,
{
    assert_eq!(usize::cast_from(T::from_f64(1.75)), 1);
}

#[test]
fn float_families_convert_integral_coordinates_to_indices() {
    assert_index_cast::<f32>();
    assert_index_cast::<f64>();
    assert_index_cast::<half::f16>();
    assert_index_cast::<half::bf16>();
    assert_index_cast::<F16>();
    assert_index_cast::<F32>();
    assert_index_cast::<F64>();
    assert_index_cast::<Bf16>();
    assert_index_cast::<Bf8>();
    assert_index_cast::<Bf4>();
    assert_index_cast::<F8>();
    assert_index_cast::<F4>();
}

#[test]
fn primitive_float_to_index_casts_follow_rust_saturation() {
    assert_eq!(usize::cast_from(-1.0_f32), 0);
    assert_eq!(usize::cast_from(f64::INFINITY), usize::MAX);
}
