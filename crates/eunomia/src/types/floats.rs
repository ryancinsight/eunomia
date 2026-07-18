use crate::convert::{narrow, narrow_finite, widen, widen_finite};

/// IEEE 754 binary16 (half precision), stored as its raw `u16` bit pattern.
///
/// Conversions run through the native [`convert`](crate::convert) kernel;
/// `PartialEq`/`PartialOrd` are float-semantic (via `f32`), not bitwise.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct F16(pub u16);

/// Transparent wrapper for f32.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F32(pub f32);

/// Transparent wrapper for f64.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
#[repr(transparent)]
pub struct F64(pub f64);

/// bfloat16 (E8M7), stored as its raw `u16` bit pattern.
///
/// Conversions run through the native [`convert`](crate::convert) kernel;
/// `PartialEq`/`PartialOrd` are float-semantic (via `f32`), not bitwise.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct Bf16(pub u16);

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

impl F16 {
    /// The value `+0.0`.
    pub const ZERO: Self = Self(0x0000);
    /// The value `1.0`.
    pub const ONE: Self = Self(0x3C00);
    /// A quiet not-a-number.
    pub const NAN: Self = Self(0x7E00);
    /// Positive infinity.
    pub const INFINITY: Self = Self(0x7C00);
    /// Negative infinity.
    pub const NEG_INFINITY: Self = Self(0xFC00);

    /// Widen to `f32` (exact).
    #[inline]
    #[must_use]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(widen::<5, 10>(self.0 as u32))
    }
    /// Narrow from `f32`, rounding to nearest with ties to even.
    #[inline]
    #[must_use]
    pub fn from_f32(value: f32) -> Self {
        Self(narrow::<5, 10>(value.to_bits()) as u16)
    }
    /// Narrow from `f64` via `f32` — exact, since `f32`'s 24-bit significand
    /// meets the `2·11 + 2` bits the double-rounding theorem requires for binary16.
    #[inline]
    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        Self::from_f32(value as f32)
    }
    /// Whether `self` is finite (neither infinite nor NaN).
    #[inline]
    #[must_use]
    pub fn is_finite(self) -> bool {
        (self.0 & 0x7C00) != 0x7C00
    }
    /// Whether `self` is NaN.
    #[inline]
    #[must_use]
    pub fn is_nan(self) -> bool {
        (self.0 & 0x7C00) == 0x7C00 && (self.0 & 0x03FF) != 0
    }
}

impl PartialEq for F16 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_f32() == other.to_f32()
    }
}

impl PartialOrd for F16 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_f32().partial_cmp(&other.to_f32())
    }
}

impl core::fmt::Debug for F16 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "F16({})", self.to_f32())
    }
}

impl Bf16 {
    /// The value `+0.0`.
    pub const ZERO: Self = Self(0x0000);
    /// The value `1.0`.
    pub const ONE: Self = Self(0x3F80);
    /// A quiet not-a-number.
    pub const NAN: Self = Self(0x7FC0);
    /// Positive infinity.
    pub const INFINITY: Self = Self(0x7F80);
    /// Negative infinity.
    pub const NEG_INFINITY: Self = Self(0xFF80);

    /// Widen to `f32` (exact).
    #[inline]
    #[must_use]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(widen::<8, 7>(self.0 as u32))
    }
    /// Narrow from `f32`, rounding to nearest with ties to even.
    #[inline]
    #[must_use]
    pub fn from_f32(value: f32) -> Self {
        Self(narrow::<8, 7>(value.to_bits()) as u16)
    }
    /// Narrow from `f64` via `f32` — exact for bfloat16 (`f32` exceeds the
    /// `2·8 + 2` bits double rounding requires).
    #[inline]
    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        Self::from_f32(value as f32)
    }
    /// Whether `self` is finite (neither infinite nor NaN).
    #[inline]
    #[must_use]
    pub fn is_finite(self) -> bool {
        (self.0 & 0x7F80) != 0x7F80
    }
    /// Whether `self` is NaN.
    #[inline]
    #[must_use]
    pub fn is_nan(self) -> bool {
        (self.0 & 0x7F80) == 0x7F80 && (self.0 & 0x007F) != 0
    }
}

impl PartialEq for Bf16 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_f32() == other.to_f32()
    }
}

impl PartialOrd for Bf16 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_f32().partial_cmp(&other.to_f32())
    }
}

impl core::fmt::Debug for Bf16 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Bf16({})", self.to_f32())
    }
}

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
