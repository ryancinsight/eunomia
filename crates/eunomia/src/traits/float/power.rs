use super::NumericElement;

/// `base^exp` for unsigned `exp` by squaring: the shared kernel of
/// [`super::FloatElement::powi`]'s upward pass and its underflow-preserving
/// downward recomputation.
#[inline(always)]
pub(super) fn powi_positive<T: NumericElement>(mut base: T, mut exp: u32) -> T {
    let mut acc = T::ONE;
    while exp > 0 {
        if exp & 1 == 1 {
            acc *= base;
        }
        base *= base;
        exp >>= 1;
    }
    acc
}
