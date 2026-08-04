//! Compare accumulation error and storage cost across Eunomia precisions.

use eunomia::{Bf16, FloatElement, NumericElement, F16, F32, F64};

fn accumulated_sum<T: FloatElement>(terms: usize) -> f64 {
    let mut sum = <T as NumericElement>::ZERO;
    for index in 1..=terms {
        let term = <T as FloatElement>::from_f64(1.0 / index as f64);
        sum += term;
    }
    <T as NumericElement>::to_f64(sum)
}

fn report<T: FloatElement>(name: &str, reference: f64, terms: usize, tolerance: f64) {
    let sum = accumulated_sum::<T>(terms);
    let absolute_error = (sum - reference).abs();
    let relative_error = absolute_error / reference.abs();
    eunomia::assert_relative_eq!(sum, reference, max_relative = tolerance);
    println!(
        "{name:>4}: {} bytes, sum = {sum:.9}, relative error = {relative_error:.3e}",
        core::mem::size_of::<T>()
    );
}

fn main() {
    let terms = 256;
    let reference = accumulated_sum::<F64>(terms);

    println!("harmonic sum of the first {terms} terms");
    report::<F64>("F64", reference, terms, 1.0e-12);
    report::<F32>("F32", reference, terms, 1.0e-5);
    report::<F16>("F16", reference, terms, 5.0e-2);
    report::<Bf16>("Bf16", reference, terms, 2.0e-1);

    // Construction through FloatElement is precision-correct and the reference
    // remains the double-precision value used by the comparison below.
    let one = <F16 as FloatElement>::from_f64(1.0);
    eunomia::assert_relative_eq!(one.to_f32(), 1.0_f32);
}
