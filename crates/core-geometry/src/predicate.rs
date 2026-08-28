//! Deterministic 2D geometry predicates.
//!
//! Per the frozen v1.1 contract
//! (`spec/domain-model.md` §"Core value types and invariants"):
//!
//! > Geometry predicates that require robust classification MUST use an
//! > explicit tolerance policy; tolerance is never implicit or
//! > caller-chosen on a per-operation basis.
//!
//! Every predicate in this module uses the canonical singleton tolerance
//! policy [`Tolerance::CANONICAL`](crate::tolerance::Tolerance::CANONICAL)
//! internally. There is NO per-call `tol: Tolerance` parameter on any
//! predicate signature; the tolerance is not caller-chosen per-operation.

use crate::line::Line2;
use crate::line::LineSegment2;
use crate::point::Point2;
use crate::tolerance::Tolerance;

/// Orientation of three points, classified within the canonical tolerance
/// policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// The three points are (approximately) on a single line.
    Collinear,
    /// The third point lies to the right of the directed line `a → b → c`
    /// (CW orientation in standard math frame).
    Clockwise,
    /// The third point lies to the left of the directed line `a → b → c`
    /// (CCW orientation).
    CounterClockwise,
}

/// Returns the orientation of the triple `(a, b, c)` using the signed-area
/// (cross product) test. The magnitude of the cross product is compared to
/// the canonical tolerance policy
/// ([`Tolerance::CANONICAL`](crate::tolerance::Tolerance::CANONICAL)) scaled
/// by the segment length (so the test is robust to the geometry's scale).
///
/// Returns [`Orientation::Collinear`] if the cross product is within the
/// canonical tolerance of zero (after scaling), else
/// [`Orientation::CounterClockwise`] if positive or
/// [`Orientation::Clockwise`] if negative.
#[must_use]
pub fn orientation(a: &Point2, b: &Point2, c: &Point2) -> Orientation {
    // Signed area: cross(b - a, c - a). Magnitude scales with the segment
    // length; normalize by the sum of squared chord lengths to make the test
    // scale-stable.
    let tol = Tolerance::CANONICAL;
    let ab = a.vector_to(*b);
    let ac = a.vector_to(*c);
    let cross = ab.cross(ac);
    let scale = (ab.length_squared() + ac.length_squared())
        .sqrt()
        .max(tol.absolute());
    let s = cross / scale;
    if s.abs() <= tol.absolute() {
        Orientation::Collinear
    } else if s > 0.0 {
        Orientation::CounterClockwise
    } else {
        Orientation::Clockwise
    }
}

/// Returns `true` iff `(a, b, c)` are collinear within the canonical
/// tolerance policy.
#[must_use]
pub fn are_collinear(a: &Point2, b: &Point2, c: &Point2) -> bool {
    matches!(orientation(a, b, c), Orientation::Collinear)
}

/// Returns `true` iff `a` and `b` are coincident within the canonical
/// tolerance policy.
#[must_use]
pub fn are_coincident(a: &Point2, b: &Point2) -> bool {
    a.distance_to(*b) <= Tolerance::CANONICAL.absolute()
}

/// Returns `true` iff the two segments are parallel (or anti-parallel)
/// within the canonical tolerance policy.
#[must_use]
pub fn segments_parallel(s1: &LineSegment2, s2: &LineSegment2) -> bool {
    let tol = Tolerance::CANONICAL;
    let d1 = s1.start.vector_to(s1.end);
    let d2 = s2.start.vector_to(s2.end);
    let cross = d1.cross(d2).abs();
    let scale = (d1.length_squared() * d2.length_squared())
        .sqrt()
        .max(tol.absolute());
    cross / scale <= tol.absolute()
}

/// Returns `Some(true)` if `p` lies strictly to the left of `line` (CCW),
/// `Some(false)` if to the right, or `None` if `p` is on the line within
/// the canonical tolerance policy.
#[must_use]
pub fn point_left_of_line(p: &Point2, line: &Line2) -> Option<bool> {
    let tol = Tolerance::CANONICAL;
    let v = line.point.vector_to(*p);
    let cross = line.direction.cross(v);
    let scale = (v.length_squared() + 1.0).sqrt().max(tol.absolute());
    let s = cross / scale;
    if s.abs() <= tol.absolute() {
        None
    } else {
        Some(s > 0.0)
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC02 — deterministic predicates with hard-coded
    // expected outputs.
    // Evidence: WO-002-AC03 — degenerate (collinear / coincident) inputs
    // explicitly handled.
    use super::*;

    #[test]
    fn orientation_ccw_cw_collinear() {
        // CCW triangle: (0,0) -> (1,0) -> (0,1).
        let a = Point2::new(0.0, 0.0).unwrap();
        let b = Point2::new(1.0, 0.0).unwrap();
        let c = Point2::new(0.0, 1.0).unwrap();
        assert_eq!(orientation(&a, &b, &c), Orientation::CounterClockwise);
        // CW triangle: (0,0) -> (0,1) -> (1,0).
        assert_eq!(orientation(&a, &c, &b), Orientation::Clockwise);
        // Collinear: (0,0) -> (1,0) -> (2,0).
        let d = Point2::new(2.0, 0.0).unwrap();
        assert_eq!(orientation(&a, &b, &d), Orientation::Collinear);
    }

    #[test]
    fn are_collinear_predicate() {
        let a = Point2::new(0.0, 0.0).unwrap();
        let b = Point2::new(2.0, 0.0).unwrap();
        let c = Point2::new(5.0, 0.0).unwrap();
        assert!(are_collinear(&a, &b, &c));
        let d = Point2::new(5.0, 1.0).unwrap();
        assert!(!are_collinear(&a, &b, &d));
    }

    #[test]
    fn are_coincident_predicate() {
        // Evidence: WO-002-AC03 — coincident points.
        let a = Point2::new(1.0, 2.0).unwrap();
        let b = Point2::new(1.0 + 1e-12, 2.0 - 1e-12).unwrap();
        assert!(are_coincident(&a, &b));
        let c = Point2::new(1.0 + 1e-3, 2.0).unwrap();
        assert!(!are_coincident(&a, &c));
    }

    #[test]
    fn segments_parallel_predicate() {
        let s1 = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        let s2 = LineSegment2::new(
            Point2::new(0.0, 1.0).unwrap(),
            Point2::new(1.0, 1.0).unwrap(),
        )
        .unwrap();
        assert!(segments_parallel(&s1, &s2));
        let s3 = LineSegment2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(0.0, 1.0).unwrap(),
        )
        .unwrap();
        assert!(!segments_parallel(&s1, &s3));
    }

    #[test]
    fn point_left_of_line_predicate() {
        let line = Line2::from_two_points(
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 0.0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            point_left_of_line(&Point2::new(0.5, 1.0).unwrap(), &line),
            Some(true)
        );
        assert_eq!(
            point_left_of_line(&Point2::new(0.5, -1.0).unwrap(), &line),
            Some(false)
        );
        assert_eq!(
            point_left_of_line(&Point2::new(0.5, 0.0).unwrap(), &line),
            None
        );
    }
}
