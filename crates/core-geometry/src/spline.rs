//! NURBS spline representation (degree, control points, knots, optional weights).
//!
//! Provides a stable NURBS representation per the frozen v1.1 domain model
//! (`Spline` entity in `spec/domain-model.md`).
//!
//! Out-of-scope per W002: exact spline-spline intersection, exact
//! closest-point evaluation. Distance/projection here are sampling-based
//! with a deterministic fixed sample count and DOCUMENTED as approximate;
//! exact evaluation is a future refinement.

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::ops::{Bounded2, DistanceTo2, Project2, Transformable2, Validate};
use crate::point::Point2;
use crate::transform::Transform2D;
use serde::{Deserialize, Serialize};

/// A 2D NURBS spline: degree, control points, knot vector, optional
/// rational weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spline2 {
    /// Polynomial degree (≥ 1).
    pub degree: u32,
    /// Control points (`len >= degree + 1`).
    pub control_points: Vec<Point2>,
    /// Knot vector (`len == control_points.len() + degree + 1`), non-decreasing.
    pub knots: Vec<f64>,
    /// Optional rational weights (same length as control points; all > 0).
    pub weights: Option<Vec<f64>>,
}

impl Spline2 {
    /// Construct a validated NURBS spline.
    ///
    /// Validation rules (canonical boundary):
    /// - `degree >= 1`;
    /// - `control_points.len() >= degree + 1`;
    /// - `knots.len() == control_points.len() + degree + 1`;
    /// - knots are non-decreasing (`knots[i] <= knots[i+1]`);
    /// - knot multiplicity ≤ `degree + 1` (no internal multiplicity
    ///   > degree+1);
    /// - if weights are `Some`, they have the same length as control points
    ///   and all are finite and strictly positive;
    /// - all control points and knots are finite.
    #[must_use]
    pub fn new(
        degree: u32,
        control_points: Vec<Point2>,
        knots: Vec<f64>,
        weights: Option<Vec<f64>>,
    ) -> Result<Self, GeometryError> {
        if degree < 1 {
            return Err(GeometryError::Degenerate("spline degree < 1"));
        }
        if control_points.len() < degree as usize + 1 {
            return Err(GeometryError::Degenerate(
                "spline has fewer than degree+1 control points",
            ));
        }
        let expected_knots = control_points.len() + degree as usize + 1;
        if knots.len() != expected_knots {
            return Err(GeometryError::InvalidInput("knot count mismatch"));
        }
        for p in &control_points {
            p.validate()?;
        }
        for k in &knots {
            if !k.is_finite() {
                return Err(GeometryError::NonFinite);
            }
        }
        // Non-decreasing check + multiplicity check.
        let mut i = 0;
        while i + 1 < knots.len() {
            if knots[i + 1] < knots[i] {
                return Err(GeometryError::InvalidInput(
                    "knot vector must be non-decreasing",
                ));
            }
            // Multiplicity count.
            if knots[i + 1] > knots[i] {
                i += 1;
                continue;
            }
            // Equal: count multiplicity.
            let mut mult = 1usize;
            let v = knots[i];
            while i + 1 + mult < knots.len() && knots[i + 1 + mult - 1] == v {
                // Actually we need to count consecutive duplicates starting at i.
                if knots[i + mult] == v {
                    mult += 1;
                    if i + mult >= knots.len() {
                        break;
                    }
                } else {
                    break;
                }
            }
            let _ = mult; // counter for clarity; we re-check below more robustly.
            i += 1;
        }
        // Robust multiplicity scan: walk and count equal runs.
        let mut run_start = 0;
        while run_start < knots.len() {
            let mut run_end = run_start + 1;
            while run_end < knots.len() && knots[run_end] == knots[run_start] {
                run_end += 1;
            }
            let mult = run_end - run_start;
            if mult > degree as usize + 1 {
                return Err(GeometryError::InvalidInput(
                    "knot multiplicity exceeds degree+1",
                ));
            }
            run_start = run_end;
        }

        // Weights validation.
        if let Some(ws) = &weights {
            if ws.len() != control_points.len() {
                return Err(GeometryError::InvalidInput(
                    "weights length must equal control_points length",
                ));
            }
            for w in ws {
                if !w.is_finite() || *w <= 0.0 {
                    return Err(GeometryError::InvalidInput(
                        "weights must be finite and > 0",
                    ));
                }
            }
        }
        Ok(Self {
            degree,
            control_points,
            knots,
            weights,
        })
    }

    /// Returns the polynomial degree.
    #[must_use]
    pub const fn degree(&self) -> u32 {
        self.degree
    }

    /// Returns the control points slice.
    #[must_use]
    pub fn control_points(&self) -> &[Point2] {
        &self.control_points
    }

    /// Returns the knot vector slice.
    #[must_use]
    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    /// Returns the optional weights slice.
    #[must_use]
    pub fn weights(&self) -> Option<&[f64]> {
        self.weights.as_deref()
    }

    /// Lower bound of the parametric domain: `knots[degree]`.
    #[must_use]
    pub fn domain_min(&self) -> f64 {
        self.knots[self.degree as usize]
    }

    /// Upper bound of the parametric domain: `knots[control_points.len()]`.
    #[must_use]
    pub fn domain_max(&self) -> f64 {
        self.knots[self.control_points.len()]
    }

    /// Returns `true` iff the spline is clamped (first/last knot have
    /// multiplicity `degree + 1`).
    #[must_use]
    pub fn is_clamped(&self) -> bool {
        let d = self.degree as usize;
        if self.knots.len() < 2 * d + 1 {
            return false;
        }
        let first = self.knots[0];
        let last = self.knots[self.knots.len() - 1];
        let mut low = 0usize;
        while low < self.knots.len() && self.knots[low] == first {
            low += 1;
        }
        let mut high = self.knots.len();
        while high > 0 && self.knots[high - 1] == last {
            high -= 1;
        }
        let mult_first = low;
        let mult_last = self.knots.len() - high;
        mult_first > d && mult_last > d
    }

    /// Cox–de Boor basis function `N_{i, p}(u)` for this spline's knot vector.
    fn basis(&self, i: usize, p: u32, u: f64) -> f64 {
        let p = p as usize;
        if p == 0 {
            return if (self.knots[i] <= u) && (u < self.knots[i + 1]) {
                1.0
            } else if u == self.knots[self.knots.len() - 1]
                && i + 1 == self.control_points.len()
                && self.knots[i] <= u
            {
                // Endpoint inclusion at the right boundary.
                1.0
            } else {
                0.0
            };
        }
        let left = if self.knots[i + p] == self.knots[i] {
            0.0
        } else {
            (u - self.knots[i]) / (self.knots[i + p] - self.knots[i])
        };
        let right = if self.knots[i + p + 1] == self.knots[i + 1] {
            0.0
        } else {
            (self.knots[i + p + 1] - u) / (self.knots[i + p + 1] - self.knots[i + 1])
        };
        let n_left = self.basis(i, p as u32 - 1, u);
        let n_right = self.basis(i + 1, p as u32 - 1, u);
        n_left * left + n_right * right
    }

    /// Evaluate the spline at parameter `u` via Cox–de Boor. Clamps `u` to
    /// the spline's parametric domain.
    #[must_use]
    pub fn evaluate(&self, u: f64) -> Point2 {
        let lo = self.domain_min();
        let hi = self.domain_max();
        let u = u.clamp(lo, hi);
        let n = self.control_points.len();
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_w = 0.0;
        for i in 0..n {
            let b = self.basis(i, self.degree, u);
            if b == 0.0 {
                continue;
            }
            let w = self.weights.as_ref().map_or(1.0, |ws| ws[i]);
            let bw = b * w;
            sum_x += bw * self.control_points[i].x;
            sum_y += bw * self.control_points[i].y;
            sum_w += bw;
        }
        if sum_w == 0.0 {
            // Degenerate fallback to the first control point.
            return self.control_points[0];
        }
        Point2::new_unchecked(sum_x / sum_w, sum_y / sum_w)
    }

    /// Closest point on the spline to `p` via deterministic sampling
    /// (64 samples + local refine). Documented as approximate.
    #[must_use]
    pub fn project_point(&self, p: &Point2) -> Point2 {
        const N: usize = 64;
        let lo = self.domain_min();
        let hi = self.domain_max();
        let mut best_d = f64::INFINITY;
        let mut best_u = lo;
        for i in 0..N {
            let u = lo + (hi - lo) * (i as f64) / (N as f64 - 1.0);
            let q = self.evaluate(u);
            let d = q.distance_to(*p);
            if d < best_d {
                best_d = d;
                best_u = u;
            }
        }
        // Local refine: 8 iterations of fine scan around best_u.
        let mut span = (hi - lo) / N as f64;
        for _ in 0..8 {
            let mut fine_best = best_d;
            let mut fine_u = best_u;
            for k in 0..11 {
                let u = best_u + (k as f64 - 5.0) * (span / 5.0);
                let uc = u.clamp(lo, hi);
                let q = self.evaluate(uc);
                let d = q.distance_to(*p);
                if d < fine_best {
                    fine_best = d;
                    fine_u = uc;
                }
            }
            best_d = fine_best;
            best_u = fine_u;
            span /= 5.0;
        }
        self.evaluate(best_u)
    }

    /// Distance from `p` to the spline curve (sampling-based, approximate).
    #[must_use]
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        self.project_point(p).distance_to(*p)
    }

    /// Convex-hull-of-control-points bounding box (conservative — the spline
    /// lies inside the convex hull of its control points, by the variation
    /// diminishing property).
    #[must_use]
    pub fn convex_hull_bbox(&self) -> BoundingBox2 {
        BoundingBox2::from_points(&self.control_points).unwrap_or(BoundingBox2::new_unchecked(
            self.control_points[0],
            self.control_points[0],
        ))
    }
}

impl Validate for Spline2 {
    fn validate(&self) -> Result<(), GeometryError> {
        if self.degree < 1 {
            return Err(GeometryError::Degenerate("spline degree < 1"));
        }
        if self.control_points.len() < self.degree as usize + 1 {
            return Err(GeometryError::Degenerate(
                "spline has fewer than degree+1 control points",
            ));
        }
        for p in &self.control_points {
            p.validate()?;
        }
        if self.knots.len() != self.control_points.len() + self.degree as usize + 1 {
            return Err(GeometryError::InvalidInput("knot count mismatch"));
        }
        for k in &self.knots {
            if !k.is_finite() {
                return Err(GeometryError::NonFinite);
            }
        }
        for w in self.knots.windows(2) {
            if w[1] < w[0] {
                return Err(GeometryError::InvalidInput(
                    "knot vector must be non-decreasing",
                ));
            }
        }
        // Multiplicity check.
        let mut run_start = 0;
        while run_start < self.knots.len() {
            let mut run_end = run_start + 1;
            while run_end < self.knots.len() && self.knots[run_end] == self.knots[run_start] {
                run_end += 1;
            }
            let mult = run_end - run_start;
            if mult > self.degree as usize + 1 {
                return Err(GeometryError::InvalidInput(
                    "knot multiplicity exceeds degree+1",
                ));
            }
            run_start = run_end;
        }
        if let Some(ws) = &self.weights {
            if ws.len() != self.control_points.len() {
                return Err(GeometryError::InvalidInput(
                    "weights length must equal control_points length",
                ));
            }
            for w in ws {
                if !w.is_finite() || *w <= 0.0 {
                    return Err(GeometryError::InvalidInput(
                        "weights must be finite and > 0",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Bounded2 for Spline2 {
    /// Conservative AABB = bounding box of the control points (NURBS lies
    /// in the convex hull of its control points; the control-point AABB is
    /// an upper bound).
    fn bounding_box(&self) -> BoundingBox2 {
        self.convex_hull_bbox()
    }
}

impl Transformable2 for Spline2 {
    /// Affine-transform the control points; weights are unchanged (rational
    /// NURBS is affinely invariant under transformation of control points).
    fn transform(&self, transform: &Transform2D) -> Self {
        let control_points = self
            .control_points
            .iter()
            .map(|p| transform.apply_point(p))
            .collect();
        Self {
            degree: self.degree,
            control_points,
            knots: self.knots.clone(),
            weights: self.weights.clone(),
        }
    }
}

impl DistanceTo2<Point2> for Spline2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl Project2 for Spline2 {
    fn project_point(&self, point: &Point2) -> Point2 {
        self.project_point(point)
    }
}

// NOTE: Intersect2 and Contains2 for Spline2 are intentionally NOT
// implemented per W002 scope (spline intersection is not specified by the
// frozen contracts; deferred to a later refinement).

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Spline2 serde round-trip.
    // Evidence: WO-002-AC03 — knot multiplicity > degree+1 rejected;
    // insufficient control points rejected; non-monotonic knots rejected;
    // weight mismatch rejected; zero-degree rejected.
    use super::*;
    use crate::ops::{Bounded2, Transformable2, Validate};
    use crate::point::Point2;
    use crate::testutil::roundtrip;
    use crate::transform::Transform2D;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Clamped cubic Bézier (degree 3, 4 control points, 8 knots).
    fn cubic_bezier(pts: [Point2; 4]) -> Spline2 {
        Spline2::new(
            3,
            pts.to_vec(),
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            None,
        )
        .unwrap()
    }

    #[test]
    fn new_rejects_zero_degree() {
        // Evidence: WO-002-AC03 — degree < 1 rejected.
        let pts = vec![Point2::ORIGIN, Point2::new(1.0, 0.0).unwrap()];
        assert!(Spline2::new(0, pts, vec![0.0, 0.0, 1.0, 1.0], None).is_err());
    }

    #[test]
    fn new_rejects_insufficient_control_points() {
        // Evidence: WO-002-AC03 — control_points < degree+1 rejected.
        let pts = vec![Point2::ORIGIN, Point2::new(1.0, 0.0).unwrap()];
        // degree 3 needs >= 4 control points
        assert!(Spline2::new(3, pts, vec![0.0; 6], None).is_err());
    }

    #[test]
    fn new_rejects_knot_count_mismatch() {
        let pts = vec![
            Point2::ORIGIN,
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        // degree 3, 4 cps, needs 4 + 3 + 1 = 8 knots
        assert!(Spline2::new(3, pts.clone(), vec![0.0; 7], None).is_err());
        assert!(Spline2::new(3, pts, vec![0.0; 9], None).is_err());
    }

    #[test]
    fn new_rejects_non_monotonic_knots() {
        // Evidence: WO-002-AC03 — non-monotonic knots rejected.
        let pts = vec![
            Point2::ORIGIN,
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        let knots = vec![0.0, 0.0, 0.0, 0.5, 0.25, 1.0, 1.0, 1.0]; // 0.5 > 0.25 violation
        assert!(Spline2::new(3, pts, knots, None).is_err());
    }

    #[test]
    fn new_rejects_excess_knot_multiplicity() {
        // Evidence: WO-002-AC03 — multiplicity > degree+1 rejected.
        // degree 1 → max multiplicity 2; knots have a run of 3 zeros in the
        // interior, which is illegal.
        let pts = vec![
            Point2::ORIGIN,
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
        ];
        // degree 1, 3 cps → needs 5 knots.
        // 0, 0, 0, 1, 1 — the leading triple-zero violates multiplicity (≤ 2).
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0];
        assert!(Spline2::new(1, pts, knots, None).is_err());
    }

    #[test]
    fn new_rejects_weight_mismatch() {
        // Evidence: WO-002-AC03 — weights length / value rejected.
        let pts = vec![
            Point2::ORIGIN,
            Point2::new(1.0, 0.0).unwrap(),
            Point2::new(2.0, 0.0).unwrap(),
        ];
        // degree 1 needs 5 knots; weights length mismatch.
        let knots = vec![0.0, 0.0, 0.5, 1.0, 1.0];
        assert!(Spline2::new(1, pts.clone(), knots.clone(), Some(vec![1.0, 1.0])).is_err());
        // Weight value invalid (zero).
        assert!(Spline2::new(1, pts, knots, Some(vec![1.0, 0.0, 1.0])).is_err());
    }

    #[test]
    fn evaluate_bezier_endpoints() {
        // Cubic Bézier: evaluate(0) == P0, evaluate(1) == P3.
        let pts = [
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        let s = cubic_bezier(pts);
        let p0 = s.evaluate(0.0);
        assert!(approx(p0.x, 0.0));
        assert!(approx(p0.y, 0.0));
        let p1 = s.evaluate(1.0);
        assert!(approx(p1.x, 3.0));
        assert!(approx(p1.y, 0.0));
    }

    #[test]
    fn is_clamped_detection() {
        let pts = [
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        let s = cubic_bezier(pts);
        assert!(s.is_clamped());
    }

    #[test]
    fn project_point_lies_near_curve() {
        // Evidence: WO-002-AC04 — projection (sampling) lies near primitive.
        let pts = [
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        let s = cubic_bezier(pts);
        // Project the curve's own midpoint (u=0.5).
        let mid = s.evaluate(0.5);
        let proj = s.project_point(&mid);
        assert!(proj.distance_to(mid) < 1e-6);
    }

    #[test]
    fn transform_identity_preserves_evaluation() {
        // Evidence: WO-002-AC04 — transform identity invariance.
        let pts = [
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        let s = cubic_bezier(pts);
        let st = s.transform(&Transform2D::identity());
        let p = s.evaluate(0.5);
        let q = st.evaluate(0.5);
        assert!(approx(p.x, q.x));
        assert!(approx(p.y, q.y));
    }

    #[test]
    fn bbox_contains_control_points() {
        // Evidence: WO-002-AC04 — bbox contains its (control) points.
        let pts = [
            Point2::new(-1.0, -2.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, -1.0).unwrap(),
        ];
        let s = cubic_bezier(pts);
        let bb = s.bounding_box();
        for p in &pts {
            assert!(bb.contains(p));
        }
    }

    #[test]
    fn validate_rejects_deserialized_bad_knots() {
        let bad = Spline2 {
            degree: 3,
            control_points: vec![
                Point2::ORIGIN,
                Point2::new(1.0, 0.0).unwrap(),
                Point2::new(2.0, 0.0).unwrap(),
                Point2::new(3.0, 0.0).unwrap(),
            ],
            knots: vec![0.0, 0.0, 0.5, 0.25, 1.0, 1.0, 1.0, 1.0], // non-monotonic
            weights: None,
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — Spline2 round-trip serialization.
        let pts = vec![
            Point2::new(0.0, 0.0).unwrap(),
            Point2::new(1.0, 2.0).unwrap(),
            Point2::new(2.0, 2.0).unwrap(),
            Point2::new(3.0, 0.0).unwrap(),
        ];
        let s = Spline2::new(
            3,
            pts,
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            Some(vec![1.0, 2.0, 2.0, 1.0]),
        )
        .unwrap();
        let d = roundtrip(&s).unwrap();
        assert_eq!(s, d);
    }
}
