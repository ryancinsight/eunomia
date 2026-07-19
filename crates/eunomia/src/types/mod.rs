mod complex;
mod floats;
mod ints;

pub use complex::{Complex, Complex32, Complex64};
pub use floats::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8};
pub use ints::{I16, I32, I8};

// SAFETY: `Complex<T>` is `#[repr(C)]` with two `T` fields, so it is zeroable
// and plain-old-data exactly when `T` is.
unsafe impl<T: bytemuck::Zeroable> bytemuck::Zeroable for Complex<T> {}
unsafe impl<T: bytemuck::Pod> bytemuck::Pod for Complex<T> {}

const _: () = {
    assert!(core::mem::size_of::<Complex32>() == 2 * core::mem::size_of::<f32>());
    assert!(core::mem::align_of::<Complex32>() == core::mem::align_of::<f32>());
    assert!(core::mem::offset_of!(Complex32, re) == 0);
    assert!(core::mem::offset_of!(Complex32, im) == core::mem::size_of::<f32>());

    assert!(core::mem::size_of::<Complex64>() == 2 * core::mem::size_of::<f64>());
    assert!(core::mem::align_of::<Complex64>() == core::mem::align_of::<f64>());
    assert!(core::mem::offset_of!(Complex64, re) == 0);
    assert!(core::mem::offset_of!(Complex64, im) == core::mem::size_of::<f64>());
};

// SAFETY: every wrapper is `#[repr(transparent)]` over a type that is itself
// `Pod`/`Zeroable` — `f32`/`f64`/`i8`/`i16`/`i32`, a `u16` for `F16`/`Bf16`, or
// a `u8` for the sub-byte formats (all bit patterns are valid encodings). None
// carry padding or invalid bit patterns, and the `const _` block below pins each
// type's size and alignment.
unsafe impl bytemuck::Zeroable for F16 {}
unsafe impl bytemuck::Pod for F16 {}
unsafe impl bytemuck::Zeroable for F32 {}
unsafe impl bytemuck::Pod for F32 {}
unsafe impl bytemuck::Zeroable for F64 {}
unsafe impl bytemuck::Pod for F64 {}
unsafe impl bytemuck::Zeroable for Bf16 {}
unsafe impl bytemuck::Pod for Bf16 {}
unsafe impl bytemuck::Zeroable for Bf8 {}
unsafe impl bytemuck::Pod for Bf8 {}
unsafe impl bytemuck::Zeroable for Bf4 {}
unsafe impl bytemuck::Pod for Bf4 {}
unsafe impl bytemuck::Zeroable for F8 {}
unsafe impl bytemuck::Pod for F8 {}
unsafe impl bytemuck::Zeroable for F4 {}
unsafe impl bytemuck::Pod for F4 {}
unsafe impl bytemuck::Zeroable for I8 {}
unsafe impl bytemuck::Pod for I8 {}
unsafe impl bytemuck::Zeroable for I16 {}
unsafe impl bytemuck::Pod for I16 {}
unsafe impl bytemuck::Zeroable for I32 {}
unsafe impl bytemuck::Pod for I32 {}

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
