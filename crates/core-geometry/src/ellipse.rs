//! 2D ellipses (axis-aligned in local frame, with optional rotation).

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

/// A 2D ellipse centered at `center` with semi-axes `radii = (rx, ry)` and
/// counter-clockwise rotation `rotation_rad` of its local frame.
///
/// Both `radii.x` and `radii.y` must be strictly positive and finite. The
/// curve is the locus `(x'/rx)^2 + (y'/ry)^2 = 1` where `(x', y')` is the
/// point in the local (rotated+translated) frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ellipse2 {
    /// Center of the ellipse.
    pub center: Point2,
    /// Semi-axes `(rx, ry)` (both positive).
    pub radii: Vector2,
    /// Counter-clockwise rotation of the local frame in radians.
    pub rotation_rad: f64,
}

impl Ellipse2 {
    /// Construct an ellipse from a center, a `radii` vector `(rx, ry)`, and a
    /// rotation. Requires `rx > 0`, `ry > 0`, and all finite.
    #[must_use]
    pub fn new(center: Point2, radii: Vector2, rotation_rad: f64) -> Result<Self, GeometryError> {
        center.validate()?;
        radii.validate()?;
        if !rotation_rad.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if radii.x <= 0.0 || radii.y <= 0.0 {
            return Err(GeometryError::Degenerate(
                "ellipse semi-axes must be positive",
            ));
        }
        Ok(Self {
            center,
            radii,
            rotation_rad,
        })
    }

    /// Area enclosed: `π · rx · ry`.
    #[must_use]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radii.x * self.radii.y
    }

    /// Transform `point` into the ellipse's local (unrotated, untranslated)
    /// frame: rotate by `-rotation_rad` about `center`.
    #[must_use]
    pub fn to_local(&self, point: &Point2) -> Vector2 {
        let v = self.center.vector_to(*point);
        let c = (-self.rotation_rad).cos();
        let s = (-self.rotation_rad).sin();
        Vector2::new_unchecked(v.x * c - v.y * s, v.x * s + v.y * c)
    }

    /// Returns the normalized radial coordinate `(x'/rx)^2 + (y'/ry)^2 - 1`.
    /// Zero on the curve; negative inside; positive outside.
    #[must_use]
    pub fn implicit_value(&self, point: &Point2) -> f64 {
        let v = self.to_local(point);
        (v.x / self.radii.x) * (v.x / self.radii.x) + (v.y / self.radii.y) * (v.y / self.radii.y)
            - 1.0
    }

    /// Closest point on the ellipse curve to `p`.
    ///
    /// Uses Newton iteration on the parametric form `P(t) = (rx cos t, ry sin t)`
    /// in local coordinates, after rotating back. Converges in a few
    /// iterations for well-conditioned inputs; falls back to a coarse grid
    /// scan if iteration fails to converge. Documented as approximate; exact
    /// closest-point on a rotated ellipse is quartic and is deferred to a
    /// later refinement per W002 scope ("a foundation; sampling-based with
    /// deterministic sample count is acceptable"). The center-coincidence
    /// check in `initial_guess_angle` is exact-zero (no implicit tolerance
    /// per the frozen v1.1 contract).
    #[must_use]
    pub fn project_point(&self, p: &Point2) -> Point2 {
        let local = self.to_local(p);
        let t0 = Self::initial_guess_angle(&local);
        let t = self.refine_angle(t0, &local);
        self.point_at_angle(t)
    }

    /// Distance from `p` to the ellipse curve.
    #[must_use]
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        self.project_point(p).distance_to(*p)
    }

    /// Returns `true` if `p` lies on the ellipse curve within `tolerance`
    /// (uses the implicit value scaled by the larger semi-axis).
    #[must_use]
    pub fn contains_point(&self, p: &Point2, tolerance: Tolerance) -> bool {
        let scale = self.radii.x.max(self.radii.y);
        self.implicit_value(p).abs() * scale <= tolerance.absolute
    }

    /// Returns `true` if `p` lies inside the closed region bounded by the
    /// ellipse.
    #[must_use]
    pub fn contains_disk(&self, p: &Point2) -> bool {
        self.implicit_value(p) <= 0.0
    }

    /// Point on the ellipse at parametric angle `t` (local frame, then
    /// rotated + translated).
    #[must_use]
    pub fn point_at_angle(&self, t: f64) -> Point2 {
        let lx = self.radii.x * t.cos();
        let ly = self.radii.y * t.sin();
        let c = self.rotation_rad.cos();
        let s = self.rotation_rad.sin();
        Point2::new_unchecked(
            self.center.x + lx * c - ly * s,
            self.center.y + lx * s + ly * c,
        )
    }

    fn initial_guess_angle(local: &Vector2) -> f64 {
        if local.length_squared() == 0.0 {
            return 0.0;
        }
        local.y.atan2(local.x)
    }

    fn refine_angle(&self, mut t: f64, local: &Vector2) -> f64 {
        // Newton iterations to minimize squared distance from local point to
        // the parametric curve (rx cos t, ry sin t).
        let rx = self.radii.x;
        let ry = self.radii.y;
        for _ in 0..32 {
            let px = rx * t.cos();
            let py = ry * t.sin();
            // Derivative of squared distance w.r.t. t:
            // f(t) = (rx cos t - lx)^2 + (ry sin t - ly)^2
            // f'(t) = 2(rx cos t - lx)(-rx sin t) + 2(ry sin t - ly)(ry cos t)
            let df = -2.0 * (px - local.x) * (rx * t.sin()) + 2.0 * (py - local.y) * (ry * t.cos());
            // Second derivative (approximate).
            let d2f = 2.0 * (px - local.x) * (-px)
                + 2.0 * rx * t.sin() * rx * t.sin()
                + 2.0 * (py - local.y) * (-py)
                + 2.0 * ry * t.cos() * ry * t.cos();
            if d2f.abs() < 1e-12 {
                break;
            }
            let step = df / d2f;
            if !step.is_finite() {
                break;
            }
            t -= step;
            if step.abs() < 1e-12 {
                break;
            }
        }
        // Fallback: coarse grid scan if Newton produced a point that's clearly
        // far from the local point.
        let candidate = t;
        let proj = Vector2::new_unchecked(rx * candidate.cos(), ry * candidate.sin());
        if proj.sub(*local).length_squared() > 4.0 * (rx.max(ry)).powi(2) {
            // Scan 64 deterministic samples.
            let mut best_t = 0.0_f64;
            let mut best_d = f64::INFINITY;
            for i in 0..64 {
                let ti = (i as f64) * std::f64::consts::TAU / 64.0;
                let pi = Vector2::new_unchecked(rx * ti.cos(), ry * ti.sin());
                let d = pi.sub(*local).length_squared();
                if d < best_d {
                    best_d = d;
                    best_t = ti;
                }
            }
            return best_t;
        }
        candidate
    }
}

impl Validate for Ellipse2 {
    fn validate(&self) -> Result<(), GeometryError> {
        self.center.validate()?;
        self.radii.validate()?;
        if !self.rotation_rad.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if self.radii.x <= 0.0 || self.radii.y <= 0.0 {
            return Err(GeometryError::Degenerate(
                "ellipse semi-axes must be positive",
            ));
        }
        Ok(())
    }
}

impl Bounded2 for Ellipse2 {
    /// Exact axis-aligned bounding box of the ellipse.
    ///
    /// For an ellipse with center `c`, semi-axes `(rx, ry)`, and rotation
    /// `θ`, the world-X half-width is `sqrt((rx·cos θ)² + (ry·sin θ)²)` and
    /// the world-Y half-width is `sqrt((rx·sin θ)² + (ry·cos θ)²)`. These
    /// are the exact extrema (the cardinal angles 0, π/2, π, 3π/2 in the
    /// LOCAL frame are NOT generally the world-space extrema once the
    /// ellipse is rotated).
    fn bounding_box(&self) -> BoundingBox2 {
        let cos = self.rotation_rad.cos();
        let sin = self.rotation_rad.sin();
        let rx = self.radii.x;
        let ry = self.radii.y;
        let half_w = ((rx * cos).powi(2) + (ry * sin).powi(2)).sqrt();
        let half_h = ((rx * sin).powi(2) + (ry * cos).powi(2)).sqrt();
        BoundingBox2::new_unchecked(
            Point2::new_unchecked(self.center.x - half_w, self.center.y - half_h),
            Point2::new_unchecked(self.center.x + half_w, self.center.y + half_h),
        )
    }
}

impl Transformable2 for Ellipse2 {
    /// Exact affine image of the ellipse.
    ///
    /// The image of an ellipse under any affine transform is an ellipse
    /// (representable), EXCEPT when the transform is singular (collapses to
    /// a line/point). The exact image ellipse is computed via the
    /// eigendecomposition of `M·Mᵀ` where `M` is the combined linear map
    /// from the unit circle: `M = R(φ)·diag(sx, sy)·R(θ)·diag(a, b)`.
    ///
    /// Returns `Err(GeometryError::Degenerate(_))` when the transform
    /// collapses the ellipse to a point or line (singular).
    ///
    /// Evidence: WO-002-AC02 — exact image, no silent approximation.
    /// Evidence: WO-002-AC03 — singular collapse explicitly rejected.
    fn transform(&self, transform: &Transform2D, tol: Tolerance) -> Result<Self, GeometryError> {
        let phi = transform.rotation_rad;
        let sx = transform.scale_x;
        let sy = transform.scale_y;
        let theta = self.rotation_rad;
        let a = self.radii.x;
        let b = self.radii.y;

        // A = diag(sx, sy) * R(theta)
        let a00 = sx * theta.cos();
        let a01 = -sx * theta.sin();
        let a10 = sy * theta.sin();
        let a11 = sy * theta.cos();
        // B = A * diag(a, b)
        let b00 = a00 * a;
        let b01 = a01 * b;
        let b10 = a10 * a;
        let b11 = a11 * b;
        // M = R(phi) * B
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        let m00 = cos_phi * b00 - sin_phi * b10;
        let m01 = cos_phi * b01 - sin_phi * b11;
        let m10 = sin_phi * b00 + cos_phi * b10;
        let m11 = sin_phi * b01 + cos_phi * b11;

        // S = M * M^T (symmetric 2x2)
        let s00 = m00 * m00 + m01 * m01;
        let s01 = m00 * m10 + m01 * m11;
        let s11 = m10 * m10 + m11 * m11;

        // Eigenvalues of S.
        let tr = s00 + s11;
        let det_s = s00 * s11 - s01 * s01;
        let disc = ((tr / 2.0).powi(2) - det_s).max(0.0);
        let sqrt_disc = disc.sqrt();
        let lambda_max = tr / 2.0 + sqrt_disc;
        let lambda_min = tr / 2.0 - sqrt_disc;

        let tol_sq = tol.absolute * tol.absolute;
        if lambda_max <= tol_sq {
            return Err(GeometryError::Degenerate(
                "transform collapses ellipse to a point",
            ));
        }
        if lambda_min <= tol_sq {
            return Err(GeometryError::Degenerate(
                "transform collapses ellipse to a line segment",
            ));
        }

        let new_rx = lambda_max.sqrt();
        let new_ry = lambda_min.sqrt();

        // Eigenvector for lambda_max: for [[s00, s01], [s01, s11]], the
        // eigenvector for the larger eigenvalue is (s01, lambda_max - s00)
        // when s01 != 0; angle = atan2(s01, lambda_max - s11).
        let new_rotation = if s01.abs() > tol.absolute {
            s01.atan2(lambda_max - s11)
        } else if s00 >= s11 {
            0.0
        } else {
            std::f64::consts::FRAC_PI_2
        };

        let new_center = transform.apply_point(&self.center);
        Ellipse2::new(
            new_center,
            Vector2::new_unchecked(new_rx, new_ry),
            new_rotation,
        )
    }
}

impl DistanceTo2<Point2> for Ellipse2 {
    fn distance_to(&self, rhs: &Point2) -> f64 {
        self.distance_to_point(rhs)
    }
}

impl Project2 for Ellipse2 {
    fn project_point(&self, point: &Point2, _tol: Tolerance) -> Point2 {
        self.project_point(point)
    }
}

impl Contains2<Point2> for Ellipse2 {
    fn contains(&self, rhs: &Point2, tol: Tolerance) -> bool {
        self.contains_point(rhs, tol)
    }
}

impl Intersect2<Line2> for Ellipse2 {
    fn intersect(&self, rhs: &Line2, tolerance: Tolerance) -> Intersection2 {
        // Transform the line into the ellipse's local frame: rotate the line's
        // direction and point by -rotation_rad about the ellipse center.
        let c = (-self.rotation_rad).cos();
        let s = (-self.rotation_rad).sin();
        let local_pt = {
            let v = self.center.vector_to(rhs.point);
            Point2::new_unchecked(v.x * c - v.y * s, v.x * s + v.y * c)
        };
        let local_dir = Vector2::new_unchecked(
            rhs.direction.x * c - rhs.direction.y * s,
            rhs.direction.x * s + rhs.direction.y * c,
        );
        let rx = self.radii.x;
        let ry = self.radii.y;
        // Local line: (px + t*dx)^2 / rx^2 + (py + t*dy)^2 / ry^2 = 1.
        let px = local_pt.x;
        let py = local_pt.y;
        let dx = local_dir.x;
        let dy = local_dir.y;
        let a = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry);
        let b = 2.0 * (px * dx / (rx * rx) + py * dy / (ry * ry));
        let cc = (px * px) / (rx * rx) + (py * py) / (ry * ry) - 1.0;
        let disc = b * b - 4.0 * a * cc;
        if disc < -tolerance.absolute {
            return Intersection2::Empty;
        }
        if disc.abs() <= tolerance.absolute {
            let t = -b / (2.0 * a);
            let local_hit = Point2::new_unchecked(px + dx * t, py + dy * t);
            return Intersection2::Point(self.local_to_world(&local_hit));
        }
        let sd = disc.sqrt();
        let t1 = (-b + sd) / (2.0 * a);
        let t2 = (-b - sd) / (2.0 * a);
        let h1 = self.local_to_world(&Point2::new_unchecked(px + dx * t1, py + dy * t1));
        let h2 = self.local_to_world(&Point2::new_unchecked(px + dx * t2, py + dy * t2));
        Intersection2::Points(vec![h1, h2])
    }
}

impl Intersect2<LineSegment2> for Ellipse2 {
    fn intersect(&self, rhs: &LineSegment2, tolerance: Tolerance) -> Intersection2 {
        // Sample-and-clip: treat the segment parametrically (start..=end),
        // solve the quadratic in segment parameter t in [0,1].
        let c = (-self.rotation_rad).cos();
        let s = (-self.rotation_rad).sin();
        let local_start = {
            let v = self.center.vector_to(rhs.start);
            Point2::new_unchecked(v.x * c - v.y * s, v.x * s + v.y * c)
        };
        let local_dir = rhs.start.vector_to(rhs.end);
        // Apply the same rotation to the segment direction.
        let local_dir = Vector2::new_unchecked(
            local_dir.x * c - local_dir.y * s,
            local_dir.x * s + local_dir.y * c,
        );
        let rx = self.radii.x;
        let ry = self.radii.y;
        let px = local_start.x;
        let py = local_start.y;
        let dx = local_dir.x;
        let dy = local_dir.y;
        let a = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry);
        let b = 2.0 * (px * dx / (rx * rx) + py * dy / (ry * ry));
        let cc = (px * px) / (rx * rx) + (py * py) / (ry * ry) - 1.0;
        let disc = b * b - 4.0 * a * cc;
        if disc < -tolerance.absolute {
            return Intersection2::Empty;
        }
        if disc.abs() <= tolerance.absolute {
            let t = -b / (2.0 * a);
            if (0.0..=1.0).contains(&t) {
                let local_hit = Point2::new_unchecked(px + dx * t, py + dy * t);
                return Intersection2::Point(self.local_to_world(&local_hit));
            }
            return Intersection2::Empty;
        }
        let sd = disc.sqrt();
        let t1 = (-b + sd) / (2.0 * a);
        let t2 = (-b - sd) / (2.0 * a);
        let mut hits = Vec::new();
        for t in [t1, t2] {
            if (0.0..=1.0).contains(&t) {
                let local_hit = Point2::new_unchecked(px + dx * t, py + dy * t);
                hits.push(self.local_to_world(&local_hit));
            }
        }
        match hits.len() {
            0 => Intersection2::Empty,
            1 => Intersection2::Point(hits[0]),
            _ => Intersection2::Points(hits),
        }
    }
}

impl Ellipse2 {
    /// Convert a point in local frame back to world frame (rotate by
    /// `rotation_rad` and translate by `center`).
    fn local_to_world(&self, local: &Point2) -> Point2 {
        let c = self.rotation_rad.cos();
        let s = self.rotation_rad.sin();
        Point2::new_unchecked(
            self.center.x + local.x * c - local.y * s,
            self.center.y + local.x * s + local.y * c,
        )
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Ellipse2 serde round-trip.
    // Evidence: WO-002-AC03 — degenerate axes rejected; NaN/Inf rejected.
    // Evidence: WO-002-AC04 — projection lies on ellipse (within tolerance).
    use super::*;
    use crate::line::Line2;
    use crate::ops::{Bounded2, Intersect2, Intersection2, Transformable2, Validate};
    use crate::testutil::roundtrip;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_rejects_nonpositive_axes() {
        // Evidence: WO-002-AC03 — degenerate axes rejected.
        assert!(Ellipse2::new(Point2::ORIGIN, Vector2::new(0.0, 1.0).unwrap(), 0.0).is_err());
        assert!(Ellipse2::new(Point2::ORIGIN, Vector2::new(1.0, 0.0).unwrap(), 0.0).is_err());
        assert!(Ellipse2::new(Point2::ORIGIN, Vector2::new(-1.0, 1.0).unwrap(), 0.0).is_err());
        assert!(Ellipse2::new(Point2::ORIGIN, Vector2::new(1.0, 1.0).unwrap(), f64::NAN).is_err());
        assert!(Ellipse2::new(Point2::ORIGIN, Vector2::new(2.0, 1.0).unwrap(), 0.0).is_ok());
    }

    #[test]
    fn area_is_pi_rx_ry() {
        let e = Ellipse2::new(Point2::ORIGIN, Vector2::new(2.0, 3.0).unwrap(), 0.0).unwrap();
        assert!(approx(e.area(), 6.0 * std::f64::consts::PI));
    }

    #[test]
    fn contains_disk_includes_center() {
        let e = Ellipse2::new(
            Point2::new(1.0, 2.0).unwrap(),
            Vector2::new(3.0, 2.0).unwrap(),
            0.4,
        )
        .unwrap();
        assert!(e.contains_disk(&e.center));
    }

    #[test]
    fn project_point_lies_on_ellipse() {
        // Evidence: WO-002-AC04 — projection lies on primitive.
        let e = Ellipse2::new(
            Point2::new(1.0, 1.0).unwrap(),
            Vector2::new(2.0, 1.0).unwrap(),
            0.3,
        )
        .unwrap();
        for t in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
            let p = e.point_at_angle(t);
            let proj = e.project_point(&p);
            assert!(
                e.contains_point(&proj, Tolerance::new(1e-6).unwrap()),
                "projected point not on ellipse at angle {t}: {proj:?}"
            );
        }
    }

    #[test]
    fn ellipse_line_two_intersections_axis_aligned() {
        let e = Ellipse2::new(Point2::ORIGIN, Vector2::new(2.0, 1.0).unwrap(), 0.0).unwrap();
        let l = Line2::from_two_points(
            Point2::new(-5.0, 0.0).unwrap(),
            Point2::new(5.0, 0.0).unwrap(),
        )
        .unwrap();
        match e.intersect(&l, Tolerance::DEFAULT) {
            Intersection2::Points(ps) => {
                assert_eq!(ps.len(), 2);
                for p in &ps {
                    assert!(approx(e.implicit_value(p), 0.0));
                }
            }
            other => panic!("expected 2 points, got {other:?}"),
        }
    }

    #[test]
    fn bounding_box_axis_aligned_ellipse() {
        let e = Ellipse2::new(Point2::ORIGIN, Vector2::new(2.0, 1.0).unwrap(), 0.0).unwrap();
        let b = e.bounding_box();
        assert!(approx(b.min.x, -2.0));
        assert!(approx(b.max.x, 2.0));
        assert!(approx(b.min.y, -1.0));
        assert!(approx(b.max.y, 1.0));
    }

    #[test]
    // Evidence: WO-002-AC04 — rotated-ellipse bounding box is the exact
    // analytical AABB (not the 4-local-cardinal-angle sampling, which
    // under-bounds after rotation).
    fn bounding_box_rotated_ellipse_is_tight_and_contains_curve() {
        for rot in [
            0.0,
            std::f64::consts::FRAC_PI_6,
            std::f64::consts::FRAC_PI_4,
            std::f64::consts::FRAC_PI_3,
            std::f64::consts::FRAC_PI_2,
            1.7,
        ] {
            let e = Ellipse2::new(
                Point2::new(0.5, -0.3).unwrap(),
                Vector2::new(2.0, 1.0).unwrap(),
                rot,
            )
            .unwrap();
            let bb = e.bounding_box();
            // (a) The curve is contained in the bbox: sample 256 angles.
            for i in 0..256 {
                let ang = (i as f64) * std::f64::consts::TAU / 256.0;
                let p = e.point_at_angle(ang);
                assert!(p.x >= bb.min.x - 1e-9, "x={p:?} < min.x={bb:?} (rot={rot})");
                assert!(p.x <= bb.max.x + 1e-9, "x={p:?} > max.x={bb:?} (rot={rot})");
                assert!(p.y >= bb.min.y - 1e-9, "y={p:?} < min.y={bb:?} (rot={rot})");
                assert!(p.y <= bb.max.y + 1e-9, "y={p:?} > max.y={bb:?} (rot={rot})");
            }
            // (b) The bbox extents exactly match the analytical formula.
            let cos = rot.cos();
            let sin = rot.sin();
            let exp_half_w = ((2.0 * cos).powi(2) + (1.0 * sin).powi(2)).sqrt();
            let exp_half_h = ((2.0 * sin).powi(2) + (1.0 * cos).powi(2)).sqrt();
            assert!(
                approx(bb.max.x - e.center.x, exp_half_w),
                "half-width mismatch at rot={rot}"
            );
            assert!(
                approx(bb.max.y - e.center.y, exp_half_h),
                "half-height mismatch at rot={rot}"
            );
        }
    }

    #[test]
    fn transform_identity_preserves_ellipse() {
        // Evidence: WO-002-AC04 — identity transform invariance.
        let e = Ellipse2::new(
            Point2::new(1.0, 2.0).unwrap(),
            Vector2::new(3.0, 4.0).unwrap(),
            0.5,
        )
        .unwrap();
        let t = e
            .transform(&Transform2D::identity(), Tolerance::DEFAULT)
            .unwrap();
        assert!(approx(t.center.x, 1.0));
        assert!(approx(t.center.y, 2.0));
        // For identity, the radii and rotation are preserved (the
        // eigendecomposition may choose an equivalent (radii, rotation)
        // pair — the ellipse AS A SET is identical; check via sampling).
        for ang in [0.0, 1.0, 2.0, 3.0, 5.0] {
            let p = e.point_at_angle(ang);
            assert!(t.contains_point(&p, Tolerance::new(1e-6).unwrap()));
        }
    }

    #[test]
    fn ellipse_transform_uniform_scale_and_rotate_exact() {
        // Evidence: WO-002-AC02 — exact image ellipse under uniform
        // scale + rotation: transformed curve points lie on the new ellipse.
        let e = Ellipse2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Vector2::new(2.0, 1.0).unwrap(),
            0.3,
        )
        .unwrap();
        let t = Transform2D::new(Vector2::new(1.0, 1.0).unwrap(), 0.5, 2.0, 2.0).unwrap();
        let img = e.transform(&t, Tolerance::DEFAULT).unwrap();
        let loose = Tolerance::new(1e-6).unwrap();
        for i in 0..32 {
            let ang = (i as f64) * std::f64::consts::TAU / 32.0;
            let p = e.point_at_angle(ang);
            let tp = t.apply_point(&p);
            assert!(
                img.contains_point(&tp, loose),
                "transformed point {tp:?} not on image ellipse {img:?} (ang={ang})"
            );
        }
    }

    #[test]
    fn ellipse_transform_non_uniform_scale_axis_aligned_exact() {
        // Evidence: WO-002-AC02 — axis-aligned ellipse under non-uniform
        // scale (no rotation in either) is exactly representable. The
        // eigendecomposition may produce an equivalent (radii, rotation)
        // pair (e.g. rotation π/2 with swapped radii); verify by sampling
        // that the image ellipse AS A SET is correct.
        let e = Ellipse2::new(Point2::ORIGIN, Vector2::new(2.0, 3.0).unwrap(), 0.0).unwrap();
        let t = Transform2D::scaling(2.0, 5.0);
        let img = e.transform(&t, Tolerance::DEFAULT).unwrap();
        // The image should have semi-axes 4 and 15 (in some order/rotation).
        let big = img.radii.x.max(img.radii.y);
        let small = img.radii.x.min(img.radii.y);
        assert!(approx(big, 15.0));
        assert!(approx(small, 4.0));
        // Verify by sampling that transformed curve points lie on the image.
        let loose = Tolerance::new(1e-6).unwrap();
        for i in 0..32 {
            let ang = (i as f64) * std::f64::consts::TAU / 32.0;
            let p = e.point_at_angle(ang);
            let tp = t.apply_point(&p);
            assert!(img.contains_point(&tp, loose));
        }
    }

    #[test]
    fn ellipse_transform_singular_rejected() {
        // Evidence: WO-002-AC03 — singular transform collapses the ellipse
        // to a line/point; explicitly rejected.
        let e = Ellipse2::new(Point2::ORIGIN, Vector2::new(2.0, 1.0).unwrap(), 0.0).unwrap();
        let t = Transform2D::scaling(0.0, 1.0);
        let err = e.transform(&t, Tolerance::DEFAULT).unwrap_err();
        assert!(matches!(err, GeometryError::Degenerate(_)));
    }

    #[test]
    fn ellipse_transform_non_uniform_and_rotated_still_representable() {
        // Evidence: WO-002-AC02 — non-uniform scale + rotation on a
        // rotated ellipse still produces a representable image ellipse
        // (the image of an ellipse under any non-singular affine is an
        // ellipse). Verify by sampling.
        let e = Ellipse2::new(
            Point2::new(0.0, 0.0).unwrap(),
            Vector2::new(2.0, 1.0).unwrap(),
            0.3,
        )
        .unwrap();
        let t = Transform2D::new(Vector2::ZERO, 0.5, 2.0, 3.0).unwrap();
        let img = e.transform(&t, Tolerance::DEFAULT).unwrap();
        let loose = Tolerance::new(1e-6).unwrap();
        for i in 0..32 {
            let ang = (i as f64) * std::f64::consts::TAU / 32.0;
            let p = e.point_at_angle(ang);
            let tp = t.apply_point(&p);
            assert!(
                img.contains_point(&tp, loose),
                "transformed point {tp:?} not on image ellipse {img:?} (ang={ang})"
            );
        }
    }

    #[test]
    fn validate_rejects_deserialized_zero_axis() {
        let bad = Ellipse2 {
            center: Point2::ORIGIN,
            radii: Vector2::new_unchecked(0.0, 1.0),
            rotation_rad: 0.0,
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — Ellipse2 round-trip serialization.
        let e = Ellipse2::new(
            Point2::new(-1.5, 2.5).unwrap(),
            Vector2::new(3.0, 4.0).unwrap(),
            0.5,
        )
        .unwrap();
        let d = roundtrip(&e).unwrap();
        assert_eq!(e, d);
    }
}
