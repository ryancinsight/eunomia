//! The integer oracle: the correctly rounded encoding of an `f64` in a reduced
//! format, computed from the exact significand in integer arithmetic with no
//! `f32` intermediate.

use alloc::vec::Vec;

/// A reduced binary float format `[sign | exponent | fraction]`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Format {
    pub(super) exponent_bits: i32,
    pub(super) fraction_bits: i32,
    /// Whether overflow saturates to the largest finite value (the top
    /// exponent is NaN-only) rather than rounding to infinity.
    pub(super) saturates: bool,
}

pub(super) const F16_FORMAT: Format = Format {
    exponent_bits: 5,
    fraction_bits: 10,
    saturates: false,
};
pub(super) const BF16_FORMAT: Format = Format {
    exponent_bits: 8,
    fraction_bits: 7,
    saturates: false,
};
pub(super) const BF8_FORMAT: Format = Format {
    exponent_bits: 5,
    fraction_bits: 2,
    saturates: false,
};
pub(super) const F8_FORMAT: Format = Format {
    exponent_bits: 4,
    fraction_bits: 3,
    saturates: true,
};
pub(super) const BF4_FORMAT: Format = Format {
    exponent_bits: 2,
    fraction_bits: 1,
    saturates: true,
};
pub(super) const F4_FORMAT: Format = Format {
    exponent_bits: 3,
    fraction_bits: 0,
    saturates: true,
};

pub(super) const F64_FRACTION_BITS: i32 = 52;
pub(super) const F64_FRACTION_MASK: u64 = (1 << F64_FRACTION_BITS) - 1;

/// `2^exponent` for an exponent inside `f64`'s normal range.
pub(super) fn pow2(exponent: i32) -> f64 {
    let field = u64::try_from(exponent + 1023).expect("invariant: the exponent is normal in f64");
    f64::from_bits(field << F64_FRACTION_BITS)
}

impl Format {
    pub(super) fn bias(self) -> i32 {
        (1 << (self.exponent_bits - 1)) - 1
    }

    pub(super) fn top_field(self) -> u32 {
        (1 << self.exponent_bits) - 1
    }

    fn fraction_mask(self) -> u32 {
        (1 << self.fraction_bits) - 1
    }

    /// The encoding that an overflowing magnitude rounds to.
    fn overflow(self) -> u32 {
        if self.saturates {
            ((self.top_field() - 1) << self.fraction_bits) | self.fraction_mask()
        } else {
            self.top_field() << self.fraction_bits
        }
    }

    /// The value of the finite non-negative encoding `field | fraction`.
    fn magnitude(self, field: u32, fraction: u32) -> f64 {
        let significand = f64::from(fraction);
        let min_exponent = 1 - self.bias();
        if field == 0 {
            significand * pow2(min_exponent - self.fraction_bits)
        } else {
            let exponent = i32::try_from(field).expect("invariant: the field is small")
                - self.bias()
                - self.fraction_bits;
            (f64::from(1_u32 << self.fraction_bits) + significand) * pow2(exponent)
        }
    }

    /// Every finite non-negative value in increasing order, then the value the
    /// next encoding would hold were the top exponent finite: the upper edge
    /// of the overflow midpoint.
    pub(super) fn magnitudes(self) -> Vec<f64> {
        let mut values = Vec::new();
        for field in 0..self.top_field() {
            for fraction in 0..=self.fraction_mask() {
                values.push(self.magnitude(field, fraction));
            }
        }
        values.push(self.magnitude(self.top_field(), 0));
        values
    }

    /// The correctly rounded encoding of `x`, nearest with ties to even,
    /// computed from the exact `f64` significand in integer arithmetic.
    /// `None` for NaN.
    pub(super) fn round(self, x: f64) -> Option<u32> {
        let bits = x.to_bits();
        let sign = if bits >> 63 == 1 {
            1_u32 << (self.exponent_bits + self.fraction_bits)
        } else {
            0
        };
        let field = (bits >> F64_FRACTION_BITS) & 0x7FF;
        let fraction = bits & F64_FRACTION_MASK;
        if field == 0x7FF {
            return (fraction == 0).then_some(sign | self.overflow());
        }
        let (significand, scale) = if field == 0 {
            (fraction, -1074)
        } else {
            (
                fraction | (1 << F64_FRACTION_BITS),
                i32::try_from(field).expect("invariant: eleven bits") - 1075,
            )
        };
        if significand == 0 {
            return Some(sign);
        }

        // `x = significand * 2^scale`; the target quantum is `2^quantum`.
        let width =
            i32::try_from(u64::BITS - significand.leading_zeros()).expect("invariant: at most 64");
        let exponent = width - 1 + scale;
        let quantum = exponent.max(1 - self.bias()) - self.fraction_bits;
        let shift = u32::try_from(quantum - scale)
            .expect("invariant: an f64 significand outwidths every target");
        let units = if shift >= u128::BITS {
            0
        } else {
            let exact = u128::from(significand);
            let kept = exact >> shift;
            let discarded = exact & ((1_u128 << shift) - 1);
            let half = 1_u128 << (shift - 1);
            kept + u128::from(discarded > half || (discarded == half && kept & 1 == 1))
        };
        let units = u32::try_from(units).expect("invariant: at most 2^(M + 1)");
        if units == 0 {
            return Some(sign);
        }
        let (units, quantum) = if units == 1 << (self.fraction_bits + 1) {
            (units >> 1, quantum + 1)
        } else {
            (units, quantum)
        };
        let (field, fraction) = if units < 1 << self.fraction_bits {
            (0, units)
        } else {
            let field = quantum + self.fraction_bits + self.bias();
            (field, units - (1 << self.fraction_bits))
        };
        let top = i32::try_from(self.top_field()).expect("invariant: small");
        if field >= top {
            return Some(sign | self.overflow());
        }
        let field = u32::try_from(field).expect("invariant: the field is non-negative");
        Some(sign | (field << self.fraction_bits) | fraction)
    }
}

/// `value` displaced by `offset` units in the last place, in magnitude.
pub(super) fn displaced(value: f64, offset: i64) -> f64 {
    let bits = i64::try_from(value.to_bits()).expect("invariant: a magnitude's sign bit is clear");
    f64::from_bits(
        u64::try_from(bits + offset).expect("invariant: the displacement stays positive"),
    )
}
