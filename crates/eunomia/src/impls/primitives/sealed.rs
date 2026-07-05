//! `private::Sealed` impls for the primitive scalar types.

use crate::traits::private;

impl private::Sealed for f32 {}
impl private::Sealed for f64 {}
impl private::Sealed for half::f16 {}
impl private::Sealed for half::bf16 {}
impl private::Sealed for i8 {}
impl private::Sealed for i16 {}
impl private::Sealed for i32 {}
impl private::Sealed for i64 {}
impl private::Sealed for isize {}
impl private::Sealed for u8 {}
impl private::Sealed for u16 {}
impl private::Sealed for u32 {}
impl private::Sealed for u64 {}
impl private::Sealed for usize {}
