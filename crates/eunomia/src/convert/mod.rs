//! Native soft-float conversion SSOT.
//!
//! One generic const-parameterized kernel ([`narrow`]/[`widen`]) converts
//! between `f32` and any reduced IEEE-754 binary format `(E, M)` — exponent
//! width `E`, mantissa width `M` — with round-to-nearest, ties-to-even. It is
//! eunomia's replacement for the `half` crate's `f16`/`bf16` conversions and the
//! future single home for the sub-byte formats' conversions, so every precision
//! shares one authoritative, monomorphized implementation.
//!
//! Format parameters for the reduced types eunomia ships:
//!
//! | Format | `E` | `M` | Bias |
//! | --- | --- | --- | --- |
//! | `binary16` (`F16`) | 5 | 10 | 15 |
//! | `bfloat16` (`Bf16`) | 8 | 7 | 127 |
//!
//! ```
//! use eunomia::convert::{narrow, widen};
//! // binary16 round-trips 1.0 exactly.
//! let one_f16 = narrow::<5, 10>(1.0f32.to_bits());
//! assert_eq!(one_f16, 0x3C00);
//! assert_eq!(f32::from_bits(widen::<5, 10>(one_f16)), 1.0);
//! ```

mod bulk;
mod kernel;

pub(crate) use bulk::{narrow_bf16, narrow_f16, widen_bf16, widen_f16};
pub use kernel::{narrow, widen};
pub(crate) use kernel::{narrow_finite, widen_finite, widen_finite_high_word, widen_high_word};
