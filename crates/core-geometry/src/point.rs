//! 2D points (located positions in the canonical drawing plane).
//!
//! `Point2` is a `Copy` stack value type — no heap allocation, per
//! architecture §6 ("Geometry calculations on the hot path must avoid
//! unnecessary heap allocations").

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::ops::{Bounded2, Contains2, DistanceTo2, Project2, Transformable2, Validate};
use crate::tolerance::Tolerance;
use crate::transform::Transform2D;
use crate::vector::Vector2;
use serde::{Deserialize, Serialize};

/// A 2D point with finite `f64` coordinates. NaN/Inf are rejected at
/// construction; deserialized points MUST be [`Validate::validate`]d at
/// canonical-model boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

impl Point2 {
    /// Construct a point with finite coordinates.
    ///
    /// Returns [`GeometryError::NonFinite`] if either component is NaN or
    /// ±infinity.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Result<Self, GeometryError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        Ok(Self { x, y })
    }

    /// Construct a point WITHOUT validating finiteness. Crate-internal helper
    /// for use where the caller has already validated inputs.
    #[must_use]
    pub(crate) const fn new_unchecked(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// The origin `(0, 0)`.
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// Returns the origin (alias for [`Self::ORIGIN`] as a constructor
    /// function — convenient for chaining).
    #[must_use]
    pub const fn origin() -> Self {
        Self::ORIGIN
    }

    /// Displacement vector from this point to `rhs` (`rhs - self`).
    #[must_use]
    pub const fn vector_to(self, rhs: Self) -> Vector2 {
        Vector2::new_unchecked(rhs.x - self.x, rhs.y - self.y)
    }

    /// Displacement vector from the origin to this point (`self - origin`).
    #[must_use]
    pub const fn to_vector(self) -> Vector2 {
        Vector2::new_unchecked(self.x, self.y)
    }

    /// Translate this point by `delta`.
    #[must_use]
    pub const fn add(self, delta: Vector2) -> Self {
        Self::new_unchecked(self.x + delta.x, self.y + delta.y)
    }

    /// Translate this point by `-delta`.
    #[must_use]
    pub const fn sub(self, delta: Vector2) -> Self {
        Self::new_unchecked(self.x - delta.x, self.y - delta.y)
    }

    /// Squared distance to `rhs` (never allocates, never fails).
    #[must_use]
    pub const fn distance_squared_to(self, rhs: Self) -> f64 {
        let dx = self.x - rhs.x;
        let dy = self.y - rhs.y;
        dx * dx + dy * dy
    }

    /// Euclidean distance to `rhs`.
    #[must_use]
    pub fn distance_to(self, rhs: Self) -> f64 {
        self.distance_squared_to(rhs).sqrt()
    }

    /// Linear interpolation between `self` and `rhs` at parameter `t` in
    /// `[0, 1]` (clamped). Useful for tests and for tessellation inputs.
    #[must_use]
    pub fn lerp(self, rhs: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new_unchecked(self.x + (rhs.x - self.x) * t, self.y + (rhs.y - self.y) * t)
    }
}

impl Validate for Point2 {
    fn validate(&self) -> Result<(), GeometryError> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        Ok(())
    }
}

impl Bounded2 for Point2 {
    /// A single point's AABB is the degenerate box `[p, p]`.
    fn bounding_box(&self) -> BoundingBox2 {
        // Safety: a point's min == max is a degenerate but valid box.
        BoundingBox2::new_unchecked(*self, *self)
    }
}

impl Transformable2 for Point2 {
    fn transform(&self, transform: &Transform2D, _tol: Tolerance) -> Result<Self, GeometryError> {
        // The image of a point under an affine transform is always a point.
        Ok(transform.apply_point(self))
    }
}

impl DistanceTo2<Point2> for Point2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        Point2::distance_to(*self, *rhs)
    }
}

impl Project2 for Point2 {
    /// Projecting a point onto a point returns the point itself.
    fn project_point(&self, _point: &Point2, _tol: Tolerance) -> Point2 {
        *self
    }
}

impl Contains2<Point2> for Point2 {
    /// A point "contains" another point iff they are coincident within `tol`.
    fn contains(&self, rhs: &Point2, tol: Tolerance) -> bool {
        self.distance_squared_to(*rhs) <= tol.coincident_squared()
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Point2 serde round-trip.
    // Evidence: WO-002-AC02 — distance to point determinism.
    // Evidence: WO-002-AC03 — NaN/Inf rejection at construction.
    // Evidence: WO-002-AC04 — distance symmetry (in lib property tests).
    use super::*;
    use crate::ops::{Bounded2, Contains2, Project2, Transformable2, Validate};
    use crate::testutil::roundtrip;
    use crate::transform::Transform2D;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn new_rejects_non_finite() {
        assert!(Point2::new(f64::NAN, 0.0).is_err());
        assert!(Point2::new(0.0, f64::INFINITY).is_err());
        assert!(Point2::new(1.0, 2.0).is_ok());
    }

    #[test]
    fn validate_rejects_serde_deserialized_nan() {
        // Deserialized points MUST be validate()d at canonical boundaries.
        let p = Point2 {
            x: f64::NAN,
            y: 1.0,
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn distance_and_squared_distance_match() {
        let a = Point2::new(0.0, 0.0).unwrap();
        let b = Point2::new(3.0, 4.0).unwrap();
        assert!(approx(a.distance_to(b), 5.0));
        assert!(approx(a.distance_squared_to(b), 25.0));
    }

    #[test]
    fn bounding_box_is_degenerate_at_self() {
        let p = Point2::new(1.5, -2.5).unwrap();
        let bb = p.bounding_box();
        assert_eq!(bb.min, p);
        assert_eq!(bb.max, p);
    }

    #[test]
    fn transform_identity_leaves_point_unchanged() {
        // Evidence: WO-002-AC04 — transform identity invariance.
        let p = Point2::new(1.5, -2.5).unwrap();
        let id = Transform2D::identity();
        let q = p.transform(&id, Tolerance::DEFAULT).unwrap();
        assert!(approx(q.x, p.x));
        assert!(approx(q.y, p.y));
    }

    #[test]
    fn project_onto_self_returns_self() {
        let p = Point2::new(1.0, 1.0).unwrap();
        let q = Point2::new(2.0, 5.0).unwrap();
        assert_eq!(p.project_point(&q, Tolerance::DEFAULT), p);
    }

    #[test]
    fn contains_coincident_point() {
        let p = Point2::new(1.0, 2.0).unwrap();
        let q = Point2::new(1.0 + 1e-12, 2.0 - 1e-12).unwrap();
        assert!(p.contains(&q, Tolerance::DEFAULT));
        let r = Point2::new(1.0 + 1e-3, 2.0).unwrap();
        assert!(!p.contains(&r, Tolerance::DEFAULT));
    }

    #[test]
    fn lerp_endpoints() {
        let a = Point2::new(0.0, 0.0).unwrap();
        let b = Point2::new(10.0, 20.0).unwrap();
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        let m = a.lerp(b, 0.5);
        assert!(approx(m.x, 5.0));
        assert!(approx(m.y, 10.0));
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — Point2 round-trip serialization.
        let p = Point2::new(-3.0, 7.5).unwrap();
        let d = roundtrip(&p).unwrap();
        assert_eq!(p, d);
    }
}
