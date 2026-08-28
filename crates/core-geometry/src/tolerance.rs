//! Explicit tolerance policy for robust geometry predicates.
//!
//! Per the frozen v1.1 domain model: "Geometry predicates that require robust
//! classification MUST use an explicit tolerance policy; tolerance is never
//! implicit or caller-chosen on a per-operation basis."
//!
//! This module provides exactly one tolerance type: [`Tolerance`]. All
//! predicate-style operations take a `Tolerance` by value; there is no implicit
//! default and no per-call hidden tolerance.

use crate::error::GeometryError;

/// A single, explicit absolute tolerance policy for robust classification.
///
/// The canonical default absolute tolerance is `1e-9`. Callers that need a
/// coarser tolerance (e.g. for low-resolution imported data) construct a
/// non-default `Tolerance` once per logical scope and pass it explicitly to
/// every predicate. There is no global mutable tolerance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    /// Absolute tolerance in canonical drawing units.
    pub absolute: f64,
}

impl Tolerance {
    /// Canonical default absolute tolerance: `1e-9` drawing units.
    ///
    /// Matches the frozen v1.1 contract: a single, fixed tolerance value is
    /// defined for the default case; deviations must be explicit.
    pub const DEFAULT: Self = Self { absolute: 1e-9_f64 };

    /// Construct a tolerance from a finite, strictly positive `absolute`
    /// value.
    ///
    /// Returns [`GeometryError::NonFinite`] for NaN/inf input and
    /// [`GeometryError::InvalidInput`] for non-positive values.
    #[must_use]
    pub fn new(absolute: f64) -> Result<Self, GeometryError> {
        if !absolute.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if absolute <= 0.0 {
            return Err(GeometryError::InvalidInput("tolerance must be > 0"));
        }
        Ok(Self { absolute })
    }

    /// Squared tolerance (for comparing squared distances without a sqrt).
    #[must_use]
    pub const fn squared(self) -> f64 {
        self.absolute * self.absolute
    }

    /// Squared tolerance used for "are these points coincident?" predicates.
    ///
    /// Equal to [`Self::squared`] by definition; provided as a named, semantic
    /// entry point so call sites read as `tolerance.coincident_squared()`.
    #[must_use]
    pub const fn coincident_squared(self) -> f64 {
        self.squared()
    }

    /// Returns `true` if `delta` is within `±absolute` of zero.
    #[must_use]
    pub fn is_zero(self, delta: f64) -> bool {
        delta.abs() <= self.absolute
    }

    /// Returns `true` if `a` and `b` are within `absolute` of each other.
    #[must_use]
    pub fn eq(self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.absolute
    }
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC02 — Predicates are deterministic (single tolerance policy).
    // Evidence: WO-002-AC03 — NaN/Inf rejection at constructor.
    use super::Tolerance;
    use crate::error::GeometryError;

    #[test]
    fn default_is_1e9() {
        assert_eq!(Tolerance::DEFAULT.absolute, 1e-9_f64);
        let t = Tolerance::default();
        assert_eq!(t, Tolerance::DEFAULT);
    }

    #[test]
    fn new_rejects_non_finite() {
        assert_eq!(
            Tolerance::new(f64::NAN).unwrap_err(),
            GeometryError::NonFinite
        );
        assert_eq!(
            Tolerance::new(f64::INFINITY).unwrap_err(),
            GeometryError::NonFinite
        );
        assert_eq!(
            Tolerance::new(f64::NEG_INFINITY).unwrap_err(),
            GeometryError::NonFinite
        );
    }

    #[test]
    fn new_rejects_non_positive() {
        assert!(Tolerance::new(0.0).is_err());
        assert!(Tolerance::new(-1e-12).is_err());
    }

    #[test]
    fn new_accepts_positive_finite() {
        let t = Tolerance::new(1e-6).unwrap();
        assert!((t.absolute - 1e-6).abs() < 1e-18);
    }

    #[test]
    fn squared_and_coincident_squared_match() {
        let t = Tolerance::new(1e-3).unwrap();
        assert!((t.squared() - 1e-6).abs() < 1e-18);
        assert_eq!(t.squared(), t.coincident_squared());
    }

    #[test]
    fn is_zero_and_eq_are_symmetric() {
        let t = Tolerance::new(1e-3).unwrap();
        assert!(t.is_zero(1e-4));
        assert!(!t.is_zero(2e-3));
        assert!(t.eq(1.0, 1.0 + 1e-4));
        assert!(t.eq(1.0 + 1e-4, 1.0));
        assert!(!t.eq(1.0, 1.0 + 2e-3));
    }
}
