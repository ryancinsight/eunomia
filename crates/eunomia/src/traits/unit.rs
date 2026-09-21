//! Scalar conversion required by multiplicative physical units.

/// A provider scalar that can be scaled by an SI-unit coefficient.
///
/// The conversion is intentionally owned by Eunomia so downstream physical
/// quantity crates use one provider-defined path for real storage types and
/// complex phasors. A complex value is scaled componentwise; its imaginary
/// component is quadrature, not a second physical unit.
///
/// # Migration
///
/// `divide_by_f64` is a required method, so external `UnitScalar`
/// implementations must add direct native-precision division by the converted
/// coefficient. Implementing it as reciprocal scaling is not equivalent for
/// coefficients whose reciprocal overflows.
pub trait UnitScalar: Copy {
    /// Scale this value by a real coefficient in the scalar's native precision.
    fn scale_by_f64(self, factor: f64) -> Self;

    /// Divide this value by a real coefficient in the scalar's native precision.
    ///
    /// The division is evaluated directly rather than by multiplying by the
    /// coefficient's reciprocal, which can overflow before it is applied to a
    /// small value.
    fn divide_by_f64(self, factor: f64) -> Self;
}
