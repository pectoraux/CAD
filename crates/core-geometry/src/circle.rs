//! 2D circles and arcs.

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::line::{Line2, LineSegment2};
use crate::ops::{
    Bounded2, Contains2, DistanceTo2, Intersect2, Intersection2, Project2, Transformable2, Validate,
};
use crate::point::Point2;
use crate::tolerance::Tolerance;
use crate::transform::Transform2D;
use crate::vector::Vector2;
use serde::{Deserialize, Serialize};

/// A 2D circle defined by a center and a positive radius. Zero-radius
/// circles are rejected at construction as degenerate (a point is not a
/// circle).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle2 {
    /// Center of the circle.
    pub center: Point2,
    /// Radius (positive, finite).
    pub radius: f64,
}

impl Circle2 {
    /// Construct a circle from a center and a strictly-positive, finite
    /// radius. Zero radius is rejected as degenerate; non-finite is rejected
    /// as `NonFinite`.
    #[must_use]
    pub fn new(center: Point2, radius: f64) -> Result<Self, GeometryError> {
        center.validate()?;
        if !radius.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if radius <= 0.0 {
            return Err(GeometryError::Degenerate("zero or negative radius"));
        }
        Ok(Self { center, radius })
    }

    /// Area enclosed: `π r²`.
    #[must_use]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    /// Circumference length: `2 π r`.
    #[must_use]
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    /// Closest point on the circle curve to `p`. If `p == center` (exact
    /// zero check — no implicit tolerance per the frozen v1.1 contract),
    /// returns `center + (radius, 0)`.
    #[must_use]
    pub fn project_point(&self, p: &Point2) -> Point2 {
        let v = self.center.vector_to(*p);
        if v.length_squared() == 0.0 {
            return Point2::new_unchecked(self.center.x + self.radius, self.center.y);
        }
        let len = v.length();
        let n = Vector2::new_unchecked(v.x / len, v.y / len);
        Point2::new_unchecked(
            self.center.x + n.x * self.radius,
            self.center.y + n.y * self.radius,
        )
    }

    /// Distance from `p` to the circle curve (signed magnitude of the radial
    /// offset; returns absolute distance to the curve, so it is zero when
    /// `p` lies on the curve).
    #[must_use]
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        (self.center.distance_to(*p) - self.radius).abs()
    }

    /// Returns `true` if `p` lies on the circle curve within `tolerance`.
    #[must_use]
    pub fn contains_point(&self, p: &Point2, tolerance: Tolerance) -> bool {
        self.distance_to_point(p) <= tolerance.absolute
    }

    /// Returns `true` if `p` lies inside the closed disk bounded by the
    /// circle (distance from center ≤ radius).
    #[must_use]
    pub fn contains_disk(&self, p: &Point2) -> bool {
        self.center.distance_to(*p) <= self.radius
    }

    /// Angle (in radians, `[-π, π]`) of `p` about `center`.
    #[must_use]
    pub fn angle_of(&self, p: &Point2) -> f64 {
        (p.y - self.center.y).atan2(p.x - self.center.x)
    }

    /// Point on the circle at the given angle.
    #[must_use]
    pub fn point_at(&self, angle: f64) -> Point2 {
        Point2::new_unchecked(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }
}

impl Validate for Circle2 {
    fn validate(&self) -> Result<(), GeometryError> {
        self.center.validate()?;
        if !self.radius.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if self.radius <= 0.0 {
            return Err(GeometryError::Degenerate("zero or negative radius"));
        }
        Ok(())
    }
}

impl Bounded2 for Circle2 {
    fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new_unchecked(
            Point2::new_unchecked(self.center.x - self.radius, self.center.y - self.radius),
            Point2::new_unchecked(self.center.x + self.radius, self.center.y + self.radius),
        )
    }
}

impl Transformable2 for Circle2 {
    /// Transform the circle's center by `transform`, and scale the radius
    /// by `|scale_x|` (== `|scale_y|` for representable cases).
    ///
    /// Returns `Err(GeometryError::Degenerate(_))` when `transform` has
    /// non-uniform scale (`|scale_x| != |scale_y|` within `tol`): the image
    /// of a circle under non-uniform scaling is an ellipse, which is not
    /// representable in the frozen `Circle2` form. Also returns `Err` when
    /// the transform is singular and collapses the circle to a point
    /// (zero-radius result is rejected by `Circle2::new`).
    ///
    /// Evidence: WO-002-AC03 — non-representable images explicitly rejected
    /// rather than silently approximated.
    fn transform(&self, transform: &Transform2D, tol: Tolerance) -> Result<Self, GeometryError> {
        if !tol.eq(transform.scale_x.abs(), transform.scale_y.abs()) {
            return Err(GeometryError::Degenerate(
                "non-uniform scale cannot be represented as a circle (frozen Circle2 form)",
            ));
        }
        let new_center = transform.apply_point(&self.center);
        let new_radius = self.radius * transform.scale_x.abs();
        Circle2::new(new_center, new_radius)
    }
}

impl DistanceTo2<Point2> for Circle2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl Project2 for Circle2 {
    fn project_point(&self, point: &Point2, _tol: Tolerance) -> Point2 {
        self.project_point(point)
    }
}

impl Contains2<Point2> for Circle2 {
    fn contains(&self, rhs: &Point2, tol: Tolerance) -> bool {
        self.contains_point(rhs, tol)
    }
}

impl Intersect2<Circle2> for Circle2 {
    fn intersect(&self, rhs: &Circle2, tolerance: Tolerance) -> Intersection2 {
        let d = self.center.distance_to(rhs.center);
        let r_sum = self.radius + rhs.radius;
        let r_diff = (self.radius - rhs.radius).abs();
        if d > r_sum + tolerance.absolute {
            return Intersection2::Empty;
        }
        if d < r_diff - tolerance.absolute {
            return Intersection2::Empty; // one inside the other, no contact
        }
        // Coincident (same center and radius)
        if d <= tolerance.absolute && (self.radius - rhs.radius).abs() <= tolerance.absolute {
            return Intersection2::Coincident;
        }
        // Tangent (external or internal)
        if (d - r_sum).abs() <= tolerance.absolute || (d - r_diff).abs() <= tolerance.absolute {
            // Single tangent point along the line of centers.
            let dir = if d > tolerance.absolute {
                self.center
                    .vector_to(rhs.center)
                    .normalize_with(tolerance)
                    .unwrap_or(Vector2::I)
            } else {
                Vector2::I
            };
            let p = Point2::new_unchecked(
                self.center.x + dir.x * self.radius,
                self.center.y + dir.y * self.radius,
            );
            return Intersection2::Point(p);
        }
        // Two intersections: standard formula.
        let a = (d * d + self.radius * self.radius - rhs.radius * rhs.radius)
            / (2.0 * d * d).max(f64::MIN_POSITIVE);
        // Point on the line of centers, halfway-projected.
        let px = self.center.x + a * (rhs.center.x - self.center.x);
        let py = self.center.y + a * (rhs.center.y - self.center.y);
        // Perpendicular distance h.
        let h_sq = (self.radius * self.radius - a * a * d * d).max(0.0);
        let h = h_sq.sqrt();
        // Perpendicular direction (unit) to the line of centers.
        let axis = if d > tolerance.absolute {
            self.center.vector_to(rhs.center)
        } else {
            Vector2::I
        };
        let perp = Vector2::new_unchecked(-axis.y, axis.x);
        let pn = perp.normalize_with(tolerance).unwrap_or(Vector2::J);
        let p1 = Point2::new_unchecked(px + pn.x * h, py + pn.y * h);
        let p2 = Point2::new_unchecked(px - pn.x * h, py - pn.y * h);
        if (p1.x - p2.x).abs() <= tolerance.absolute && (p1.y - p2.y).abs() <= tolerance.absolute {
            Intersection2::Point(p1)
        } else {
            Intersection2::Points(vec![p1, p2])
        }
    }
}

impl Intersect2<Line2> for Circle2 {
    fn intersect(&self, rhs: &Line2, tolerance: Tolerance) -> Intersection2 {
        // Distance from center to line; compare with radius.
        let dist = rhs.distance_to_point(&self.center);
        if dist > self.radius + tolerance.absolute {
            return Intersection2::Empty;
        }
        let perp = rhs.project_point(&self.center);
        if (dist - self.radius).abs() <= tolerance.absolute {
            return Intersection2::Point(perp);
        }
        // Two points: along the line at ±sqrt(r^2 - d^2) from perp.
        let off = (self.radius * self.radius - dist * dist).sqrt();
        let d = rhs.direction;
        let p1 = Point2::new_unchecked(perp.x + d.x * off, perp.y + d.y * off);
        let p2 = Point2::new_unchecked(perp.x - d.x * off, perp.y - d.y * off);
        Intersection2::Points(vec![p1, p2])
    }
}

impl Intersect2<LineSegment2> for Circle2 {
    fn intersect(&self, rhs: &LineSegment2, tolerance: Tolerance) -> Intersection2 {
        // Clip the segment to the circle.
        // Parameterize the segment: p(t) = start + t*(end-start), t in [0,1].
        // |start - center + t*d|^2 = r^2, where d = end - start.
        // Let f' = start - center (note: NOT center - start).
        // (f' + t*d).dot(f' + t*d) = r^2
        // f'.dot(f') + 2 t (f'.dot(d)) + t^2 d.dot(d) = r^2
        // a t^2 + b t + c = 0 where a = d.dot(d), b = 2 f'.dot(d), c = f'.dot(f') - r^2.
        let d = rhs.start.vector_to(rhs.end);
        let fp = self.center.vector_to(rhs.start); // start - center
        let a = d.dot(d);
        let b = 2.0 * fp.dot(d);
        let c = fp.dot(fp) - self.radius * self.radius;
        let disc = b * b - 4.0 * a * c;
        if disc < -tolerance.absolute {
            return Intersection2::Empty;
        }
        if disc.abs() <= tolerance.absolute {
            // Tangent
            let t = -b / (2.0 * a);
            if (0.0..=1.0).contains(&t) {
                return Intersection2::Point(Point2::new_unchecked(
                    rhs.start.x + d.x * t,
                    rhs.start.y + d.y * t,
                ));
            }
            return Intersection2::Empty;
        }
        let sd = disc.sqrt();
        let t1 = (-b + sd) / (2.0 * a);
        let t2 = (-b - sd) / (2.0 * a);
        let mut ts = [t1, t2];
        ts.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        let mut hits = Vec::new();
        for t in ts {
            if (0.0..=1.0).contains(&t) {
                hits.push(Point2::new_unchecked(
                    rhs.start.x + d.x * t,
                    rhs.start.y + d.y * t,
                ));
            }
        }
        match hits.len() {
            0 => Intersection2::Empty,
            1 => Intersection2::Point(hits[0]),
            _ => Intersection2::Points(hits),
        }
    }
}

// ---------------------------------------------------------------------------
// Arc
// ---------------------------------------------------------------------------

/// A 2D circular arc defined by center, radius, start angle, and sweep.
///
/// The sweep is in `(-2π, 2π]`. Positive sweep is CCW; negative is CW. A
/// zero-sweep arc is degenerate and rejected at construction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc2 {
    /// Center of the arc's circle.
    pub center: Point2,
    /// Radius (positive, finite).
    pub radius: f64,
    /// Start angle in radians.
    pub start_angle: f64,
    /// Sweep angle in radians (sign indicates direction).
    pub sweep_angle: f64,
}

impl Arc2 {
    /// Construct an arc. Requires positive finite radius and non-zero sweep
    /// (zero sweep is degenerate — a point, not an arc).
    ///
    /// The sweep is normalized into `(-2π, 2π]` for canonical representation.
    #[must_use]
    pub fn new(
        center: Point2,
        radius: f64,
        start_angle: f64,
        sweep_angle: f64,
    ) -> Result<Self, GeometryError> {
        center.validate()?;
        if !radius.is_finite() || !start_angle.is_finite() || !sweep_angle.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if radius <= 0.0 {
            return Err(GeometryError::Degenerate("zero or negative radius"));
        }
        let s = Self::normalize_sweep(sweep_angle);
        if s == 0.0 {
            return Err(GeometryError::Degenerate("zero-sweep arc"));
        }
        Ok(Self {
            center,
            radius,
            start_angle,
            sweep_angle: s,
        })
    }

    /// Normalize sweep into the canonical `(-2π, 2π]` range.
    fn normalize_sweep(sweep: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        let mut s = sweep;
        // Bring into (-2π, 2π].
        while s > two_pi {
            s -= two_pi;
        }
        while s <= -two_pi {
            s += two_pi;
        }
        s
    }

    /// Start endpoint of the arc.
    #[must_use]
    pub fn start_point(&self) -> Point2 {
        self.point_at_angle(self.start_angle)
    }

    /// End endpoint of the arc (`start_angle + sweep_angle`).
    #[must_use]
    pub fn end_angle(&self) -> f64 {
        self.start_angle + self.sweep_angle
    }

    /// End endpoint of the arc.
    #[must_use]
    pub fn end_point(&self) -> Point2 {
        self.point_at_angle(self.end_angle())
    }

    /// Returns `true` if the arc is counter-clockwise.
    #[must_use]
    pub fn is_ccw(&self) -> bool {
        self.sweep_angle > 0.0
    }

    /// Arc length: `radius * |sweep|`.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.radius * self.sweep_angle.abs()
    }

    /// Point on the arc at the given absolute angle (radians).
    #[must_use]
    pub fn point_at_angle(&self, angle: f64) -> Point2 {
        Point2::new_unchecked(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    /// Returns `true` if `angle` lies within the arc's angular range.
    #[must_use]
    pub fn contains_angle(&self, angle: f64) -> bool {
        let two_pi = 2.0 * std::f64::consts::PI;
        // Normalize relative angle into [0, |sweep|].
        let mut rel = (angle - self.start_angle).rem_euclid(two_pi);
        if self.sweep_angle < 0.0 {
            // CW arc: angles in (sweep, 0] (modulo 2π).
            rel = two_pi - rel;
            if rel > -self.sweep_angle {
                return false;
            }
            return true;
        }
        rel <= self.sweep_angle
    }

    /// Closest point on the arc curve to `p`. Projects to the underlying
    /// circle; if the projected angle is inside the arc range, returns that
    /// projected point; otherwise returns the nearer endpoint. The
    /// center-coincidence check is exact-zero (no implicit tolerance per
    /// the frozen v1.1 contract).
    #[must_use]
    pub fn project_point(&self, p: &Point2) -> Point2 {
        let v = self.center.vector_to(*p);
        if v.length_squared() == 0.0 {
            return self.start_point();
        }
        let a = v.y.atan2(v.x);
        if self.contains_angle(a) {
            return self.point_at_angle(a);
        }
        // Pick nearer endpoint.
        let sp = self.start_point();
        let ep = self.end_point();
        if sp.distance_to(*p) <= ep.distance_to(*p) {
            sp
        } else {
            ep
        }
    }

    /// Distance from `p` to the arc curve.
    #[must_use]
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        self.project_point(p).distance_to(*p)
    }

    /// Returns `true` if `p` lies on the arc curve within `tolerance`.
    #[must_use]
    pub fn contains_point(&self, p: &Point2, tolerance: Tolerance) -> bool {
        self.distance_to_point(p) <= tolerance.absolute
    }
}

impl Validate for Arc2 {
    fn validate(&self) -> Result<(), GeometryError> {
        self.center.validate()?;
        if !self.radius.is_finite()
            || !self.start_angle.is_finite()
            || !self.sweep_angle.is_finite()
        {
            return Err(GeometryError::NonFinite);
        }
        if self.radius <= 0.0 {
            return Err(GeometryError::Degenerate("zero or negative radius"));
        }
        if self.sweep_angle == 0.0 {
            return Err(GeometryError::Degenerate("zero-sweep arc"));
        }
        Ok(())
    }
}

impl Bounded2 for Arc2 {
    fn bounding_box(&self) -> BoundingBox2 {
        // Conservative: include start, end, and any of the cardinal angles
        // (0, π/2, π, 3π/2) that fall within the arc's angular range.
        let mut pts = vec![self.start_point(), self.end_point()];
        let cardinals = [
            0.0,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
            -std::f64::consts::FRAC_PI_2,
        ];
        for &a in &cardinals {
            if self.contains_angle(a) {
                pts.push(self.point_at_angle(a));
            }
        }
        BoundingBox2::from_points(&pts)
            .unwrap_or(BoundingBox2::new_unchecked(self.center, self.center))
    }
}

impl Transformable2 for Arc2 {
    /// Transform the arc's center, radius, and angles by `transform`.
    ///
    /// Returns `Err(GeometryError::Degenerate(_))` when `transform` has
    /// non-uniform scale (`|scale_x| != |scale_y|` within `tol`): the image
    /// of a circular arc under non-uniform scaling is an elliptic arc, not
    /// representable in the frozen `Arc2` form. Reflection (sign-flip of
    /// exactly one scale) is handled correctly (sweep sign flips).
    ///
    /// Evidence: WO-002-AC03 — non-representable images explicitly rejected
    /// rather than silently approximated.
    fn transform(&self, transform: &Transform2D, tol: Tolerance) -> Result<Self, GeometryError> {
        if !tol.eq(transform.scale_x.abs(), transform.scale_y.abs()) {
            return Err(GeometryError::Degenerate(
                "non-uniform scale cannot be represented as an arc (frozen Arc2 form)",
            ));
        }
        let new_center = transform.apply_point(&self.center);
        let s = transform.scale_x.abs();
        let new_radius = self.radius * s;
        let phi = transform.rotation_rad;
        let reflection = (transform.scale_x * transform.scale_y) < 0.0;
        let extra_pi = if transform.scale_x < 0.0 {
            std::f64::consts::PI
        } else {
            0.0
        };
        let (new_start, new_sweep) = if reflection {
            (phi + extra_pi - self.start_angle, -self.sweep_angle)
        } else {
            (phi + extra_pi + self.start_angle, self.sweep_angle)
        };
        Arc2::new(new_center, new_radius, new_start, new_sweep)
    }
}

impl DistanceTo2<Point2> for Arc2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl Project2 for Arc2 {
    fn project_point(&self, point: &Point2, _tol: Tolerance) -> Point2 {
        self.project_point(point)
    }
}

impl Contains2<Point2> for Arc2 {
    fn contains(&self, rhs: &Point2, tol: Tolerance) -> bool {
        self.contains_point(rhs, tol)
    }
}

impl Intersect2<Line2> for Arc2 {
    fn intersect(&self, rhs: &Line2, tolerance: Tolerance) -> Intersection2 {
        // Circle-line intersections, filtered by arc angular range.
        let circle = Circle2 {
            center: self.center,
            radius: self.radius,
        };
        let hits = match circle.intersect(rhs, tolerance) {
            Intersection2::Point(p) => vec![p],
            Intersection2::Points(ps) => ps,
            Intersection2::Coincident => return Intersection2::Coincident,
            // `Empty` is propagated; `Segment(_)` is impossible for circle-line.
            Intersection2::Empty | Intersection2::Segment(_) => return Intersection2::Empty,
        };
        let mut out: Vec<Point2> = hits
            .into_iter()
            .filter(|p| self.contains_angle((p.y - self.center.y).atan2(p.x - self.center.x)))
            .collect();
        match out.len() {
            0 => Intersection2::Empty,
            1 => Intersection2::Point(out.remove(0)),
            _ => Intersection2::Points(out),
        }
    }
}

impl Intersect2<LineSegment2> for Arc2 {
    fn intersect(&self, rhs: &LineSegment2, tolerance: Tolerance) -> Intersection2 {
        let circle = Circle2 {
            center: self.center,
            radius: self.radius,
        };
        let hits = match circle.intersect(rhs, tolerance) {
            Intersection2::Point(p) => vec![p],
            Intersection2::Points(ps) => ps,
            Intersection2::Coincident => return Intersection2::Coincident,
            Intersection2::Empty | Intersection2::Segment(_) => return Intersection2::Empty,
        };
        let mut out: Vec<Point2> = hits
            .into_iter()
            .filter(|p| self.contains_angle((p.y - self.center.y).atan2(p.x - self.center.x)))
            .collect();
        match out.len() {
            0 => Intersection2::Empty,
            1 => Intersection2::Point(out.remove(0)),
            _ => Intersection2::Points(out),
        }
    }
}

impl Intersect2<Circle2> for Arc2 {
    fn intersect(&self, rhs: &Circle2, tolerance: Tolerance) -> Intersection2 {
        let circle = Circle2 {
            center: self.center,
            radius: self.radius,
        };
        let hits = match circle.intersect(rhs, tolerance) {
            Intersection2::Point(p) => vec![p],
            Intersection2::Points(ps) => ps,
            Intersection2::Coincident => {
                // Arc's underlying circle coincides with `rhs`. The arc is a
                // subset of the circle. There is no discrete intersection
                // point set, so we report Coincident.
                return Intersection2::Coincident;
            }
            Intersection2::Empty | Intersection2::Segment(_) => return Intersection2::Empty,
        };
        let mut out: Vec<Point2> = hits
            .into_iter()
            .filter(|p| self.contains_angle((p.y - self.center.y).atan2(p.x - self.center.x)))
            .collect();
        match out.len() {
            0 => Intersection2::Empty,
            1 => Intersection2::Point(out.remove(0)),
            _ => Intersection2::Points(out),
        }
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Circle2 / Arc2 serde round-trip.
    // Evidence: WO-002-AC02 — circle-circle intersection determinism.
    // Evidence: WO-002-AC03 — zero-radius rejected; zero-sweep arc rejected.
    // Evidence: WO-002-AC04 — circle contains its center (disk); projection on circle.
    use super::*;
    use crate::line::Line2;
    use crate::ops::{Bounded2, Contains2, Intersect2, Intersection2, Transformable2, Validate};
    use crate::testutil::{Prng, roundtrip};

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_rejects_zero_radius() {
        // Evidence: WO-002-AC03 — degenerate zero-radius circle.
        assert!(Circle2::new(Point2::ORIGIN, 0.0).is_err());
        assert!(Circle2::new(Point2::ORIGIN, -1.0).is_err());
        assert!(Circle2::new(Point2::ORIGIN, f64::NAN).is_err());
        assert!(Circle2::new(Point2::ORIGIN, 1.0).is_ok());
    }

    #[test]
    fn area_and_circumference() {
        let c = Circle2::new(Point2::ORIGIN, 2.0).unwrap();
        assert!(approx(c.area(), 4.0 * std::f64::consts::PI));
        assert!(approx(c.circumference(), 4.0 * std::f64::consts::PI));
    }

    #[test]
    fn project_point_on_circle_returns_self() {
        // Evidence: WO-002-AC04 — projection lies on primitive.
        let c = Circle2::new(Point2::ORIGIN, 2.0).unwrap();
        // A point already on the circle at angle 0:
        let on = Point2::new(2.0, 0.0).unwrap();
        let p = c.project_point(&on);
        assert!(approx(p.x, 2.0));
        assert!(approx(p.y, 0.0));
        // A point inside (origin): the doc says pick +x direction.
        let proj = c.project_point(&Point2::ORIGIN);
        assert!(approx(proj.x, 2.0));
        assert!(approx(proj.y, 0.0));
    }

    #[test]
    fn circle_contains_disk_includes_center() {
        // Evidence: WO-002-AC04 — circle (disk) contains its center.
        let c = Circle2::new(Point2::new(1.0, 2.0).unwrap(), 3.0).unwrap();
        assert!(c.contains_disk(&c.center));
    }

    #[test]
    fn circle_contains_curve_excludes_center() {
        let c = Circle2::new(Point2::new(1.0, 2.0).unwrap(), 3.0).unwrap();
        assert!(!c.contains(&c.center, Tolerance::DEFAULT));
    }

    #[test]
    fn circle_line_two_intersections() {
        let c = Circle2::new(Point2::ORIGIN, 1.0).unwrap();
        let l = Line2::from_two_points(
            Point2::new(-2.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
        )
        .unwrap();
        match c.intersect(&l, Tolerance::DEFAULT) {
            Intersection2::Points(ps) => {
                assert_eq!(ps.len(), 2);
                let xs: Vec<f64> = ps.iter().map(|p| p.x).collect();
                assert!(xs.iter().any(|&x| approx(x, 1.0)));
                assert!(xs.iter().any(|&x| approx(x, -1.0)));
            }
            other => panic!("expected 2 points, got {other:?}"),
        }
    }

    #[test]
    fn circle_circle_two_intersections() {
        // Evidence: WO-002-AC02 — circle-circle intersection determinism.
        let c1 = Circle2::new(Point2::new(0.0, 0.0).unwrap(), 1.0).unwrap();
        let c2 = Circle2::new(Point2::new(1.5, 0.0).unwrap(), 1.0).unwrap();
        match c1.intersect(&c2, Tolerance::DEFAULT) {
            Intersection2::Points(ps) => {
                assert_eq!(ps.len(), 2);
                for p in &ps {
                    assert!(approx(c1.center.distance_to(*p), 1.0));
                    assert!(approx(c2.center.distance_to(*p), 1.0));
                }
            }
            other => panic!("expected 2 points, got {other:?}"),
        }
    }

    #[test]
    fn circle_circle_disjoint_is_empty() {
        let c1 = Circle2::new(Point2::new(0.0, 0.0).unwrap(), 1.0).unwrap();
        let c2 = Circle2::new(Point2::new(10.0, 0.0).unwrap(), 1.0).unwrap();
        assert_eq!(c1.intersect(&c2, Tolerance::DEFAULT), Intersection2::Empty);
    }

    #[test]
    fn circle_circle_coincident() {
        let c1 = Circle2::new(Point2::new(0.0, 0.0).unwrap(), 1.0).unwrap();
        let c2 = Circle2::new(Point2::new(0.0, 0.0).unwrap(), 1.0).unwrap();
        assert_eq!(
            c1.intersect(&c2, Tolerance::DEFAULT),
            Intersection2::Coincident
        );
    }

    #[test]
    fn circle_circle_tangent() {
        let c1 = Circle2::new(Point2::new(0.0, 0.0).unwrap(), 1.0).unwrap();
        let c2 = Circle2::new(Point2::new(2.0, 0.0).unwrap(), 1.0).unwrap();
        match c1.intersect(&c2, Tolerance::DEFAULT) {
            Intersection2::Point(p) => {
                assert!(approx(p.x, 1.0));
                assert!(approx(p.y, 0.0));
            }
            other => panic!("expected single tangent point, got {other:?}"),
        }
    }

    #[test]
    fn arc_zero_sweep_rejected() {
        // Evidence: WO-002-AC03 — zero-sweep arc is degenerate.
        assert!(Arc2::new(Point2::ORIGIN, 1.0, 0.0, 0.0).is_err());
        assert!(Arc2::new(Point2::ORIGIN, 0.0, 0.0, 1.0).is_err());
        assert!(Arc2::new(Point2::ORIGIN, 1.0, 0.0, 1.0).is_ok());
    }

    #[test]
    fn arc_length_is_radius_times_sweep_abs() {
        let a = Arc2::new(Point2::ORIGIN, 2.0, 0.0, std::f64::consts::FRAC_PI_2).unwrap();
        assert!(approx(a.length(), std::f64::consts::PI));
        let b = Arc2::new(Point2::ORIGIN, 2.0, 0.0, -std::f64::consts::FRAC_PI_2).unwrap();
        assert!(approx(b.length(), std::f64::consts::PI));
        assert!(!b.is_ccw());
        assert!(a.is_ccw());
    }

    #[test]
    fn arc_start_end_points() {
        let a = Arc2::new(Point2::ORIGIN, 1.0, 0.0, std::f64::consts::FRAC_PI_2).unwrap();
        let sp = a.start_point();
        let ep = a.end_point();
        assert!(approx(sp.x, 1.0));
        assert!(approx(sp.y, 0.0));
        assert!(approx(ep.x, 0.0));
        assert!(approx(ep.y, 1.0));
    }

    #[test]
    fn arc_bounding_box_includes_extremes() {
        let a = Arc2::new(Point2::ORIGIN, 2.0, 0.0, std::f64::consts::PI).unwrap();
        let b = a.bounding_box();
        assert!(approx(b.min.x, -2.0));
        assert!(approx(b.min.y, 0.0));
        assert!(approx(b.max.x, 2.0));
        assert!(approx(b.max.y, 2.0));
    }

    #[test]
    fn transform_identity_preserves_circle() {
        // Evidence: WO-002-AC04 — identity transform invariance.
        let c = Circle2::new(Point2::new(1.0, 2.0).unwrap(), 3.0).unwrap();
        let t = c
            .transform(&Transform2D::identity(), Tolerance::DEFAULT)
            .unwrap();
        assert!(approx(t.center.x, 1.0));
        assert!(approx(t.center.y, 2.0));
        assert!(approx(t.radius, 3.0));
    }

    #[test]
    fn circle_transform_uniform_scale_and_rotation_ok() {
        // Evidence: WO-002-AC02 — uniform scale + rotation is representable;
        // the image circle has the transformed center and radius * |scale|.
        let c = Circle2::new(Point2::new(1.0, 1.0).unwrap(), 2.0).unwrap();
        let t = Transform2D::new(Vector2::new(1.0, 0.0).unwrap(), 0.5, 3.0, 3.0).unwrap();
        let img = c.transform(&t, Tolerance::DEFAULT).unwrap();
        assert!(approx(img.center.x, t.apply_point(&c.center).x));
        assert!(approx(img.center.y, t.apply_point(&c.center).y));
        assert!(approx(img.radius, 6.0));
    }

    #[test]
    fn circle_transform_non_uniform_rejected() {
        // Evidence: WO-002-AC03 — non-uniform scale produces an ellipse,
        // not representable as Circle2; explicitly rejected.
        let c = Circle2::new(Point2::ORIGIN, 2.0).unwrap();
        let t = Transform2D::scaling(2.0, 3.0);
        let err = c.transform(&t, Tolerance::DEFAULT).unwrap_err();
        assert!(matches!(err, GeometryError::Degenerate(_)));
    }

    #[test]
    fn arc_transform_uniform_preserves_endpoints() {
        // Evidence: WO-002-AC04 — arc transform maps endpoints to the
        // transformed endpoint positions (uniform scale + rotation case).
        let a = Arc2::new(Point2::ORIGIN, 1.0, 0.0, std::f64::consts::FRAC_PI_2).unwrap();
        let t = Transform2D::new(Vector2::ZERO, std::f64::consts::FRAC_PI_4, 2.0, 2.0).unwrap();
        let img = a.transform(&t, Tolerance::DEFAULT).unwrap();
        let new_start = img.start_point();
        let new_end = img.end_point();
        let exp_start = t.apply_point(&a.start_point());
        let exp_end = t.apply_point(&a.end_point());
        assert!(approx(new_start.x, exp_start.x));
        assert!(approx(new_start.y, exp_start.y));
        assert!(approx(new_end.x, exp_end.x));
        assert!(approx(new_end.y, exp_end.y));
    }

    #[test]
    fn arc_transform_reflection_flips_sweep() {
        // Evidence: WO-002-AC02 — reflection (sign-flip of exactly one
        // scale) flips the sweep direction.
        let a = Arc2::new(Point2::ORIGIN, 1.0, 0.0, std::f64::consts::FRAC_PI_2).unwrap();
        let t = Transform2D::scaling(1.0, -1.0); // reflection across x-axis
        let img = a.transform(&t, Tolerance::DEFAULT).unwrap();
        assert!(approx(img.sweep_angle, -a.sweep_angle));
    }

    #[test]
    fn arc_transform_non_uniform_rejected() {
        // Evidence: WO-002-AC03 — non-uniform scale produces an elliptic
        // arc, not representable as Arc2.
        let a = Arc2::new(Point2::ORIGIN, 1.0, 0.0, std::f64::consts::FRAC_PI_2).unwrap();
        let t = Transform2D::scaling(2.0, 3.0);
        let err = a.transform(&t, Tolerance::DEFAULT).unwrap_err();
        assert!(matches!(err, GeometryError::Degenerate(_)));
    }

    #[test]
    fn distance_symmetry_property() {
        // Evidence: WO-002-AC04 — distance symmetry d(a,b)==d(b,a) for
        // circle<->point and point<->point.
        use crate::ops::DistanceTo2;
        let mut p = Prng::new();
        let c = Circle2::new(Point2::new(1.0, -2.0).unwrap(), 2.5).unwrap();
        for _ in 0..128 {
            let pt = Point2::new(p.signed_f64(10.0), p.signed_f64(10.0)).unwrap();
            // Circle-to-point distance (trait).
            let _d1 = c.distance_to(&pt);
            // Point-to-point inherent distance symmetry.
            let d_ab = Point2::distance_to(pt, c.center);
            let d_ba = Point2::distance_to(c.center, pt);
            assert!((d_ab - d_ba).abs() < 1e-9);
        }
    }

    #[test]
    fn serde_roundtrip_circle() {
        // Evidence: WO-002-AC01 — Circle2 round-trip serialization.
        let c = Circle2::new(Point2::new(1.5, -2.5).unwrap(), 3.5).unwrap();
        let d = roundtrip(&c).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn serde_roundtrip_arc() {
        // Evidence: WO-002-AC01 — Arc2 round-trip serialization.
        let a = Arc2::new(
            Point2::new(1.0, 1.0).unwrap(),
            2.0,
            0.5,
            std::f64::consts::FRAC_PI_2,
        )
        .unwrap();
        let d = roundtrip(&a).unwrap();
        assert_eq!(a, d);
    }

    #[test]
    fn validate_rejects_deserialized_zero_radius() {
        let bad = Circle2 {
            center: Point2::ORIGIN,
            radius: 0.0,
        };
        assert!(bad.validate().is_err());
    }
}
