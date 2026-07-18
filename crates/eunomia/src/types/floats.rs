use crate::convert::{narrow, narrow_finite, widen, widen_finite};

/// Transparent wrapper for half::f16.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F16(pub half::f16);

/// Transparent wrapper for f32.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F32(pub f32);

/// Transparent wrapper for f64.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F64(pub f64);

/// Transparent wrapper for half::bf16.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Bf16(pub half::bf16);

/// Brain Float 8: IEEE-style E5M2 with infinity and NaN.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Bf8(pub u8);

/// Brain Float 4: finite-only E2M1 with the top exponent reserved for NaN.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Bf4(pub u8);

/// Finite-only 8-bit float: E4M3 with the top exponent reserved for NaN.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F8(pub u8);

/// Finite-only 4-bit float: E3M0 with the top exponent reserved for NaN.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F4(pub u8);

impl Bf8 {
    /// Convert to `f32` exactly.
    #[inline]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(widen::<5, 2>(u32::from(self.0)))
    }

    /// Convert from `f32` with round-to-nearest, ties-to-even.
    #[inline]
    pub fn from_f32(val: f32) -> Self {
        Self(
            u8::try_from(narrow::<5, 2>(val.to_bits()))
                .expect("invariant: E5M2 encoding occupies exactly eight bits"),
        )
    }
}

impl Bf4 {
    /// Convert to `f32` exactly.
    #[inline]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(widen_finite::<2, 1>(u32::from(self.0)))
    }

    /// Convert from `f32` with round-to-nearest, ties-to-even.
    ///
    /// Infinity and finite overflow saturate to the signed maximum finite
    /// value. NaN maps to the canonical all-ones magnitude encoding.
    #[inline]
    pub fn from_f32(val: f32) -> Self {
        Self(
            u8::try_from(narrow_finite::<2, 1>(val.to_bits()))
                .expect("invariant: E2M1 encoding occupies four bits"),
        )
    }

    /// Pack two Bf4 values into one byte.
    #[inline]
    pub fn pack_pair(low: Self, high: Self) -> u8 {
        (low.0 & 0x0F) | ((high.0 & 0x0F) << 4)
    }

    /// Unpack one byte into two Bf4 values.
    #[inline]
    pub fn unpack_pair(packed: u8) -> (Self, Self) {
        (Self(packed & 0x0F), Self((packed >> 4) & 0x0F))
    }
}

impl F8 {
    /// Convert to `f32` exactly.
    #[inline]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(widen_finite::<4, 3>(u32::from(self.0)))
    }

    /// Convert from `f32` with round-to-nearest, ties-to-even.
    ///
    /// Infinity and finite overflow saturate to the signed maximum finite
    /// value. NaN maps to the canonical all-ones magnitude encoding.
    #[inline]
    pub fn from_f32(val: f32) -> Self {
        Self(
            u8::try_from(narrow_finite::<4, 3>(val.to_bits()))
                .expect("invariant: E4M3 encoding occupies exactly eight bits"),
        )
    }
}

impl F4 {
    /// Convert to `f32` exactly.
    #[inline]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(widen_finite::<3, 0>(u32::from(self.0)))
    }

    /// Convert from `f32` with round-to-nearest, ties-to-even.
    ///
    /// Infinity and finite overflow saturate to the signed maximum finite
    /// value. NaN maps to the reserved top exponent.
    #[inline]
    pub fn from_f32(val: f32) -> Self {
        Self(
            u8::try_from(narrow_finite::<3, 0>(val.to_bits()))
                .expect("invariant: E3M0 encoding occupies four bits"),
        )
    }

    /// Pack two F4 values into one byte.
    #[inline]
    pub fn pack_pair(low: Self, high: Self) -> u8 {
        (low.0 & 0x0F) | ((high.0 & 0x0F) << 4)
    }

    /// Unpack one byte into two F4 values.
    #[inline]
    pub fn unpack_pair(packed: u8) -> (Self, Self) {
        (Self(packed & 0x0F), Self((packed >> 4) & 0x0F))
    }
}
