mod complex;
mod floats;
mod ints;

pub use complex::{Complex, Complex32, Complex64};
pub use floats::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8};
pub use ints::{I16, I32, I8};

const _: () = {
    assert!(core::mem::size_of::<Complex32>() == 2 * core::mem::size_of::<f32>());
    assert!(core::mem::align_of::<Complex32>() == core::mem::align_of::<f32>());
    assert!(core::mem::offset_of!(Complex32, re) == 0);
    assert!(core::mem::offset_of!(Complex32, im) == core::mem::size_of::<f32>());

    assert!(core::mem::size_of::<Complex64>() == 2 * core::mem::size_of::<f64>());
    assert!(core::mem::align_of::<Complex64>() == core::mem::align_of::<f64>());
    assert!(core::mem::offset_of!(Complex64, re) == 0);
    assert!(core::mem::offset_of!(Complex64, im) == core::mem::size_of::<f64>());

    assert!(core::mem::size_of::<Complex<F16>>() == 2 * core::mem::size_of::<F16>());
    assert!(core::mem::align_of::<Complex<F16>>() == core::mem::align_of::<F16>());
    assert!(core::mem::offset_of!(Complex<F16>, re) == 0);
    assert!(core::mem::offset_of!(Complex<F16>, im) == core::mem::size_of::<F16>());

    assert!(core::mem::size_of::<Complex<Bf16>>() == 2 * core::mem::size_of::<Bf16>());
    assert!(core::mem::align_of::<Complex<Bf16>>() == core::mem::align_of::<Bf16>());
    assert!(core::mem::offset_of!(Complex<Bf16>, re) == 0);
    assert!(core::mem::offset_of!(Complex<Bf16>, im) == core::mem::size_of::<Bf16>());
};

const _: () = {
    assert!(core::mem::size_of::<F16>() == 2);
    assert!(core::mem::align_of::<F16>() == 2);
    assert!(core::mem::size_of::<F32>() == 4);
    assert!(core::mem::align_of::<F32>() == 4);
    assert!(core::mem::size_of::<F64>() == 8);
    assert!(core::mem::align_of::<F64>() == 8);
    assert!(core::mem::size_of::<Bf16>() == 2);
    assert!(core::mem::align_of::<Bf16>() == 2);
    assert!(core::mem::size_of::<Bf8>() == 1);
    assert!(core::mem::align_of::<Bf8>() == 1);
    assert!(core::mem::size_of::<Bf4>() == 1);
    assert!(core::mem::align_of::<Bf4>() == 1);
    assert!(core::mem::size_of::<F8>() == 1);
    assert!(core::mem::align_of::<F8>() == 1);
    assert!(core::mem::size_of::<F4>() == 1);
    assert!(core::mem::align_of::<F4>() == 1);
    assert!(core::mem::size_of::<I8>() == 1);
    assert!(core::mem::align_of::<I8>() == 1);
    assert!(core::mem::size_of::<I16>() == 2);
    assert!(core::mem::align_of::<I16>() == 2);
    assert!(core::mem::size_of::<I32>() == 4);
    assert!(core::mem::align_of::<I32>() == 4);
};
