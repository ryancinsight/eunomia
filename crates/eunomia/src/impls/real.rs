//! `RealElement` impls for every shipped element type except `Complex`.

use crate::traits::RealElement;
use crate::types::{Bf16, Bf4, Bf8, F16, F32, F4, F64, F8, I16, I32, I8};

impl RealElement for f32 {}
impl RealElement for f64 {}
impl RealElement for i8 {}
impl RealElement for i16 {}
impl RealElement for i32 {}
impl RealElement for i64 {}
impl RealElement for isize {}
impl RealElement for u8 {}
impl RealElement for u16 {}
impl RealElement for u32 {}
impl RealElement for u64 {}
impl RealElement for usize {}
impl RealElement for F16 {}
impl RealElement for Bf16 {}
impl RealElement for F32 {}
impl RealElement for F64 {}
impl RealElement for Bf8 {}
impl RealElement for Bf4 {}
impl RealElement for F8 {}
impl RealElement for F4 {}
impl RealElement for I8 {}
impl RealElement for I16 {}
impl RealElement for I32 {}
