use eunomia::{Complex32, Complex64, ComplexField};

#[test]
fn complex_layout_round_trips_through_plain_arrays() {
    let single = Complex32::new(1.25, -2.5);
    let double = Complex64::new(-3.5, 7.25);

    assert_eq!(bytemuck::cast::<Complex32, [f32; 2]>(single), [1.25, -2.5]);
    assert_eq!(bytemuck::cast::<Complex64, [f64; 2]>(double), [-3.5, 7.25]);
}

#[test]
fn native_and_generic_identities_are_equivalent() {
    assert_eq!(Complex32::ZERO, <Complex32 as ComplexField>::zero());
    assert_eq!(Complex32::ONE, <Complex32 as ComplexField>::one());
    assert_eq!(Complex64::ZERO, <Complex64 as ComplexField>::zero());
    assert_eq!(Complex64::ONE, <Complex64 as ComplexField>::one());

    let value = Complex64::new(4.0, -3.0);
    assert_eq!(value + Complex64::ZERO, value);
    assert_eq!(value * Complex64::ONE, value);
}

#[cfg(feature = "numpy")]
#[test]
fn complex_types_implement_the_selected_numpy_element_contract() {
    fn assert_element<T: numpy::Element>() {}

    assert_element::<Complex32>();
    assert_element::<Complex64>();
}
