//! Digamma contract: independent anchors and domain behavior.
use eunomia::{FloatElement, NumericElement, F64};

fn trace() -> tracing::subscriber::DefaultGuard {
    let subscriber = tracing_subscriber::fmt()
        .with_test_writer()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .finish();
    tracing::subscriber::set_default(subscriber)
}

fn close(actual: f64, expected: f64, unit_roundoff: f64) {
    let _assert = tracing::info_span!("assert", actual, expected).entered();
    // Additional tight regression check; the per-input propagated budgets below
    // establish the mathematical fixture acceptance, including cancellation.
    let gamma = 128.0 * unit_roundoff / (1.0 - 128.0 * unit_roundoff);
    assert!(
        (actual - expected).abs() <= gamma * (1.0 + expected.abs()),
        "actual={actual:.17e} expected={expected:.17e}"
    );
}

#[test]
fn digamma_analytic_anchors_preserve_precision() {
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    let euler_gamma = 0.577_215_664_901_532_9;
    for (x, expected) in [
        (1.0, -euler_gamma),
        (0.5, -euler_gamma - 2.0 * core::f64::consts::LN_2),
        (2.0, 1.0 - euler_gamma),
        (-0.5, 2.0 - euler_gamma - 2.0 * core::f64::consts::LN_2),
    ] {
        close(FloatElement::digamma(x), expected, f64::EPSILON / 2.0);
        close(
            FloatElement::digamma(F64(x)).0,
            expected,
            f64::EPSILON / 2.0,
        );
        close(
            f64::from(FloatElement::digamma(x as f32)),
            expected,
            f64::from(f32::EPSILON) / 2.0,
        );
    }
}

#[test]
fn digamma_domain_and_signed_pole_limits() {
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    assert_eq!(FloatElement::digamma(0.0_f64), f64::NEG_INFINITY);
    assert_eq!(FloatElement::digamma(-0.0_f64), f64::INFINITY);
    assert_eq!(FloatElement::digamma(f64::INFINITY), f64::INFINITY);
    for x in [f64::NEG_INFINITY, f64::NAN, -1.0, -2.0, -1e30] {
        assert!(FloatElement::digamma(x).is_nan(), "{x}");
    }
    assert_eq!(FloatElement::digamma(f64::from_bits(1)), f64::NEG_INFINITY);
    assert_eq!(FloatElement::digamma(-f64::from_bits(1)), f64::INFINITY);
    assert!(NumericElement::is_finite(FloatElement::digamma(f64::MAX)));
}

#[test]
fn digamma_high_precision_binary_fixtures() {
    assert_eq!(
        super::digamma_fixtures::REFERENCES.len(),
        super::digamma_error_bounds::BOUNDS.len()
    );
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    for (&(x, expected), &(absolute, relative)) in super::digamma_fixtures::REFERENCES
        .iter()
        .zip(super::digamma_error_bounds::BOUNDS)
    {
        assert!(
            (FloatElement::digamma(x) - expected).abs() <= absolute + relative * expected.abs()
        );
        assert!(
            (FloatElement::digamma(F64(x)).0 - expected).abs()
                <= absolute + relative * expected.abs()
        );
        close(FloatElement::digamma(x), expected, f64::EPSILON / 2.0);
        close(
            FloatElement::digamma(F64(x)).0,
            expected,
            f64::EPSILON / 2.0,
        );
    }
}

#[test]
fn digamma_recurrence_and_reflection() {
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    for x in [
        0.125_f64, 0.5, 1.25, 3.25, 17.5, 1000.25, -0.25, -1.25, -15.75,
    ] {
        close(
            FloatElement::digamma(x + 1.0),
            FloatElement::digamma(x) + 1.0 / x,
            f64::EPSILON / 2.0,
        );
        close(
            FloatElement::digamma(1.0 - x) - FloatElement::digamma(x),
            core::f64::consts::PI / (core::f64::consts::PI * (x - x.round())).tan(),
            f64::EPSILON / 2.0,
        );
    }
}

fn reduced_boundary<T: FloatElement>(x: T) {
    let actual = FloatElement::digamma(x);
    let expected = T::from_f32(FloatElement::digamma(x.to_f32()));
    if expected.is_nan() {
        assert!(actual.is_nan());
    } else {
        assert_eq!(actual, expected, "input={x:?}");
        assert_eq!(actual.to_f32().to_bits(), expected.to_f32().to_bits());
    }
}

#[test]
fn digamma_reduced_storage_exhausts_encodings() {
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    use eunomia::{Bf16, Bf4, Bf8, F16, F4, F8};
    for bits in 0..=u16::MAX {
        reduced_boundary(F16(bits));
        reduced_boundary(Bf16(bits));
    }
    for bits in 0..=u8::MAX {
        reduced_boundary(F8(bits));
        reduced_boundary(Bf8(bits));
        if bits < 16 {
            reduced_boundary(F4(bits));
            reduced_boundary(Bf4(bits));
        }
    }
}

#[test]
fn digamma_single_precision_binary_fixtures_and_domain() {
    assert_eq!(
        super::digamma_fixtures::SINGLE_REFERENCES.len(),
        super::digamma_error_bounds::SINGLE_BOUNDS.len()
    );
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    for (&(x, expected), &(absolute, relative)) in super::digamma_fixtures::SINGLE_REFERENCES
        .iter()
        .zip(super::digamma_error_bounds::SINGLE_BOUNDS)
    {
        assert!(
            (f64::from(FloatElement::digamma(x as f32)) - expected).abs()
                <= absolute + relative * expected.abs()
        );
        let input = x as f32; // Fixture inputs are exactly representable in f32.
        close(
            f64::from(FloatElement::digamma(input)),
            expected,
            f64::from(f32::EPSILON) / 2.0,
        );
        assert_eq!(
            FloatElement::digamma(eunomia::F32(input)).0,
            FloatElement::digamma(input)
        );
    }
    assert_eq!(FloatElement::digamma(0.0_f32), f32::NEG_INFINITY);
    assert_eq!(FloatElement::digamma(-0.0_f32), f32::INFINITY);
    assert_eq!(FloatElement::digamma(f32::INFINITY), f32::INFINITY);
    for x in [f32::NAN, f32::NEG_INFINITY, -1.0, -2.0, -1e30] {
        assert!(FloatElement::digamma(x).is_nan());
    }
    assert_eq!(FloatElement::digamma(f32::from_bits(1)), f32::NEG_INFINITY);
    assert_eq!(FloatElement::digamma(-f32::from_bits(1)), f32::INFINITY);
}

fn primitive(actual: f64, high: f64, low: f64, ulp: f64, limit: f64) {
    // hi+lo resolves the exact reference past the target's significand.
    // Account for rounding low and the subtraction that measures the error.
    let subtraction = actual - high;
    let reference_error = f64::EPSILON * (low.abs() + subtraction.abs());
    assert!(
        (subtraction - low).abs() + reference_error <= limit * ulp,
        "primitive actual={actual:.17e} reference={high:.17e}+{low:.17e}"
    );
}

#[test]
fn digamma_primitive_rounding_premises_hold_at_fixture_arguments() {
    let _trace = trace();
    let _arrange = tracing::info_span!("arrange").entered();
    let _act = tracing::info_span!("act").entered();
    use super::digamma_primitive_fixtures as p;
    for (values, logarithm, limit) in [(p::LOG, true, 1.0), (p::TAN, false, 2.0)] {
        for &(argument, high, low) in values {
            let actual = if logarithm {
                FloatElement::ln(argument)
            } else {
                FloatElement::tan(argument)
            };
            let ulp = (high.abs().next_up() - high.abs()).abs();
            primitive(actual, high, low, ulp, limit);
        }
    }
    for (values, logarithm, limit) in [(p::SINGLE_LOG, true, 1.0), (p::SINGLE_TAN, false, 2.0)] {
        for &(argument, high, low) in values {
            let argument = argument as f32;
            let actual = if logarithm {
                FloatElement::ln(argument)
            } else {
                FloatElement::tan(argument)
            };
            let rounded = high as f32;
            let ulp = f64::from(rounded.abs().next_up() - rounded.abs());
            primitive(f64::from(actual), high, low, ulp, limit);
        }
    }
}
