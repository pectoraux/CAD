//! 2D affine transforms: exact fields per `spec/domain-model.md`.
//!
//! Per the frozen v1.1 domain model:
//! ```text
//! Transform2D { translation: Vector2, rotation_rad: f64, scale_x: f64, scale_y: f64 }
//! ```
//! These EXACT field names and types are part of the stable canonical
//! representation. Field renaming or reordering is a frozen-contract violation
//! and requires Architect approval.

use crate::error::GeometryError;
use crate::ops::{Transformable2, Validate};
use crate::point::Point2;
use crate::tolerance::Tolerance;
use crate::vector::Vector2;
use serde::{Deserialize, Serialize};

/// 2D affine transform composed of translation, rotation, and non-uniform
/// scale. Fields are EXACTLY those specified by `spec/domain-model.md`:
/// `translation: Vector2`, `rotation_rad: f64`, `scale_x: f64`,
/// `scale_y: f64`. The canonical composition order is:
///
/// 1. Scale `(scale_x, scale_y)`;
/// 2. Rotate by `rotation_rad`;
/// 3. Translate by `translation`.
///
/// i.e. `apply_point(p) = translation + R(rotation) * (scale * p)`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// Translation applied after rotation and scale.
    pub translation: Vector2,
    /// Rotation in radians (CCW for positive value).
    pub rotation_rad: f64,
    /// X scale factor.
    pub scale_x: f64,
    /// Y scale factor.
    pub scale_y: f64,
}

impl Transform2D {
    /// Identity transform: zero translation, zero rotation, unit scale.
    pub const IDENTITY: Self = Self {
        translation: Vector2::ZERO,
        rotation_rad: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
    };

    /// Construct a transform with finite components. NaN/Inf are rejected
    /// (canonical-model boundary).
    ///
    /// Note: zero `scale_x` or `scale_y` is NOT rejected at construction —
    /// such a transform maps every point to a degenerate line, but it is a
    /// valid (singular) transform. [`Self::inverse`] returns `None` for
    /// singular transforms.
    #[must_use]
    pub fn new(
        translation: Vector2,
        rotation_rad: f64,
        scale_x: f64,
        scale_y: f64,
    ) -> Result<Self, GeometryError> {
        translation.validate()?;
        if !rotation_rad.is_finite() || !scale_x.is_finite() || !scale_y.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        Ok(Self {
            translation,
            rotation_rad,
            scale_x,
            scale_y,
        })
    }

    /// Identity constructor.
    #[must_use]
    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    /// Pure translation `(tx, ty)`, identity rotation and scale.
    #[must_use]
    pub fn translation(tx: f64, ty: f64) -> Self {
        let t = Vector2::new(tx, ty).unwrap_or(Vector2::ZERO);
        Self {
            translation: t,
            rotation_rad: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Pure rotation about the origin (CCW for positive `rad`), zero
    /// translation and unit scale.
    #[must_use]
    pub fn rotation(rad: f64) -> Self {
        Self {
            translation: Vector2::ZERO,
            rotation_rad: rad,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Pure non-uniform scaling about the origin.
    #[must_use]
    pub fn scaling(sx: f64, sy: f64) -> Self {
        Self {
            translation: Vector2::ZERO,
            rotation_rad: 0.0,
            scale_x: sx,
            scale_y: sy,
        }
    }

    /// Applies the transform to a point: scale, then rotate, then translate.
    #[must_use]
    pub fn apply_point(&self, p: &Point2) -> Point2 {
        // scale
        let sx = p.x * self.scale_x;
        let sy = p.y * self.scale_y;
        // rotate
        let cos = self.rotation_rad.cos();
        let sin = self.rotation_rad.sin();
        let rx = sx * cos - sy * sin;
        let ry = sx * sin + sy * cos;
        // translate
        Point2::new_unchecked(rx + self.translation.x, ry + self.translation.y)
    }

    /// Applies the transform to a vector (rotate + scale, NO translation).
    #[must_use]
    pub fn apply_vector(&self, v: &Vector2) -> Vector2 {
        let sx = v.x * self.scale_x;
        let sy = v.y * self.scale_y;
        let cos = self.rotation_rad.cos();
        let sin = self.rotation_rad.sin();
        Vector2::new_unchecked(sx * cos - sy * sin, sx * sin + sy * cos)
    }

    /// Compose two transforms: result applies `rhs` first, then `self`.
    ///
    /// i.e. `(self ∘ rhs).apply_point(p) == self.apply_point(rhs.apply_point(p))`
    /// when the composition is representable in the frozen `R·diag` form.
    ///
    /// Returns `Err(GeometryError::Degenerate(_))` when the composition
    /// produces shear that the frozen `Transform2D` representation
    /// (`R(rotation)·diag(scale_x, scale_y) + translation`) cannot express.
    /// This occurs when `self` has non-uniform scale (`scale_x != scale_y`)
    /// and `rhs` has a rotation that is not a multiple of π/2. The
    /// frozen-contract-respecting choice is to reject rather than silently
    /// approximate (WO-002-AC02 determinism / explicit representability).
    ///
    /// The `tol` policy governs the orthogonality test on the composed
    /// matrix's columns. It is explicit per the frozen v1.1 contract.
    pub fn compose(&self, rhs: &Self, tol: Tolerance) -> Result<Self, GeometryError> {
        // Linear map of self: M_s = R(phi_s) * diag(sx_s, sy_s).
        // Columns of M_s: (sx_s*cos, sx_s*sin) and (-sy_s*sin, sy_s*cos).
        let (sa, sb) = (self.rotation_rad.sin(), self.rotation_rad.cos());
        let s_m00 = self.scale_x * sb;
        let s_m01 = -self.scale_y * sa;
        let s_m10 = self.scale_x * sa;
        let s_m11 = self.scale_y * sb;
        // Linear map of rhs: M_r = R(phi_r) * diag(sx_r, sy_r).
        let (ra, rb) = (rhs.rotation_rad.sin(), rhs.rotation_rad.cos());
        let r_m00 = rhs.scale_x * rb;
        let r_m01 = -rhs.scale_y * ra;
        let r_m10 = rhs.scale_x * ra;
        let r_m11 = rhs.scale_y * rb;
        // M = M_s * M_r
        let m00 = s_m00 * r_m00 + s_m01 * r_m10;
        let m01 = s_m00 * r_m01 + s_m01 * r_m11;
        let m10 = s_m10 * r_m00 + s_m11 * r_m10;
        let m11 = s_m10 * r_m01 + s_m11 * r_m11;
        // New translation: self.translation + M_s * rhs.translation.
        let new_translation = self.apply_vector(&rhs.translation).add(self.translation);
        // Orthogonality test on M's columns. The frozen R*diag form has
        // orthogonal columns; shear (non-orthogonal columns) is unrepresentable.
        let col1_len_sq = m00 * m00 + m10 * m10;
        let col2_len_sq = m01 * m01 + m11 * m11;
        let dot = m00 * m01 + m10 * m11;
        let col1_len = col1_len_sq.sqrt();
        let col2_len = col2_len_sq.sqrt();
        // Zero matrix: representable as scaling(0,0) with any rotation.
        let both_zero = col1_len <= tol.absolute && col2_len <= tol.absolute;
        if !both_zero {
            // Relative orthogonality: |dot| <= tol.absolute * |col1| * |col2|.
            // tol.absolute is used as a dimensionless cosine tolerance.
            let bound = tol.absolute * col1_len * col2_len;
            if dot.abs() > bound {
                return Err(GeometryError::Degenerate(
                    "compose produces non-representable shear (frozen R*diag form)",
                ));
            }
        }
        // Extract (rotation, scale_x, scale_y) from M's columns.
        let (new_rotation, new_scale_x, new_scale_y) = if col1_len > tol.absolute {
            let new_rotation = m10.atan2(m00);
            let new_scale_x = col1_len;
            let det = m00 * m11 - m01 * m10;
            let new_scale_y = det / new_scale_x;
            (new_rotation, new_scale_x, new_scale_y)
        } else {
            // col1 is zero (scale_x = 0): extract from col2.
            // M = R(phi)*diag(0, sy). col2 = M*e2 = sy*(-sin phi, cos phi).
            // So sy = |col2| (sign ambiguous when det=0; take positive) and
            // phi = atan2(-col2.x, col2.y) = atan2(-m01, m11).
            let new_scale_x = 0.0;
            let new_scale_y = col2_len;
            let new_rotation = (-m01).atan2(m11);
            (new_rotation, new_scale_x, new_scale_y)
        };
        Ok(Self {
            translation: new_translation,
            rotation_rad: new_rotation,
            scale_x: new_scale_x,
            scale_y: new_scale_y,
        })
    }

    /// Inverse of `self`, or `None` when the inverse is not representable in
    /// the frozen `Transform2D` form.
    ///
    /// `Transform2D` represents the affine map
    /// `p -> translation + R(rot)·S(sx,sy)·p` (scale, then rotate, then
    /// translate). Its inverse is `S(1/sx,1/sy)·R(-rot)·(q - translation)`,
    /// which is representable as a `Transform2D` (i.e. as `R(θ)·diag(a,b) +
    /// c`) only when the columns of the inverse linear map stay orthogonal.
    /// Concretely:
    ///
    /// - `None` if `scale_x == 0` or `scale_y == 0` (singular);
    /// - `None` if scale is non-uniform (`scale_x != scale_y` within
    ///   `tol.absolute`) AND rotation is non-zero (within the same
    ///   tolerance), because the inverse linear map `S_inv·R_-rot` has
    ///   non-orthogonal columns and is not expressible as `R·diag` without
    ///   changing the frozen contract (which would require an Architecture
    ///   Change Request);
    /// - otherwise the exact inverse.
    ///
    /// The `tol` policy is explicit per the frozen v1.1 contract.
    ///
    /// Evidence: WO-002-AC03 — singular and non-representable inverses are
    /// explicitly rejected rather than approximated.
    #[must_use]
    pub fn inverse(&self, tol: Tolerance) -> Option<Self> {
        if self.scale_x == 0.0 || self.scale_y == 0.0 {
            return None;
        }
        let uniform_scale = (self.scale_x - self.scale_y).abs() <= tol.absolute;
        let zero_rotation = self.rotation_rad.abs() <= tol.absolute;
        if !uniform_scale && !zero_rotation {
            // The inverse linear map S_inv·R_-rot has non-orthogonal columns
            // and is not expressible as R·diag within the frozen Transform2D
            // contract. Returning None is the contract-respecting choice;
            // approximating would silently violate correctness (WO-002-AC02).
            return None;
        }
        let inv_sx = 1.0 / self.scale_x;
        let inv_sy = 1.0 / self.scale_y;
        let neg_rot = -self.rotation_rad;
        let cos = neg_rot.cos();
        let sin = neg_rot.sin();
        // R(-rot) applied to the forward translation:
        let rx = self.translation.x * cos - self.translation.y * sin;
        let ry = self.translation.x * sin + self.translation.y * cos;
        // Inverse translation = -S_inv · R(-rot) · translation
        let t = Vector2::new_unchecked(-rx * inv_sx, -ry * inv_sy);
        Some(Self {
            translation: t,
            rotation_rad: neg_rot,
            scale_x: inv_sx,
            scale_y: inv_sy,
        })
    }
}

impl Validate for Transform2D {
    fn validate(&self) -> Result<(), GeometryError> {
        self.translation.validate()?;
        if !self.rotation_rad.is_finite() || !self.scale_x.is_finite() || !self.scale_y.is_finite()
        {
            return Err(GeometryError::NonFinite);
        }
        Ok(())
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transformable2 for Transform2D {
    /// `self.transform(t)` returns `self.compose(t)`: the result applies `t`
    /// first, then `self`. (Composes the right operand as the "inner" map.)
    /// Returns `Err` when the composition produces shear unrepresentable
    /// in the frozen `R·diag` form.
    fn transform(&self, transform: &Transform2D, tol: Tolerance) -> Result<Self, GeometryError> {
        self.compose(transform, tol)
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Transform2D serde round-trip.
    // Evidence: WO-002-AC03 — NaN/Inf rejection at construction.
    // Evidence: WO-002-AC04 — identity invariance, transform-then-bbox consistency.
    use super::*;
    use crate::ops::{Transformable2, Validate};
    use crate::testutil::roundtrip;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn new_rejects_non_finite() {
        assert!(Transform2D::new(Vector2::ZERO, f64::NAN, 1.0, 1.0).is_err());
        // Vector2::new already rejects non-finite, so non-finite translation
        // cannot reach Transform2D through the public constructor; exercise
        // Transform2D's own finiteness check via non-finite scale/rotation.
        assert!(Transform2D::new(Vector2::ZERO, 0.0, f64::INFINITY, 1.0).is_err());
        assert!(Transform2D::new(Vector2::ZERO, 0.0, 1.0, f64::NAN).is_err());
        assert!(Transform2D::new(Vector2::ZERO, 0.0, 1.0, 1.0).is_ok());
    }

    #[test]
    fn identity_is_identity() {
        // Evidence: WO-002-AC04 — transform identity leaves points unchanged.
        let id = Transform2D::identity();
        let p = Point2::new(std::f64::consts::PI, -2.7).unwrap();
        let q = id.apply_point(&p);
        assert!(approx(q.x, p.x));
        assert!(approx(q.y, p.y));
    }

    #[test]
    fn translation_moves_points() {
        let t = Transform2D::translation(5.0, -3.0);
        let p = Point2::new(1.0, 1.0).unwrap();
        let q = t.apply_point(&p);
        assert!(approx(q.x, 6.0));
        assert!(approx(q.y, -2.0));
    }

    #[test]
    fn rotation_by_quarter_turn() {
        let r = Transform2D::rotation(std::f64::consts::FRAC_PI_2);
        let p = Point2::new(1.0, 0.0).unwrap();
        let q = r.apply_point(&p);
        assert!(approx(q.x, 0.0));
        assert!(approx(q.y, 1.0));
    }

    #[test]
    fn scaling_components() {
        let s = Transform2D::scaling(2.0, 3.0);
        let p = Point2::new(1.0, 1.0).unwrap();
        let q = s.apply_point(&p);
        assert!(approx(q.x, 2.0));
        assert!(approx(q.y, 3.0));
    }

    #[test]
    fn inverse_round_trip_identity() {
        // Uniform scale (1.5, 1.5) + rotation 0.4: the inverse IS representable
        // as a Transform2D (uniform scale commutes with rotation).
        let t = Transform2D::new(Vector2::new(3.0, -2.0).unwrap(), 0.4, 1.5, 1.5).unwrap();
        let inv = t
            .inverse(Tolerance::DEFAULT)
            .expect("representable inverse");
        let p = Point2::new(1.7, 0.3).unwrap();
        let q = t.apply_point(&p);
        let r = inv.apply_point(&q);
        assert!(approx(r.x, p.x));
        assert!(approx(r.y, p.y));
    }

    #[test]
    fn inverse_non_representable_is_none() {
        // Evidence: WO-002-AC03 — non-uniform scale + non-zero rotation is not
        // representable as Transform2D (R·diag); the inverse is explicitly
        // rejected rather than approximated (frozen-contract preserving).
        let t = Transform2D::new(Vector2::ZERO, 0.4, 1.5, 2.0).unwrap();
        assert!(t.inverse(Tolerance::DEFAULT).is_none());
    }

    #[test]
    fn inverse_of_singular_is_none() {
        // Evidence: WO-002-AC03 — singular (zero-scale) transform inversion rejected.
        let t = Transform2D::scaling(0.0, 1.0);
        assert!(t.inverse(Tolerance::DEFAULT).is_none());
        let t = Transform2D::scaling(1.0, 0.0);
        assert!(t.inverse(Tolerance::DEFAULT).is_none());
    }

    #[test]
    fn compose_identity_preserves() {
        // Evidence: WO-002-AC04 — composing with identity is exact (up to
        // floating-point extraction error).
        let t = Transform2D::new(Vector2::new(1.0, 2.0).unwrap(), 0.5, 2.0, 3.0).unwrap();
        let id = Transform2D::identity();
        let c1 = t.compose(&id, Tolerance::DEFAULT).unwrap();
        let c2 = id.compose(&t, Tolerance::DEFAULT).unwrap();
        assert!(approx(c1.translation.x, t.translation.x));
        assert!(approx(c1.translation.y, t.translation.y));
        assert!(approx(c1.rotation_rad, t.rotation_rad));
        assert!(approx(c1.scale_x, t.scale_x));
        assert!(approx(c1.scale_y, t.scale_y));
        assert!(approx(c2.translation.x, t.translation.x));
        assert!(approx(c2.translation.y, t.translation.y));
        assert!(approx(c2.rotation_rad, t.rotation_rad));
        assert!(approx(c2.scale_x, t.scale_x));
        assert!(approx(c2.scale_y, t.scale_y));
    }

    #[test]
    fn compose_exact_for_uniform_scale_and_rotation() {
        // Evidence: WO-002-AC02 — compose is exact when both factors have
        // uniform scale (no shear is introduced).
        let a = Transform2D::new(Vector2::new(1.0, 0.0).unwrap(), 0.5, 2.0, 2.0).unwrap();
        let b = Transform2D::new(Vector2::new(-1.0, 3.0).unwrap(), 0.4, 1.5, 1.5).unwrap();
        let c = a
            .compose(&b, Tolerance::DEFAULT)
            .expect("representable compose");
        for (px, py) in [(0.7, -1.2), (3.0, 4.0), (-2.5, 0.1), (10.0, -10.0)] {
            let p = Point2::new(px, py).unwrap();
            let direct = a.apply_point(&b.apply_point(&p));
            let composed = c.apply_point(&p);
            assert!(approx(direct.x, composed.x), "x mismatch at {px},{py}");
            assert!(approx(direct.y, composed.y), "y mismatch at {px},{py}");
        }
    }

    #[test]
    fn compose_exact_for_nonuniform_scale_zero_rotation() {
        // Evidence: WO-002-AC02 — non-uniform scale + zero rotation (self)
        // composes exactly when the rhs ALSO has zero rotation (so no
        // shear is introduced: the orthogonality condition
        // `(sy_a²-sx_a²)·sin(φ_b)·cos(φ_b)==0` holds trivially when
        // `φ_b==0`).
        let a = Transform2D::new(Vector2::new(1.0, 0.0).unwrap(), 0.0, 2.0, 3.0).unwrap();
        let b = Transform2D::new(Vector2::new(-1.0, 3.0).unwrap(), 0.0, 1.5, 1.5).unwrap();
        let c = a
            .compose(&b, Tolerance::DEFAULT)
            .expect("representable compose");
        for (px, py) in [(0.7, -1.2), (3.0, 4.0), (-2.5, 0.1)] {
            let p = Point2::new(px, py).unwrap();
            let direct = a.apply_point(&b.apply_point(&p));
            let composed = c.apply_point(&p);
            assert!(approx(direct.x, composed.x), "x mismatch at {px},{py}");
            assert!(approx(direct.y, composed.y), "y mismatch at {px},{py}");
        }
    }

    #[test]
    fn compose_rejects_shear() {
        // Evidence: WO-002-AC03 — non-uniform scale (self) + non-zero
        // rotation (rhs) produces shear unrepresentable in the frozen
        // R·diag form; explicitly rejected rather than approximated.
        let a = Transform2D::new(Vector2::ZERO, 0.0, 2.0, 3.0).unwrap();
        let b = Transform2D::new(Vector2::ZERO, 0.4, 1.0, 1.0).unwrap();
        let err = a.compose(&b, Tolerance::DEFAULT).unwrap_err();
        assert!(matches!(err, GeometryError::Degenerate(_)));
    }

    #[test]
    fn validate_rejects_nan_translation() {
        let t = Transform2D {
            translation: Vector2::new_unchecked(f64::NAN, 0.0),
            rotation_rad: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — Transform2D round-trip serialization.
        let t = Transform2D::new(Vector2::new(1.5, -2.5).unwrap(), 0.5, 2.0, 3.0).unwrap();
        let d = roundtrip(&t).unwrap();
        assert_eq!(t, d);
    }

    #[test]
    fn transform_trait_returns_compose() {
        // The trait `Transformable2` impl for `Transform2D` composes:
        // self.transform(t) = self.compose(t).
        let a = Transform2D::new(Vector2::new(1.0, 0.0).unwrap(), 0.1, 1.0, 1.0).unwrap();
        let b = Transform2D::new(Vector2::new(0.0, 2.0).unwrap(), 0.2, 1.5, 1.0).unwrap();
        let via_trait = a.transform(&b, Tolerance::DEFAULT).unwrap();
        let via_compose = a.compose(&b, Tolerance::DEFAULT).unwrap();
        assert_eq!(via_trait, via_compose);
    }
}
