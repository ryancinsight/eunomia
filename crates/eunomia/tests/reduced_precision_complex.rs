use core::mem::{align_of, size_of};

use eunomia::{Bf16, Complex, F16};

#[test]
fn reduced_precision_complex_types_are_compact_and_provider_owned() {
    assert_eq!(size_of::<Complex<F16>>(), 4);
    assert_eq!(align_of::<Complex<F16>>(), 2);
    assert_eq!(size_of::<Complex<Bf16>>(), 4);
    assert_eq!(align_of::<Complex<Bf16>>(), 2);
}

#[test]
fn reduced_precision_complex_arithmetic_uses_the_owned_components() {
    let f16_left = Complex::new(F16::from_f32(1.5), F16::from_f32(0.5));
    let f16_right = Complex::new(F16::from_f32(2.0), F16::from_f32(-1.0));
    let f16_product = f16_left * f16_right;
    assert_eq!(f16_product.re.to_bits(), F16::from_f32(3.5).to_bits());
    assert_eq!(f16_product.im.to_bits(), F16::from_f32(-0.5).to_bits());

    let bf16_left = Complex::new(Bf16::from_f32(1.5), Bf16::from_f32(0.5));
    let bf16_right = Complex::new(Bf16::from_f32(2.0), Bf16::from_f32(-1.0));
    let bf16_product = bf16_left * bf16_right;
    assert_eq!(bf16_product.re.to_bits(), Bf16::from_f32(3.5).to_bits());
    assert_eq!(bf16_product.im.to_bits(), Bf16::from_f32(-0.5).to_bits());
}
