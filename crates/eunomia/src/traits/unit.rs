//! Scalar conversion required by multiplicative physical units.

/// A provider scalar that can be scaled by an SI-unit coefficient.
///
/// The conversion is intentionally owned by Eunomia so downstream physical
/// quantity crates use one provider-defined path for real storage types and
/// complex phasors. A complex value is scaled componentwise; its imaginary
/// component is quadrature, not a second physical unit.
pub trait UnitScalar: Copy {
    /// Scale this value by a real coefficient in the scalar's native precision.
    fn scale_by_f64(self, factor: f64) -> Self;
}
