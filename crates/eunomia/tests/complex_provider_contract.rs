use core::any::type_name;
use core::ops::Neg;

use eunomia::{
    Bf16, Bf4, Bf8, Complex, Complex32, Complex64, ComplexField, NumericElement, RealElement, F16,
    F32, F4, F64, F8, I16, I32, I8,
};

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

/// Cells of the identity grid for which a `Max` fold seeded with
/// `Complex<T>::MIN_VALUE`, or a `Min` fold seeded with `Complex<T>::MAX_VALUE`,
/// does not return the element in both argument orders.
///
/// `max_scalar`/`min_scalar` order by `re`, then `im`, so the constants must be
/// the least and greatest pairs under that lexicographic order. The grid pairs
/// each real part with the imaginary extremes, where a zero imaginary seed
/// fails; no component is NaN.
fn complex_reduction_identity_failures<T>() -> Vec<String>
where
    T: RealElement + Neg<Output = T>,
{
    let max_identity = <Complex<T> as NumericElement>::MIN_VALUE;
    let min_identity = <Complex<T> as NumericElement>::MAX_VALUE;
    let reals = [T::MIN_VALUE, T::MAX_VALUE, T::ONE];
    let imaginaries = [T::MIN_VALUE, -T::ONE, T::ZERO, T::ONE, T::MAX_VALUE];
    let mut failures = Vec::new();
    for re in reals {
        for im in imaginaries {
            let element = Complex::new(re, im);
            let cells = [
                (
                    "max",
                    "left",
                    NumericElement::max_scalar(max_identity, element),
                ),
                (
                    "max",
                    "right",
                    NumericElement::max_scalar(element, max_identity),
                ),
                (
                    "min",
                    "left",
                    NumericElement::min_scalar(min_identity, element),
                ),
                (
                    "min",
                    "right",
                    NumericElement::min_scalar(element, min_identity),
                ),
            ];
            for (fold, side, result) in cells {
                if result != element {
                    failures.push(format!(
                        "{} {fold} seed {side}: {element:?} became {result:?}",
                        type_name::<T>()
                    ));
                }
            }
        }
    }
    failures
}

/// Runs [`complex_reduction_identity_failures`] on each listed component type.
macro_rules! identity_failures_over {
    ($($component:ty),+ $(,)?) => {{
        let mut failures = Vec::new();
        $(failures.extend(complex_reduction_identity_failures::<$component>());)+
        failures
    }};
}

#[test]
fn complex_reduction_identity_returns_the_element() {
    let failures = identity_failures_over!(
        f32, f64, i8, i16, i32, i64, isize, F16, Bf16, F32, F64, Bf8, Bf4, F8, F4, I8, I16, I32,
    );
    assert_eq!(failures, Vec::<String>::new());
}

#[test]
fn complex_fold_identities_are_the_extreme_pairs_of_a_real_component() {
    let min_pair = Complex64::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    let max_pair = Complex64::new(f64::INFINITY, f64::INFINITY);
    assert_eq!(<Complex64 as NumericElement>::MIN_VALUE, min_pair);
    assert_eq!(<Complex64 as NumericElement>::MAX_VALUE, max_pair);

    let low = Complex64::new(-3.5, 7.25);
    let high = Complex64::new(-3.5, 9.0);
    assert_eq!(NumericElement::max_scalar(min_pair, low), low);
    assert_eq!(NumericElement::max_scalar(low, min_pair), low);
    assert_eq!(NumericElement::min_scalar(max_pair, low), low);
    assert_eq!(NumericElement::min_scalar(low, max_pair), low);

    // Two distinct non-seed elements fold to the same extreme in either order.
    assert_eq!(NumericElement::max_scalar(low, high), high);
    assert_eq!(NumericElement::max_scalar(high, low), high);
    assert_eq!(NumericElement::min_scalar(low, high), low);
    assert_eq!(NumericElement::min_scalar(high, low), low);
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
