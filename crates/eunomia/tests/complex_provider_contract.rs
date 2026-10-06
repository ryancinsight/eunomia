use eunomia::{Complex32, Complex64, ComplexField, NumericElement};

#[test]
fn complex_layout_round_trips_through_plain_arrays() {
    let single = Complex32::new(1.25, -2.5);
    let double = Complex64::new(-3.5, 7.25);

    assert_eq!(bytemuck::cast::<Complex32, [f32; 2]>(single), [1.25, -2.5]);
    assert_eq!(bytemuck::cast::<Complex64, [f64; 2]>(double), [-3.5, 7.25]);
}

#[test]
fn numeric_element_sqrt_is_the_principal_root() {
    // Exact small cases over the rectangular form: sqrt(3+4i) = 2+i,
    // sqrt(-4) = 2i, sqrt(0) = 0.
    let z = Complex64::new(3.0, 4.0);
    assert_eq!(
        <Complex64 as NumericElement>::sqrt(z),
        Complex64::new(2.0, 1.0)
    );
    let neg = Complex64::new(-4.0, 0.0);
    assert_eq!(
        <Complex64 as NumericElement>::sqrt(neg),
        Complex64::new(0.0, 2.0)
    );
    let zero = Complex64::new(0.0, 0.0);
    assert_eq!(<Complex64 as NumericElement>::sqrt(zero), zero);
    let z32 = Complex32::new(3.0, 4.0);
    assert_eq!(
        <Complex32 as NumericElement>::sqrt(z32),
        Complex32::new(2.0, 1.0)
    );

    // Differential: the rectangular trait form agrees with the inherent
    // polar oracle to a few ulps (different rounding paths), and each root
    // squares back to its input.
    for (re, im) in [(0.5, -1.25), (-2.5, 0.75), (100.0, 100.0), (1e-8, 3e-8)] {
        let w = Complex64::new(re, im);
        let a = <Complex64 as NumericElement>::sqrt(w);
        let b = w.sqrt();
        let scale = a.re.abs().max(a.im.abs()).max(1.0);
        let tol = 8.0 * f64::EPSILON * scale;
        assert!(
            (a.re - b.re).abs() <= tol && (a.im - b.im).abs() <= tol,
            "trait {a:?} vs inherent {b:?} at {w:?}"
        );
        let back = a * a;
        let tol2 = 16.0 * f64::EPSILON * re.abs().max(im.abs()).max(1.0);
        assert!(
            (back.re - re).abs() <= tol2 && (back.im - im).abs() <= tol2,
            "round-trip {back:?} vs {w:?}"
        );
    }
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

#[cfg(feature = "numpy")]
#[test]
fn complex_types_report_their_canonical_numpy_dtypes() {
    use numpy::Element;
    use pyo3::prelude::Python;
    use pyo3::types::PyAnyMethods;

    Python::initialize();
    Python::attach(|py| {
        let complex32_dtype = <Complex32 as Element>::get_dtype(py);
        let complex64_dtype = <Complex64 as Element>::get_dtype(py);

        let complex32_name = complex32_dtype
            .getattr("name")
            .expect("complex64 dtype exposes name")
            .extract::<String>()
            .expect("complex64 dtype name is a string");
        let complex64_name = complex64_dtype
            .getattr("name")
            .expect("complex128 dtype exposes name")
            .extract::<String>()
            .expect("complex128 dtype name is a string");
        let complex32_itemsize = complex32_dtype
            .getattr("itemsize")
            .expect("complex64 dtype exposes itemsize")
            .extract::<usize>()
            .expect("complex64 dtype itemsize is an integer");
        let complex64_itemsize = complex64_dtype
            .getattr("itemsize")
            .expect("complex128 dtype exposes itemsize")
            .extract::<usize>()
            .expect("complex128 dtype itemsize is an integer");

        assert_eq!(complex32_name, "complex64");
        assert_eq!(complex64_name, "complex128");
        assert_eq!(complex32_itemsize, core::mem::size_of::<Complex32>());
        assert_eq!(complex64_itemsize, core::mem::size_of::<Complex64>());
    });
}
