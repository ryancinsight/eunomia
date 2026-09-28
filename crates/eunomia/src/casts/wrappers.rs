use crate::traits::{CastFrom, FloatElement, NumericElement};
use crate::types::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8, I16, I32, I8};

macro_rules! impl_cast_between {
    ($src:ident, $dst:ident) => {
        impl CastFrom<$src> for $dst {
            #[inline(always)]
            fn cast_from(val: $src) -> Self {
                <$dst as FloatElement>::from_f64(val.to_f64())
            }
        }
    };
}

// Float-to-Float casts
impl_cast_between!(F16, F32);
impl_cast_between!(F16, F64);
impl_cast_between!(F16, Bf16);
impl_cast_between!(F16, Bf8);
impl_cast_between!(F16, Bf4);
impl_cast_between!(F16, F8);
impl_cast_between!(F16, F4);
impl_cast_between!(F32, F16);
impl_cast_between!(F32, F64);
impl_cast_between!(F32, Bf16);
impl_cast_between!(F32, Bf8);
impl_cast_between!(F32, Bf4);
impl_cast_between!(F32, F8);
impl_cast_between!(F32, F4);
impl_cast_between!(F64, F16);
impl_cast_between!(F64, F32);
impl_cast_between!(F64, Bf16);
impl_cast_between!(F64, Bf8);
impl_cast_between!(F64, Bf4);
impl_cast_between!(F64, F8);
impl_cast_between!(F64, F4);
impl_cast_between!(Bf16, F16);
impl_cast_between!(Bf16, F32);
impl_cast_between!(Bf16, F64);
impl_cast_between!(Bf16, Bf8);
impl_cast_between!(Bf16, Bf4);
impl_cast_between!(Bf16, F8);
impl_cast_between!(Bf16, F4);
impl_cast_between!(Bf8, F16);
impl_cast_between!(Bf8, F32);
impl_cast_between!(Bf8, F64);
impl_cast_between!(Bf8, Bf16);
impl_cast_between!(Bf8, Bf4);
impl_cast_between!(Bf8, F8);
impl_cast_between!(Bf8, F4);
impl_cast_between!(Bf4, F16);
impl_cast_between!(Bf4, F32);
impl_cast_between!(Bf4, F64);
impl_cast_between!(Bf4, Bf16);
impl_cast_between!(Bf4, Bf8);
impl_cast_between!(Bf4, F8);
impl_cast_between!(Bf4, F4);
impl_cast_between!(F8, F16);
impl_cast_between!(F8, F32);
impl_cast_between!(F8, F64);
impl_cast_between!(F8, Bf16);
impl_cast_between!(F8, Bf8);
impl_cast_between!(F8, Bf4);
impl_cast_between!(F8, F4);
impl_cast_between!(F4, F16);
impl_cast_between!(F4, F32);
impl_cast_between!(F4, F64);
impl_cast_between!(F4, Bf16);
impl_cast_between!(F4, Bf8);
impl_cast_between!(F4, Bf4);
impl_cast_between!(F4, F8);

// Identity casts
macro_rules! impl_identity_cast {
    ($t:ident) => {
        impl CastFrom<$t> for $t {
            #[inline(always)]
            fn cast_from(val: $t) -> Self {
                val
            }
        }
    };
}
impl_identity_cast!(F16);
impl_identity_cast!(F32);
impl_identity_cast!(F64);
impl_identity_cast!(Bf16);
impl_identity_cast!(Bf8);
impl_identity_cast!(Bf4);
impl_identity_cast!(F8);
impl_identity_cast!(F4);
impl_identity_cast!(I8);
impl_identity_cast!(I16);
impl_identity_cast!(I32);

// Int-to-Int casts
macro_rules! impl_int_to_int {
    ($src:ident, $dst:ident) => {
        impl CastFrom<$src> for $dst {
            #[inline(always)]
            fn cast_from(val: $src) -> Self {
                Self(val.0 as _)
            }
        }
    };
}
impl_int_to_int!(I8, I16);
impl_int_to_int!(I8, I32);
impl_int_to_int!(I16, I8);
impl_int_to_int!(I16, I32);
impl_int_to_int!(I32, I8);
impl_int_to_int!(I32, I16);
