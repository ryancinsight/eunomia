use super::*;
use crate::Complex64;

#[test]
fn reference_impl_delegates_to_inner() {
    let x: Vec<f64> = (0..4).map(|i| f64::from(i) * 0.5).collect();
    let d = x.clone();
    for (xi, di) in x.iter().zip(&d) {
        assert_relative_eq!(xi, di, epsilon = 1e-12);
    }
}

#[test]
fn reference_impl_does_not_recurse_for_defaults() {
    let a_val = 1.0_f64;
    let b_val = 1.0 + f64::EPSILON;
    let a = &a_val;
    let b = &b_val;
    assert!(<f64 as RelativeEq>::relative_eq(
        a,
        b,
        <f64 as RelativeEq>::default_epsilon(),
        <f64 as RelativeEq>::default_max_relative(),
    ));
    assert_eq!(
        <f64 as RelativeEq>::default_epsilon(),
        <&f64 as RelativeEq>::default_epsilon(),
    );
}

#[test]
fn complex_relative_eq_compares_both_components() {
    let a = Complex64::new(1.0, 2.0);
    let b = Complex64::new(1.0 + 1e-15, 2.0 + 1e-15);
    assert_relative_eq!(a, b, epsilon = 1e-10);

    let imaginary_mismatch = Complex64::new(1.0, 2.1);
    assert!(!a.relative_eq(&imaginary_mismatch, 1e-10, 1e-10));

    let real_mismatch = Complex64::new(1.1, 2.0);
    assert!(!a.relative_eq(&real_mismatch, 1e-10, 1e-10));
}

#[test]
fn complex_abs_diff_eq_works_through_macro() {
    let a = Complex64::new(1.0, 0.0);
    let b = Complex64::new(1.0 + 1e-15, 0.0);
    assert_abs_diff_eq!(a, b, epsilon = 1e-10);
}

#[test]
fn complex_by_reference_composes_both_impls() {
    let values = vec![Complex64::new(1.0, 2.0), Complex64::new(3.0, 4.0)];
    let expected = values.clone();
    for (value, expected) in values.iter().zip(&expected) {
        assert_relative_eq!(value, expected, epsilon = 1e-12);
    }
}

#[test]
fn bool_macros_match_asserting_counterparts() {
    assert!(relative_eq!(1.0_f64, 1.0 + f64::EPSILON));
    assert!(!relative_eq!(1.0_f64, 1.1));
    assert!(relative_eq!(1.0_f64, 1.1, epsilon = 0.2));
    assert!(relative_eq!(1.0_f64, 1.1, max_relative = 0.2));
    assert!(abs_diff_eq!(1.0_f64, 1.0 + 1e-15, epsilon = 1e-10));
    assert!(!abs_diff_eq!(1.0_f64, 1.1));
}

#[test]
fn relative_scale_uses_both_absolute_magnitudes() {
    assert!(relative_eq!(-100.0_f64, -90.0, max_relative = 0.1));
    assert!(relative_eq!(-100.0_f32, -90.0, max_relative = 0.1));
    assert!(!relative_eq!(-100.0_f64, -80.0, max_relative = 0.1));
    assert!(!relative_eq!(-100.0_f32, -80.0, max_relative = 0.1));
}

#[test]
fn scalar_contract_preserves_precision_and_special_values() {
    let epsilon: f32 = <f32 as RelativeEq>::default_epsilon();
    assert_eq!(epsilon, f32::EPSILON);
    assert!(relative_eq!(f32::INFINITY, f32::INFINITY));
    assert!(relative_eq!(f64::NEG_INFINITY, f64::NEG_INFINITY));
    assert!(!relative_eq!(f32::INFINITY, f32::NEG_INFINITY));
    assert!(!relative_eq!(f64::NAN, f64::NAN));
    assert!(relative_eq!(0.0_f32, -0.0_f32));
}
