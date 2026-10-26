//! Native-precision real digamma (DLMF 5.5.2, 5.5.4 and 5.11.2).
use super::FloatElement;

/// Shared scalar algorithm. Only native `f32`, `f64` and `F64` instantiate it;
/// reduced formats select the documented `f32` evaluation boundary explicitly.
///
/// Recurrence raises a positive argument to at least sixteen. After eight
/// Bernoulli terms the positive-ray remainder is smaller than the first
/// omitted term: |B18|/(18*16^18) < 3e-21. Horner evaluation uses native FMA.
#[inline]
pub(crate) fn evaluate<T: FloatElement>(input: T) -> T {
    if input.is_nan() || input == T::INFINITY {
        return input;
    }
    if input == T::ZERO {
        // Dividing before negating preserves the sign of the zero argument.
        return (T::ZERO - T::ONE) / input;
    }
    if !input.is_finite() {
        return T::NAN;
    }
    let half = T::from_f64(0.5);
    let (mut x, reflection) = if input < T::ZERO {
        // Subtract before multiplying by pi. For large negative inputs this
        // subtraction is exact (Sterbenz), unlike argument reduction of pi*x.
        let fraction = input - input.round();
        if fraction == T::ZERO {
            return T::NAN;
        }
        let pi = T::from_f64(core::f64::consts::PI);
        let cotangent = if fraction.abs() == half {
            T::ZERO
        } else {
            pi / (pi * fraction).tan()
        };
        (T::ONE - input, cotangent)
    } else {
        (input, T::ZERO)
    };
    let mut recurrence = T::ZERO;
    while x < T::from_f64(16.0) {
        recurrence += T::ONE / x;
        x += T::ONE;
    }
    // Form the reciprocal first: x*x would overflow on valid large inputs.
    let inverse = T::ONE / x;
    let square = inverse * inverse;
    // B_(2k)/(2k), descending k=8..1. These are mathematical coefficients,
    // converted once to the declared scalar precision, not widened inputs.
    let mut series = T::from_f64(-3617.0 / 8160.0);
    for coefficient in [
        1.0 / 12.0,
        -691.0 / 32760.0,
        1.0 / 132.0,
        -1.0 / 240.0,
        1.0 / 252.0,
        -1.0 / 120.0,
        1.0 / 12.0,
    ] {
        series = series.scalar_fmadd(square, T::from_f64(coefficient));
    }
    ((x.ln() - half * inverse) - square * series) - recurrence - reflection
}
