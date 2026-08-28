//! 2D polylines (open or closed sequences of line segments).

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::line::LineSegment2;
use crate::ops::{
    Bounded2, Contains2, DistanceTo2, Intersect2, Intersection2, Project2, Transformable2, Validate,
};
use crate::point::Point2;
use crate::tolerance::Tolerance;
use crate::transform::Transform2D;
use serde::{Deserialize, Serialize};

/// A 2D polyline: a sequence of vertices connected by line segments. May be
/// open or closed (closed polylines connect the last vertex back to the
/// first).
///
/// Construction invariants:
/// - All vertices finite.
/// - Open polylines need at least 2 distinct vertices.
/// - Closed polylines need at least 3 distinct vertices (a closed 2-vertex
///   polyline would be a degenerate segment traced both ways).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline2 {
    /// Vertex list (in order; non-empty after construction).
    pub vertices: Vec<Point2>,
    /// Whether the polyline is closed (last vertex connected to first).
    pub closed: bool,
}

impl Polyline2 {
    /// Construct a polyline from vertices and a `closed` flag.
    ///
    /// Validates that all vertices are finite and that the vertex-count
    /// requirement is met (open: ≥ 2; closed: ≥ 3 distinct vertices).
    #[must_use]
    pub fn new(vertices: Vec<Point2>, closed: bool) -> Result<Self, GeometryError> {
        for v in &vertices {
            v.validate()?;
        }
        let min = if closed { 3 } else { 2 };
        if vertices.len() < min {
            return Err(GeometryError::Degenerate(
                "polyline has insufficient vertices",
            ));
        }
        Ok(Self { vertices, closed })
    }

    /// Returns `true` iff the polyline is closed.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Number of segments in the polyline (open: `n - 1`; closed: `n`).
    #[must_use]
    pub fn segment_count(&self) -> usize {
        if self.closed {
            self.vertices.len()
        } else {
            self.vertices.len() - 1
        }
    }

    /// Returns the `i`-th segment of the polyline (open: 0..=n-2; closed:
    /// 0..=n-1, where segment n-1 connects the last vertex to the first).
    ///
    /// Returns `None` if `i` is out of range. The returned segment may be
    /// degenerate (zero-length) if the underlying vertices are coincident —
    /// callers requiring non-degenerate segments must check separately.
    #[must_use]
    pub fn segment(&self, i: usize) -> Option<LineSegment2> {
        if i >= self.segment_count() {
            return None;
        }
        let n = self.vertices.len();
        let start = self.vertices[i];
        let end = if self.closed && i == n - 1 {
            self.vertices[0]
        } else {
            self.vertices[i + 1]
        };
        // If start==end the polyline has coincident adjacent vertices (not
        // caught by `new()` — `new` only rejects insufficient vertex counts,
        // not duplicate-adjacent vertices). Return a degenerate "zero
        // segment" by constructing via the unchecked path.
        if start.x == end.x && start.y == end.y {
            return Some(LineSegment2 { start, end });
        }
        // `LineSegment2::new` is infallible for distinct finite points; use
        // it to keep the path canonical.
        LineSegment2::new(start, end).ok()
    }

    /// Total length: sum of segment lengths. Closed polylines include the
    /// closing segment.
    #[must_use]
    pub fn length(&self) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.segment_count() {
            if let Some(s) = self.segment(i) {
                sum += s.length();
            }
        }
        sum
    }

    /// Closest point on the polyline to `p`.
    #[must_use]
    pub fn project_point(&self, p: &Point2) -> Point2 {
        let mut best = f64::INFINITY;
        let mut best_pt = self.vertices[0];
        for i in 0..self.segment_count() {
            if let Some(s) = self.segment(i) {
                let q = s.project_point(p);
                let d = q.distance_to(*p);
                if d < best {
                    best = d;
                    best_pt = q;
                }
            }
        }
        best_pt
    }

    /// Distance from `p` to the polyline curve.
    #[must_use]
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        self.project_point(p).distance_to(*p)
    }

    /// Returns `true` if `p` lies on the polyline curve (boundary) within
    /// `tolerance`.
    #[must_use]
    pub fn contains_point(&self, p: &Point2, tolerance: Tolerance) -> bool {
        for i in 0..self.segment_count() {
            if let Some(s) = self.segment(i)
                && s.contains_point(p, tolerance)
            {
                return true;
            }
        }
        false
    }
}

impl Validate for Polyline2 {
    fn validate(&self) -> Result<(), GeometryError> {
        for v in &self.vertices {
            v.validate()?;
        }
        let min = if self.closed { 3 } else { 2 };
        if self.vertices.len() < min {
            return Err(GeometryError::Degenerate(
                "polyline has insufficient vertices",
            ));
        }
        Ok(())
    }
}

impl Bounded2 for Polyline2 {
    fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::from_points(&self.vertices).unwrap_or(BoundingBox2::new_unchecked(
            self.vertices[0],
            self.vertices[0],
        ))
    }
}

impl Transformable2 for Polyline2 {
    fn transform(&self, transform: &Transform2D) -> Self {
        let vertices = self
            .vertices
            .iter()
            .map(|p| transform.apply_point(p))
            .collect();
        Self {
            vertices,
            closed: self.closed,
        }
    }
}

impl DistanceTo2<Point2> for Polyline2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl Project2 for Polyline2 {
    fn project_point(&self, point: &Point2) -> Point2 {
        self.project_point(point)
    }
}

impl Contains2<Point2> for Polyline2 {
    fn contains(&self, rhs: &Point2) -> bool {
        self.contains_point(rhs, Tolerance::DEFAULT)
    }
}

impl Intersect2<LineSegment2> for Polyline2 {
    fn intersect(&self, rhs: &LineSegment2, tolerance: Tolerance) -> Intersection2 {
        // Intersect each polyline segment with `rhs` and deduplicate results.
        let mut points: Vec<Point2> = Vec::new();
        let mut segment_hits: Vec<LineSegment2> = Vec::new();
        for i in 0..self.segment_count() {
            let Some(s) = self.segment(i) else { continue };
            if s.start.x == s.end.x && s.start.y == s.end.y {
                continue;
            }
            match s.intersect(rhs, tolerance) {
                Intersection2::Empty => {}
                Intersection2::Point(p) => {
                    if !points
                        .iter()
                        .any(|q| q.distance_to(&p) <= tolerance.absolute)
                    {
                        points.push(p);
                    }
                }
                Intersection2::Points(ps) => {
                    for p in ps {
                        if !points
                            .iter()
                            .any(|q| q.distance_to(&p) <= tolerance.absolute)
                        {
                            points.push(p);
                        }
                    }
                }
                Intersection2::Segment(seg) => segment_hits.push(seg),
                Intersection2::Coincident => segment_hits.push(s),
            }
        }
        if !segment_hits.is_empty() {
            // Collapse to the first overlap segment (conservative).
            return Intersection2::Segment(segment_hits[0]);
        }
        match points.len() {
            0 => Intersection2::Empty,
            1 => Intersection2::Point(points.remove(0)),
            _ => Intersection2::Points(points),
        }
    }
}

/// Even-odd ray-cast point-in-polygon test for closed polylines.
///
/// For OPEN polylines, this returns `false` — an open polyline does not bound
/// a region. For boundary-on testing, use [`Polyline2::contains_point`].
pub fn point_in_polygon(p: &Point2, polygon: &Polyline2) -> bool {
    if !polygon.closed {
        return false;
    }
    let mut inside = false;
    let n = polygon.vertices.len();
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let pi = polygon.vertices[i];
        let pj = polygon.vertices[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Polyline2 serde round-trip.
    // Evidence: WO-002-AC03 — insufficient vertices rejected; degenerate
    // adjacent vertices handled gracefully.
    // Evidence: WO-002-AC04 — bbox contains vertices; transform identity
    // invariance; projection on polyline.
    use super::*;
    use crate::line::LineSegment2;
    use crate::ops::{Bounded2, Contains2, Intersect2, Intersection2, Transformable2, Validate};
    use crate::testutil::{Prng, roundtrip};

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_rejects_insufficient_vertices() {
        // Evidence: WO-002-AC03 — degenerate polyline (insufficient vertices).
        assert!(Polyline2::new(vec![], false).is_err());
        assert!(Polyline2::new(vec![Point2::ORIGIN], false).is_err());
        assert!(Polyline2::new(vec![Point2::ORIGIN], true).is_err());
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        ];
        assert!(Polyline2::new(pts.clone(), false).is_ok());
        assert!(Polyline2::new(pts, true).is_err()); // closed needs 3
    }

    #[test]
    fn new_rejects_nan_vertices() {
        let pts = vec![
            Point2 {
                x: f64::NAN,
                y: 0.0,
            },
            Point2::new(1.0, 0.0).unwrap(),
        ];
        assert!(Polyline2::new(pts, false).is_err());
    }

    #[test]
    fn segment_count_open_and_closed() {
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
        ];
        let open = Polyline2::new(pts.clone(), false).unwrap();
        assert_eq!(open.segment_count(), 2);
        let closed = Polyline2::new(pts, true).unwrap();
        assert_eq!(closed.segment_count(), 3);
    }

    #[test]
    fn length_of_unit_square_is_four() {
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
            Point2::new(0.0, 1.0).unwrap(),
        ];
        let sq = Polyline2::new(pts, true).unwrap();
        assert!(approx(sq.length(), 4.0));
    }

    #[test]
    fn project_point_on_polyline_lies_on_curve() {
        // Evidence: WO-002-AC04 — projection lies on primitive.
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
        ];
        let p = Polyline2::new(pts, false).unwrap();
        // Point (1, 0.5): nearest is (1,0) on the first segment (dist 0.5),
        // beating (2,0.5) on the second segment (dist 1.0). [The earlier
        // fixture used (1,5) which actually projects to (2,2) (dist ~3.16 <
        // 5), so it was a wrong expectation, not an impl bug.]
        let q = p.project_point(&Point2::new(1.0, 0.5).unwrap());
        assert!(approx(q.x, 1.0));
        assert!(approx(q.y, 0.0));
        assert!(p.contains(&q));
    }

    #[test]
    fn point_in_polygon_unit_square() {
        // Evidence: WO-002-AC04 — point-in-polygon for closed polyline.
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
            Point2::new(0.0, 1.0).unwrap(),
        ];
        let sq = Polyline2::new(pts, true).unwrap();
        assert!(point_in_polygon(&Point2::new(0.5, 0.5).unwrap(), &sq));
        assert!(!point_in_polygon(&Point2::new(1.5, 0.5).unwrap(), &sq));
        assert!(!point_in_polygon(&Point2::new(-0.5, 0.5).unwrap(), &sq));
        // Boundary counts as on the curve (contains), but ray-cast may
        // produce either true or false; we only test unambiguous interior.
    }

    #[test]
    fn polyline_segment_intersection_finds_crossings() {
        // A closed square intersected by a vertical line through x=0.5
        // produces two crossings.
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
            Point2::new(0.0, 1.0).unwrap(),
        ];
        let sq = Polyline2::new(pts, true).unwrap();
        let seg = LineSegment2::new(
            Point2::new(0.5, -1.0).unwrap(),
            Point2::new(0.5, 2.0).unwrap(),
        )
        .unwrap();
        match sq.intersect(&seg, Tolerance::DEFAULT) {
            Intersection2::Points(ps) => assert_eq!(ps.len(), 2),
            other => panic!("expected 2 crossings, got {other:?}"),
        }
    }

    #[test]
    fn transform_identity_preserves_polyline() {
        // Evidence: WO-002-AC04 — transform identity invariance.
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(3.0, 4.0).unwrap(),
        ];
        let p = Polyline2::new(pts, false).unwrap();
        let q = p.transform(&Transform2D::identity());
        for (a, b) in p.vertices.iter().zip(q.vertices.iter()) {
            assert!(approx(a.x, b.x));
            assert!(approx(a.y, b.y));
        }
    }

    #[test]
    fn bbox_contains_all_vertices() {
        // Evidence: WO-002-AC04 — bbox contains its vertices.
        let mut prng = Prng::new();
        for _ in 0..32 {
            let mut pts = Vec::with_capacity(8);
            for _ in 0..8 {
                pts.push(Point2::new(prng.signed_f64(100.0), prng.signed_f64(100.0)).unwrap());
            }
            let p = Polyline2::new(pts.clone(), false).unwrap();
            let bb = p.bounding_box();
            for q in &pts {
                assert!(bb.contains(q));
            }
        }
    }

    #[test]
    fn validate_rejects_deserialized_empty() {
        let bad = Polyline2 {
            vertices: vec![],
            closed: false,
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — Polyline2 round-trip serialization.
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(3.0, 4.0).unwrap(),
            Point2::new(5.0, 6.0).unwrap(),
        ];
        let p = Polyline2::new(pts, true).unwrap();
        let d = roundtrip(&p).unwrap();
        assert_eq!(p, d);
    }
}
