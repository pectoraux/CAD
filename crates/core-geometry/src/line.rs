//! Infinite lines and finite line segments.

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::ops::{
    Bounded2, Contains2, DistanceTo2, Intersect2, Intersection2, Project2, Transformable2, Validate,
};
use crate::point::Point2;
use crate::tolerance::Tolerance;
use crate::transform::Transform2D;
use crate::vector::Vector2;
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
};

// ---------------------------------------------------------------------------
// Infinite line
// ---------------------------------------------------------------------------

/// An infinite 2D line defined by a `point` on the line and a unit
/// `direction`. `direction` is normalized at construction.
///
/// `Line2` is intentionally NOT [`Bounded2`] — an infinite primitive has no
/// finite axis-aligned bounding box. If a conservative bound is needed, take
/// the bounding box of an [`LineSegment2`] clipped to the region of interest.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Line2 {
    /// A point on the line.
    pub point: Point2,
    /// Unit direction along the line.
    pub direction: Vector2,
}

/// Private shadow struct used as the serde wire shape for [`Line2`].
/// Carries the raw (unvalidated) field values; the `TryFrom` impl on
/// `Line2` enforces the finiteness + non-zero direction invariant at the
/// deserialization canonical-model boundary.
#[derive(Deserialize)]
struct RawLine2 {
    point: Point2,
    direction: Vector2,
}

impl TryFrom<RawLine2> for Line2 {
    type Error = GeometryError;

    fn try_from(r: RawLine2) -> Result<Self, Self::Error> {
        // Re-normalize the direction at the canonical boundary so a wire
        // value with a non-unit direction is canonicalized (and a zero
        // direction is rejected as degenerate).
        Self::new(r.point, r.direction)
    }
}

impl<'de> Deserialize<'de> for Line2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawLine2::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl Line2 {
    /// Construct a line from a point and a non-zero direction. The direction
    /// is normalized; if it is a zero vector (exact-zero check — no implicit
    /// tolerance per the frozen v1.1 contract),
    /// [`GeometryError::Degenerate`] is returned.
    #[must_use]
    pub fn new(point: Point2, direction: Vector2) -> Result<Self, GeometryError> {
        point.validate()?;
        direction.validate()?;
        if direction.length_squared() == 0.0 {
            return Err(GeometryError::Degenerate("line direction is zero"));
        }
        let len = direction.length();
        let dir = Vector2::new_unchecked(direction.x / len, direction.y / len);
        Ok(Self {
            point,
            direction: dir,
        })
    }

    /// Construct a line through two distinct points. The direction is
    /// `(p2 - p1)` normalized. Returns `Degenerate` if `p1 == p2` (exact
    /// comparison — no implicit tolerance).
    #[must_use]
    pub fn from_two_points(p1: Point2, p2: Point2) -> Result<Self, GeometryError> {
        p1.validate()?;
        p2.validate()?;
        let d = p1.vector_to(p2);
        if d.length_squared() == 0.0 {
            return Err(GeometryError::Degenerate(
                "line from_two_points: coincident points",
            ));
        }
        Self::new(p1, d)
    }

    /// Closest point on the line to `p`. Orthogonal projection.
    #[must_use]
    pub fn project_point(&self, p: &Point2) -> Point2 {
        let v = self.point.vector_to(*p);
        let t = v.dot(self.direction);
        Point2::new_unchecked(
            self.point.x + self.direction.x * t,
            self.point.y + self.direction.y * t,
        )
    }

    /// Distance from `p` to the line (perpendicular distance, unsigned).
    #[must_use]
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        let v = self.point.vector_to(*p);
        let cross = self.direction.cross(v).abs();
        // direction is unit, so |cross| is the perpendicular distance.
        cross
    }

    /// Returns `true` if `p` lies on the line within the canonical
    /// tolerance policy
    /// ([`Tolerance::CANONICAL`](crate::tolerance::Tolerance::CANONICAL)).
    #[must_use]
    pub fn contains_point(&self, p: &Point2) -> bool {
        self.distance_to_point(p) <= Tolerance::CANONICAL.absolute()
    }

    /// Parameter `t` such that `point + direction * t == p_projected`.
    #[must_use]
    pub fn parameter_of(&self, p: &Point2) -> f64 {
        self.point.vector_to(*p).dot(self.direction)
    }
}

impl Transformable2 for Line2 {
    fn transform(&self, transform: &Transform2D) -> Result<Self, GeometryError> {
        let p = transform.apply_point(&self.point);
        let d = transform.apply_vector(&self.direction);
        // The transformed direction may be zero if the transform is singular
        // (collapses the line). Reject via exact-zero check (no implicit
        // tolerance per the frozen v1.1 contract).
        if d.length_squared() == 0.0 {
            return Err(GeometryError::Degenerate(
                "transform collapses line direction to zero",
            ));
        }
        let len = d.length();
        let dn = Vector2::new_unchecked(d.x / len, d.y / len);
        Ok(Self {
            point: p,
            direction: dn,
        })
    }
}

impl DistanceTo2<Point2> for Line2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl Project2 for Line2 {
    fn project_point(&self, point: &Point2) -> Point2 {
        self.project_point(point)
    }
}

impl Contains2<Point2> for Line2 {
    fn contains(&self, rhs: &Point2) -> bool {
        self.contains_point(rhs)
    }
}

impl Intersect2<Line2> for Line2 {
    fn intersect(&self, rhs: &Line2) -> Intersection2 {
        let tol = Tolerance::CANONICAL;
        let det = self.direction.cross(rhs.direction);
        if det.abs() <= tol.absolute() {
            // Parallel. Coincident if rhs.point is on self.
            if self.contains_point(&rhs.point) {
                return Intersection2::Coincident;
            }
            return Intersection2::Empty;
        }
        // Solve: self.point + t * self.direction == rhs.point + s * rhs.direction
        // (self.point - rhs.point) cross rhs.direction = -t * (self.dir cross rhs.dir)
        let w = rhs.point.vector_to(self.point);
        let t = w.cross(rhs.direction) / det;
        let p = Point2::new_unchecked(
            self.point.x + self.direction.x * t,
            self.point.y + self.direction.y * t,
        );
        Intersection2::Point(p)
    }
}

impl Intersect2<LineSegment2> for Line2 {
    fn intersect(&self, rhs: &LineSegment2) -> Intersection2 {
        // Treat the segment as a finite parametric range. First line-line
        // intersection, then clip t to [0, 1].
        let tol = Tolerance::CANONICAL;
        let seg_dir = rhs.start.vector_to(rhs.end);
        let det = self.direction.cross(seg_dir);
        if det.abs() <= tol.absolute() {
            // Parallel. If the segment's start is on the line, the whole
            // segment lies on it (Coincident); otherwise Empty.
            if self.contains_point(&rhs.start) {
                return Intersection2::Coincident;
            }
            return Intersection2::Empty;
        }
        let w = self.point.vector_to(rhs.start);
        let t = w.cross(self.direction) / det; // segment parameter in [0,1]
        if t < -tol.absolute() || t > 1.0 + tol.absolute() {
            return Intersection2::Empty;
        }
        let tc = t.clamp(0.0, 1.0);
        Intersection2::Point(Point2::new_unchecked(
            rhs.start.x + seg_dir.x * tc,
            rhs.start.y + seg_dir.y * tc,
        ))
    }
}

// ---------------------------------------------------------------------------
// Line segment
// ---------------------------------------------------------------------------

/// A finite line segment between two distinct points. Zero-length segments
/// (where `start == end` exactly) are rejected at construction and at the
/// deserialization canonical-model boundary as degenerate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LineSegment2 {
    /// Start endpoint.
    pub start: Point2,
    /// End endpoint.
    pub end: Point2,
}

/// Private shadow struct used as the serde wire shape for [`LineSegment2`].
#[derive(Deserialize)]
struct RawLineSegment2 {
    start: Point2,
    end: Point2,
}

impl TryFrom<RawLineSegment2> for LineSegment2 {
    type Error = GeometryError;

    fn try_from(r: RawLineSegment2) -> Result<Self, Self::Error> {
        Self::new(r.start, r.end)
    }
}

impl<'de> Deserialize<'de> for LineSegment2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawLineSegment2::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl LineSegment2 {
    /// Construct a segment with two distinct endpoints. Rejects NaN/Inf and
    /// zero-length segments (where `start == end` exactly — no implicit
    /// tolerance is applied at construction; per the frozen contract
    /// "tolerance is never implicit or caller-chosen on a per-operation
    /// basis").
    #[must_use]
    pub fn new(start: Point2, end: Point2) -> Result<Self, GeometryError> {
        start.validate()?;
        end.validate()?;
        if start.x == end.x && start.y == end.y {
            return Err(GeometryError::Degenerate("zero-length segment"));
        }
        Ok(Self { start, end })
    }

    /// Euclidean length of the segment.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.start.distance_to(self.end)
    }

    /// Squared length.
    #[must_use]
    pub const fn length_squared(&self) -> f64 {
        self.start.distance_squared_to(self.end)
    }

    /// Unit direction `end - start` normalized. Since `new()` rejected
    /// zero-length segments, the length is always strictly positive;
    /// normalization is exact (no implicit tolerance per the frozen v1.1
    /// contract).
    #[must_use]
    pub fn direction(&self) -> Vector2 {
        let d = self.start.vector_to(self.end);
        let len = d.length();
        Vector2::new_unchecked(d.x / len, d.y / len)
    }

    /// Midpoint of the segment.
    #[must_use]
    pub fn midpoint(&self) -> Point2 {
        Point2::new_unchecked(
            self.start.x.midpoint(self.end.x),
            self.start.y.midpoint(self.end.y),
        )
    }

    /// Parameter `t in [0, 1]` of the orthogonal projection of `point` onto
    /// the segment's infinite line (NOT clamped — see [`Self::project_point`]
    /// for the clamped version).
    #[must_use]
    pub fn parameter_of(&self, point: &Point2) -> f64 {
        let d = self.start.vector_to(self.end);
        let len_sq = d.length_squared();
        if len_sq == 0.0 {
            return 0.0;
        }
        self.start.vector_to(*point).dot(d) / len_sq
    }

    /// Closest point on the segment to `point`, clamped to `[start, end]`.
    #[must_use]
    pub fn project_point(&self, point: &Point2) -> Point2 {
        let t = self.parameter_of(point).clamp(0.0, 1.0);
        Point2::new_unchecked(
            self.start.x + (self.end.x - self.start.x) * t,
            self.start.y + (self.end.y - self.start.y) * t,
        )
    }

    /// Distance from `point` to the segment (clamped projection).
    #[must_use]
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        self.project_point(point).distance_to(*point)
    }

    /// Returns `true` if `point` lies on the segment within the canonical
    /// tolerance policy
    /// ([`Tolerance::CANONICAL`](crate::tolerance::Tolerance::CANONICAL)).
    #[must_use]
    pub fn contains_point(&self, point: &Point2) -> bool {
        self.distance_to_point(point) <= Tolerance::CANONICAL.absolute()
    }
}

impl Validate for LineSegment2 {
    fn validate(&self) -> Result<(), GeometryError> {
        self.start.validate()?;
        self.end.validate()?;
        if self.start.x == self.end.x && self.start.y == self.end.y {
            return Err(GeometryError::Degenerate("zero-length segment"));
        }
        Ok(())
    }
}

impl Bounded2 for LineSegment2 {
    fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new_unchecked(
            Point2::new_unchecked(self.start.x.min(self.end.x), self.start.y.min(self.end.y)),
            Point2::new_unchecked(self.start.x.max(self.end.x), self.start.y.max(self.end.y)),
        )
    }
}

impl Transformable2 for LineSegment2 {
    fn transform(&self, transform: &Transform2D) -> Result<Self, GeometryError> {
        let new_start = transform.apply_point(&self.start);
        let new_end = transform.apply_point(&self.end);
        // Route through `new` so a singular transform that collapses the
        // segment to zero length is rejected (preserves the LineSegment2
        // type invariant; never bypasses `new`).
        LineSegment2::new(new_start, new_end)
    }
}

impl DistanceTo2<Point2> for LineSegment2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl DistanceTo2<LineSegment2> for LineSegment2 {
    fn distance_to(&self, rhs: &LineSegment2) -> f64 {
        // Segment-segment distance via exact-arithmetic strict-crossing test
        // (no implicit tolerance per the frozen v1.1 contract). If the
        // segments strictly cross, distance is 0; otherwise the minimum of
        // the four point-to-segment distances (which also catches the
        // collinear-overlap case because one endpoint projects inside the
        // other segment with distance 0).
        let d1 = self.start.vector_to(self.end);
        let d2 = rhs.start.vector_to(rhs.end);
        let r = self.start.vector_to(rhs.start);
        let denom = d1.cross(d2);
        if denom != 0.0 {
            let t1 = r.cross(d2) / denom;
            let t2 = r.cross(d1) / denom;
            if (0.0..=1.0).contains(&t1) && (0.0..=1.0).contains(&t2) {
                return 0.0;
            }
        }
        // Otherwise the minimum of the four point-to-segment distances.
        let candidates = [
            self.distance_to_point(&rhs.start),
            self.distance_to_point(&rhs.end),
            rhs.distance_to_point(&self.start),
            rhs.distance_to_point(&self.end),
        ];
        candidates.iter().copied().fold(f64::INFINITY, f64::min)
    }
}

impl Project2 for LineSegment2 {
    fn project_point(&self, point: &Point2) -> Point2 {
        self.project_point(point)
    }
}

impl Contains2<Point2> for LineSegment2 {
    fn contains(&self, rhs: &Point2) -> bool {
        self.contains_point(rhs)
    }
}

impl Intersect2<Line2> for LineSegment2 {
    fn intersect(&self, rhs: &Line2) -> Intersection2 {
        // Delegate: line.intersect(segment) is symmetric.
        rhs.intersect(self)
    }
}

impl Intersect2<LineSegment2> for LineSegment2 {
    fn intersect(&self, rhs: &LineSegment2) -> Intersection2 {
        let tol = Tolerance::CANONICAL;
        let d1 = self.start.vector_to(self.end);
        let d2 = rhs.start.vector_to(rhs.end);
        let denom = d1.cross(d2);
        let r = self.start.vector_to(rhs.start);

        if denom.abs() > tol.absolute() {
            // Not parallel. Solve for parameters.
            let t1 = r.cross(d2) / denom;
            let t2 = r.cross(d1) / denom;
            let lo = -tol.absolute();
            let hi = 1.0 + tol.absolute();
            if t1 >= lo && t1 <= hi && t2 >= lo && t2 <= hi {
                let tc = t1.clamp(0.0, 1.0);
                return Intersection2::Point(Point2::new_unchecked(
                    self.start.x + d1.x * tc,
                    self.start.y + d1.y * tc,
                ));
            }
            return Intersection2::Empty;
        }

        // Parallel or anti-parallel.
        if r.length_squared() > tol.absolute() * tol.absolute()
            && r.cross(d1).abs() > tol.absolute()
        {
            // Parallel but NOT collinear: no intersection.
            return Intersection2::Empty;
        }

        // Collinear: project both segments onto d1's parameter axis and test
        // overlap. The parameter of `p` along self is `t = (p - self.start) · d1 / |d1|^2`.
        let len_sq = d1.length_squared();
        if len_sq == 0.0 {
            // self is a degenerate point — should not happen because new()
            // rejects zero-length segments, but be defensive.
            return if rhs.contains_point(&self.start) {
                Intersection2::Point(self.start)
            } else {
                Intersection2::Empty
            };
        }
        let t_start_rhs = self.start.vector_to(rhs.start).dot(d1) / len_sq;
        let t_end_rhs = self.start.vector_to(rhs.end).dot(d1) / len_sq;
        let (lo_r, hi_r) = if t_start_rhs <= t_end_rhs {
            (t_start_rhs, t_end_rhs)
        } else {
            (t_end_rhs, t_start_rhs)
        };
        // Self spans [0, 1].
        let lo_s = 0.0_f64.max(lo_r);
        let hi_s = 1.0_f64.min(hi_r);
        if lo_s > hi_s + tol.absolute() {
            return Intersection2::Empty;
        }
        // If overlap is a single point, treat as Point (degenerate segment).
        if (hi_s - lo_s).abs() <= tol.absolute() {
            let t = lo_s.clamp(0.0, 1.0);
            return Intersection2::Point(Point2::new_unchecked(
                self.start.x + d1.x * t,
                self.start.y + d1.y * t,
            ));
        }
        // Full overlap of one covering the other → Coincident.
        if lo_s <= tol.absolute() && hi_s >= 1.0 - tol.absolute() {
            return Intersection2::Coincident;
        }
        // Partial overlap → Segment. Route through `new` to preserve the
        // LineSegment2 type invariant (never bypass `new`).
        let p_lo = Point2::new_unchecked(self.start.x + d1.x * lo_s, self.start.y + d1.y * lo_s);
        let p_hi = Point2::new_unchecked(self.start.x + d1.x * hi_s, self.start.y + d1.y * hi_s);
        match LineSegment2::new(p_lo, p_hi) {
            Ok(seg) => Intersection2::Segment(seg),
            // The overlap degenerated to a point despite the length check
            // above (numerical edge case); report as Point.
            Err(_) => Intersection2::Point(p_lo),
        }
    }
}

impl Intersect2<crate::circle::Circle2> for LineSegment2 {
    fn intersect(&self, rhs: &crate::circle::Circle2) -> Intersection2 {
        // Delegate to Circle2.intersect(segment) for symmetric semantics.
        rhs.intersect(self)
    }
}

impl From<Line2> for (Point2, Vector2) {
    fn from(line: Line2) -> Self {
        (line.point, line.direction)
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Line2 / LineSegment2 serde round-trip.
    // Evidence: WO-002-AC02 — line-line / segment-segment intersection determinism.
    // Evidence: WO-002-AC03 — zero-length segment rejected; coincident points
    // for line; collinear segments handled; NaN/Inf rejected at the
    // deserialization canonical-model boundary.
    // Evidence: WO-002-AC04 — projection lies on primitive.
    use super::*;
    use crate::circle::Circle2;
    use crate::ops::{
        Bounded2, Contains2, DistanceTo2, Intersect2, Intersection2, Transformable2, Validate,
    };
    use crate::testutil::{Prng, roundtrip};

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_rejects_zero_length_segment() {
        // Evidence: WO-002-AC03 — degenerate zero-length segment.
        let p = Point2::new(1.0, 2.0).unwrap();
        assert!(LineSegment2::new(p, p).is_err());
    }

    #[test]
    fn new_rejects_non_finite_line() {
        assert!(
            Line2::new(
                Point2 {
                    x: f64::NAN,
                    y: 0.0
                },
                Vector2::I
            )
            .is_err()
        );
        assert!(Line2::new(Point2::ORIGIN, Vector2::ZERO).is_err());
    }

    #[test]
    fn from_two_points_rejects_coincident() {
        // Evidence: WO-002-AC03 — coincident points cannot form a line.
        let p = Point2::new(1.0, 1.0).unwrap();
        assert!(Line2::from_two_points(p, p).is_err());
    }

    #[test]
    fn segment_length_direction_midpoint() {
        let s = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(3.0, 4.0).unwrap(),
        )
        .unwrap();
        assert!(approx(s.length(), 5.0));
        let d = s.direction();
        assert!(approx(d.x, 0.6));
        assert!(approx(d.y, 0.8));
        let m = s.midpoint();
        assert!(approx(m.x, 1.5));
        assert!(approx(m.y, 2.0));
    }

    #[test]
    fn project_point_on_segment_clamps() {
        let s = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(10.0, 0.0).unwrap(),
        )
        .unwrap();
        let p = s.project_point(&Point2::new(20.0, 5.0).unwrap());
        assert!(approx(p.x, 10.0));
        assert!(approx(p.y, 0.0));
        let p = s.project_point(&Point2::new(-5.0, 5.0).unwrap());
        assert!(approx(p.x, 0.0));
        let p = s.project_point(&Point2::new(5.0, 5.0).unwrap());
        assert!(approx(p.x, 5.0));
        // Evidence: WO-002-AC04 — projection lies on primitive.
        assert!(s.contains(&p));
    }

    #[test]
    fn line_line_intersection_single_point() {
        // Evidence: WO-002-AC02 — deterministic line-line intersection.
        let l1 = Line2::from_two_points(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        let l2 = Line2::from_two_points(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(0.0, 1.0).unwrap(),
        )
        .unwrap();
        match l1.intersect(&l2) {
            Intersection2::Point(p) => {
                assert!(approx(p.x, 0.0));
                assert!(approx(p.y, 0.0));
            }
            other => panic!("expected Point, got {other:?}"),
        }
    }

    #[test]
    fn parallel_lines_are_empty() {
        let l1 = Line2::from_two_points(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        let l2 = Line2::from_two_points(
            Point2::new(0.0, 1.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
        )
        .unwrap();
        assert_eq!(l1.intersect(&l2), Intersection2::Empty);
    }

    #[test]
    fn coincident_lines_are_coincident() {
        let l1 = Line2::from_two_points(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        let l2 = Line2::from_two_points(
            Point2::new(2.0, 0.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        )
        .unwrap();
        assert_eq!(l1.intersect(&l2), Intersection2::Coincident);
    }

    #[test]
    fn segment_segment_cross_at_point() {
        // Evidence: WO-002-AC02 — segment-segment intersection determinism.
        let s1 = LineSegment2::new(
            Point2::new(-1.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        let s2 = LineSegment2::new(
            Point2::new(0.0, -1.0).unwrap(),
            Point2::new(0.0, 1.0).unwrap(),
        )
        .unwrap();
        match s1.intersect(&s2) {
            Intersection2::Point(p) => {
                assert!(approx(p.x, 0.0));
                assert!(approx(p.y, 0.0));
            }
            other => panic!("expected Point, got {other:?}"),
        }
    }

    #[test]
    fn segment_segment_collinear_overlap_is_segment() {
        // Evidence: WO-002-AC03 — collinear overlap → Segment.
        let s1 = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(4.0, 0.0).unwrap(),
        )
        .unwrap();
        let s2 = LineSegment2::new(
            Point2::new(2.0, 0.0).unwrap(),
            Point2::new(6.0, 0.0).unwrap(),
        )
        .unwrap();
        match s1.intersect(&s2) {
            Intersection2::Segment(seg) => {
                assert!(approx(seg.start.x, 2.0));
                assert!(approx(seg.end.x, 4.0));
            }
            other => panic!("expected Segment, got {other:?}"),
        }
    }

    #[test]
    fn segment_segment_disjoint_is_empty() {
        let s1 = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        let s2 = LineSegment2::new(
            Point2::new(5.0, 5.0).unwrap(),
            Point2::new(6.0, 5.0).unwrap(),
        )
        .unwrap();
        assert_eq!(s1.intersect(&s2), Intersection2::Empty);
        // The minimum distance between two non-overlapping, parallel
        // horizontal segments at y=0 ([0,1]) and y=5 ([5,6]) is the
        // diagonal of a 4-by-5 right triangle (4 = 5-1 horizontal gap,
        // 5 = vertical gap).
        let d = s1.distance_to(&s2);
        assert!((d - (16.0_f64 + 25.0_f64).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn segment_segment_coincident_full_overlap() {
        let s1 = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
        )
        .unwrap();
        let s2 = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
        )
        .unwrap();
        assert_eq!(s1.intersect(&s2), Intersection2::Coincident);
    }

    #[test]
    fn line_segment_intersection() {
        let line = Line2::from_two_points(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
        )
        .unwrap();
        let seg = LineSegment2::new(
            Point2::new(-1.0, -1.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
        )
        .unwrap();
        assert_eq!(line.intersect(&seg), Intersection2::Coincident);

        let seg2 = LineSegment2::new(
            Point2::new(-1.0, 3.0).unwrap(),
            Point2::new(3.0, -1.0).unwrap(),
        )
        .unwrap();
        match line.intersect(&seg2) {
            Intersection2::Point(p) => {
                assert!(approx(p.x, 1.0));
                assert!(approx(p.y, 1.0));
            }
            other => panic!("expected Point, got {other:?}"),
        }
    }

    #[test]
    fn segment_circle_intersection_two_points() {
        // Circle of radius 1 at origin; horizontal segment through (0,0) from -2 to 2.
        let c = Circle2::new(Point2::ORIGIN, 1.0).unwrap();
        let s = LineSegment2::new(
            Point2::new(-2.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
        )
        .unwrap();
        match s.intersect(&c) {
            Intersection2::Points(pts) => {
                assert_eq!(pts.len(), 2);
                let xs: Vec<f64> = pts.iter().map(|p| p.x).collect();
                assert!(xs.iter().any(|&x| approx(x, -1.0)));
                assert!(xs.iter().any(|&x| approx(x, 1.0)));
            }
            other => panic!("expected Points(2), got {other:?}"),
        }
    }

    #[test]
    fn transform_identity_preserves_segment() {
        // Evidence: WO-002-AC04 — transform identity invariance.
        let s = LineSegment2::new(
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(3.0, -1.0).unwrap(),
        )
        .unwrap();
        let id = Transform2D::identity();
        let t = s.transform(&id).unwrap();
        assert!(approx(t.start.x, s.start.x));
        assert!(approx(t.start.y, s.start.y));
        assert!(approx(t.end.x, s.end.x));
        assert!(approx(t.end.y, s.end.y));
    }

    #[test]
    fn segment_distance_to_point_property() {
        let mut p = Prng::new();
        for _ in 0..64 {
            let a = Point2::new(p.signed_f64(10.0), p.signed_f64(10.0)).unwrap();
            let b = Point2::new(p.signed_f64(10.0), p.signed_f64(10.0)).unwrap();
            if a == b {
                continue;
            }
            let s = LineSegment2::new(a, b).unwrap();
            // Sample a point ON the segment by interpolating — distance must be ~0.
            let on_seg = a.lerp(b, p.range_f64(0.0, 1.0));
            assert!(s.distance_to(&on_seg) < 1e-9);
        }
    }

    #[test]
    fn serde_roundtrip_line() {
        // Evidence: WO-002-AC01 — Line2 round-trip serialization.
        let l = Line2::from_two_points(
            Point2::new(1.0, 1.0).unwrap(),
            Point2::new(4.0, 5.0).unwrap(),
        )
        .unwrap();
        let d = roundtrip(&l).unwrap();
        assert_eq!(l, d);
    }

    #[test]
    fn serde_roundtrip_segment() {
        // Evidence: WO-002-AC01 — LineSegment2 round-trip serialization.
        let s = LineSegment2::new(
            Point2::new(-1.5, 2.0).unwrap(),
            Point2::new(3.0, -4.5).unwrap(),
        )
        .unwrap();
        let d = roundtrip(&s).unwrap();
        assert_eq!(s, d);
    }

    #[test]
    fn validate_rejects_deserialized_zero_length() {
        // After direct struct-literal construction (test-only path; not a
        // canonical-model boundary), `validate()` must catch a zero-length
        // segment that snuck in.
        let bad = LineSegment2 {
            start: Point2::ORIGIN,
            end: Point2::ORIGIN,
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn bounding_box_covers_segment() {
        let s = LineSegment2::new(
            Point2::new(-3.0, 7.0).unwrap(),
            Point2::new(5.0, 1.0).unwrap(),
        )
        .unwrap();
        let b = s.bounding_box();
        assert!(approx(b.min.x, -3.0));
        assert!(approx(b.min.y, 1.0));
        assert!(approx(b.max.x, 5.0));
        assert!(approx(b.max.y, 7.0));
    }
}
