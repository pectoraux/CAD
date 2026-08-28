//! 2D axis-aligned bounding boxes (conservative extents).

use crate::error::GeometryError;
use crate::ops::{Bounded2, Contains2, Transformable2, Validate};
use crate::point::Point2;
use crate::tolerance::Tolerance;
use crate::transform::Transform2D;
use serde::{Deserialize, Serialize};

/// An axis-aligned bounding box with `min` and `max` corners. Invariants:
/// `min.x <= max.x` and `min.y <= max.y`. A zero-area box (a single point)
/// is permitted and considered degenerate (see [`Self::is_degenerate`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox2 {
    /// The lower corner (smallest x and y).
    pub min: Point2,
    /// The upper corner (largest x and y).
    pub max: Point2,
}

impl BoundingBox2 {
    /// Construct a bounding box from explicit `min` and `max`. Requires
    /// `min.x <= max.x` and `min.y <= max.y` and all coordinates finite.
    #[must_use]
    pub fn new(min: Point2, max: Point2) -> Result<Self, GeometryError> {
        min.validate()?;
        max.validate()?;
        if min.x > max.x || min.y > max.y {
            return Err(GeometryError::Degenerate(
                "bounding box min greater than max",
            ));
        }
        Ok(Self { min, max })
    }

    /// Construct a bounding box WITHOUT validation. Crate-internal helper for
    /// callers that have already verified invariants (e.g. bounding box of a
    /// single point: `min == max`).
    #[must_use]
    pub(crate) const fn new_unchecked(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    /// Construct a bounding box that covers the given points. An empty slice
    /// is rejected (degenerate — no points to bound).
    #[must_use]
    pub fn from_points(points: &[Point2]) -> Result<Self, GeometryError> {
        if points.is_empty() {
            return Err(GeometryError::Degenerate("empty point list"));
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for p in points {
            p.validate()?;
            if p.x < min_x {
                min_x = p.x;
            }
            if p.y < min_y {
                min_y = p.y;
            }
            if p.x > max_x {
                max_x = p.x;
            }
            if p.y > max_y {
                max_y = p.y;
            }
        }
        let min = Point2::new_unchecked(min_x, min_y);
        let max = Point2::new_unchecked(max_x, max_y);
        Ok(Self { min, max })
    }

    /// Width in X (`max.x - min.x`, `>= 0`).
    #[must_use]
    pub const fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    /// Height in Y (`max.y - min.y`, `>= 0`).
    #[must_use]
    pub const fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    /// Center of the box.
    #[must_use]
    pub fn center(&self) -> Point2 {
        Point2::new_unchecked(
            self.min.x.midpoint(self.max.x),
            self.min.y.midpoint(self.max.y),
        )
    }

    /// Returns `true` if `point` lies inside or on the boundary of `self`.
    #[must_use]
    pub fn contains(&self, point: &Point2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Returns `true` if `other` is entirely contained within `self`.
    #[must_use]
    pub fn contains_box(&self, other: &Self) -> bool {
        self.min.x <= other.min.x
            && self.min.y <= other.min.y
            && self.max.x >= other.max.x
            && self.max.y >= other.max.y
    }

    /// Returns `true` if `self` and `other` overlap (touching counts).
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Returns the union of `self` and `other` (the smallest box containing
    /// both).
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: Point2::new_unchecked(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Point2::new_unchecked(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    /// Returns a box expanded to include `point`.
    #[must_use]
    pub fn expand(&self, point: &Point2) -> Self {
        Self {
            min: Point2::new_unchecked(self.min.x.min(point.x), self.min.y.min(point.y)),
            max: Point2::new_unchecked(self.max.x.max(point.x), self.max.y.max(point.y)),
        }
    }

    /// Returns `true` if the box has zero area (a single point or a line)
    /// within `tolerance`.
    #[must_use]
    pub fn is_degenerate(&self, tolerance: Tolerance) -> bool {
        self.width() <= tolerance.absolute && self.height() <= tolerance.absolute
    }

    /// Returns the four corner points of the box, CCW starting from `min`.
    #[must_use]
    pub const fn corners(&self) -> [Point2; 4] {
        [
            self.min,
            Point2::new_unchecked(self.max.x, self.min.y),
            self.max,
            Point2::new_unchecked(self.min.x, self.max.y),
        ]
    }
}

impl Validate for BoundingBox2 {
    fn validate(&self) -> Result<(), GeometryError> {
        self.min.validate()?;
        self.max.validate()?;
        if self.min.x > self.max.x || self.min.y > self.max.y {
            return Err(GeometryError::Degenerate(
                "bounding box min greater than max",
            ));
        }
        Ok(())
    }
}

impl Bounded2 for BoundingBox2 {
    fn bounding_box(&self) -> BoundingBox2 {
        *self
    }
}

impl Transformable2 for BoundingBox2 {
    /// Transform a bounding box: rotate/scale/translate all four corners and
    /// then re-fit an axis-aligned box. The resulting AABB is conservative
    /// (it may be larger than the rotated box; never smaller). Always
    /// representable — returns `Ok`.
    fn transform(&self, transform: &Transform2D, _tol: Tolerance) -> Result<Self, GeometryError> {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for &c in &self.corners() {
            let p = transform.apply_point(&c);
            if p.x < min_x {
                min_x = p.x;
            }
            if p.y < min_y {
                min_y = p.y;
            }
            if p.x > max_x {
                max_x = p.x;
            }
            if p.y > max_y {
                max_y = p.y;
            }
        }
        Ok(Self {
            min: Point2::new_unchecked(min_x, min_y),
            max: Point2::new_unchecked(max_x, max_y),
        })
    }
}

impl Contains2<Point2> for BoundingBox2 {
    /// Containment is exact (no tolerance needed for an AABB point-in-box
    /// test); the `tol` parameter is accepted for trait-signature
    /// consistency and ignored.
    fn contains(&self, rhs: &Point2, _tol: Tolerance) -> bool {
        BoundingBox2::contains(self, rhs)
    }
}

impl Contains2<BoundingBox2> for BoundingBox2 {
    fn contains(&self, rhs: &BoundingBox2, _tol: Tolerance) -> bool {
        BoundingBox2::contains_box(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — BoundingBox2 serde round-trip.
    // Evidence: WO-002-AC03 — empty point list, min > max rejected.
    // Evidence: WO-002-AC04 — bbox contains its points; transform-then-bbox
    // consistency for translation.
    use super::*;
    use crate::ops::{Bounded2, Transformable2, Validate};
    use crate::testutil::{Prng, roundtrip};
    use crate::tolerance::Tolerance;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_rejects_inverted_box() {
        // Evidence: WO-002-AC03 — min > max is a degenerate/invalid box.
        let p1 = Point2::new(1.0, 1.0).unwrap();
        let p2 = Point2::new(0.0, 0.0).unwrap();
        assert!(BoundingBox2::new(p2, p1).is_ok()); // min < max is OK
        assert!(BoundingBox2::new(p1, p2).is_err()); // min > max rejected
    }

    #[test]
    fn from_points_rejects_empty() {
        // Evidence: WO-002-AC03 — empty point list is degenerate.
        let pts: &[Point2] = &[];
        assert!(BoundingBox2::from_points(pts).is_err());
    }

    #[test]
    fn from_points_canonical() {
        let pts = [
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 5.0).unwrap(),
            Point2::new(-3.0, 2.0).unwrap(),
        ];
        let b = BoundingBox2::from_points(&pts).unwrap();
        assert!(approx(b.min.x, -3.0));
        assert!(approx(b.min.y, 0.0));
        assert!(approx(b.max.x, 1.0));
        assert!(approx(b.max.y, 5.0));
    }

    #[test]
    fn contains_box_and_intersect() {
        let a = BoundingBox2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(4.0, 4.0).unwrap(),
        )
        .unwrap();
        let b = BoundingBox2::new(
            Point2::new(1.0, 1.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
        )
        .unwrap();
        assert!(a.contains_box(&b));
        assert!(!b.contains_box(&a));
        assert!(a.intersects(&b));
        let c = BoundingBox2::new(
            Point2::new(5.0, 5.0).unwrap(),
            Point2::new(6.0, 6.0).unwrap(),
        )
        .unwrap();
        assert!(!a.intersects(&c));
    }

    #[test]
    fn union_and_expand() {
        let a = BoundingBox2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
        )
        .unwrap();
        let b = BoundingBox2::new(
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, 3.0).unwrap(),
        )
        .unwrap();
        let u = a.union(&b);
        assert!(approx(u.min.x, 0.0));
        assert!(approx(u.max.x, 3.0));
        let e = a.expand(&Point2::new(-1.0, 5.0).unwrap());
        assert!(approx(e.min.x, -1.0));
        assert!(approx(e.max.y, 5.0));
    }

    #[test]
    fn is_degenerate_for_point() {
        let p = Point2::new(1.0, 2.0).unwrap();
        let b = p.bounding_box();
        assert!(b.is_degenerate(Tolerance::DEFAULT));
        let real = BoundingBox2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
        )
        .unwrap();
        assert!(!real.is_degenerate(Tolerance::DEFAULT));
    }

    #[test]
    fn transform_by_translation_then_bbox_is_bbox_then_transformed() {
        // Evidence: WO-002-AC04 — transform-then-bbox == bbox-then-transform
        // consistency for pure translation.
        let pts = [
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 5.0).unwrap(),
            Point2::new(-3.0, 2.0).unwrap(),
        ];
        let b1 = BoundingBox2::from_points(&pts).unwrap();
        let t = Transform2D::translation(7.0, -4.0);
        let transformed_pts = pts.map(|p| p.transform(&t, Tolerance::DEFAULT).unwrap());
        let b2 = BoundingBox2::from_points(&transformed_pts).unwrap();
        let b3 = b1.transform(&t, Tolerance::DEFAULT).unwrap();
        assert!(approx(b2.min.x, b3.min.x));
        assert!(approx(b2.min.y, b3.min.y));
        assert!(approx(b2.max.x, b3.max.x));
        assert!(approx(b2.max.y, b3.max.y));
    }

    #[test]
    fn bbox_contains_all_source_points() {
        // Evidence: WO-002-AC04 — bounding box contains its points.
        let mut p = Prng::new();
        for _ in 0..64 {
            let mut pts = Vec::with_capacity(16);
            for _ in 0..16 {
                pts.push(Point2::new(p.signed_f64(1000.0), p.signed_f64(1000.0)).unwrap());
            }
            let b = BoundingBox2::from_points(&pts).unwrap();
            for q in &pts {
                assert!(b.contains(q), "bbox should contain its source point");
            }
        }
    }

    #[test]
    fn validate_rejects_nan_corner() {
        let b = BoundingBox2 {
            min: Point2::new_unchecked(f64::NAN, 0.0),
            max: Point2::new_unchecked(1.0, 1.0),
        };
        assert!(b.validate().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — BoundingBox2 round-trip serialization.
        let b = BoundingBox2::new(
            Point2::new(-1.5, 0.25).unwrap(),
            Point2::new(7.0, 9.5).unwrap(),
        )
        .unwrap();
        let d = roundtrip(&b).unwrap();
        assert_eq!(b, d);
    }
}
