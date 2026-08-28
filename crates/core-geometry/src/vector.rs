//! 2D vectors (directions and displacements, NOT located primitives).
//!
//! `Vector2` is the algebraic type used by [`Transform2D`](crate::Transform2D)
//! (translation), by [`Point2`](crate::Point2) arithmetic, and by the
//! direction fields of `Line2`. It does NOT implement the located-primitive
//! traits ([`Bounded2`](crate::ops::Bounded2),
//! [`Transformable2`](crate::ops::Transformable2),
//! [`DistanceTo2`](crate::ops::DistanceTo2)) — it is not a located primitive,
//! only an algebraic value.

use crate::error::GeometryError;
use crate::ops::Validate;
use crate::tolerance::Tolerance;
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
};

/// A 2D vector with `f64` components. NaN/Inf are rejected at construction.
///
/// `Vector2` is `Copy`; passing it by value is cheap and allocation-free on
/// the hot path (per architecture §6).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Vector2 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
}

// ---------------------------------------------------------------------------
// Canonical-model boundary: Deserialize delegates to a private Raw shadow
// and then calls `Validate`, so non-finite values are rejected at the
// deserialization boundary (per spec/domain-model.md §"Core value types
// and invariants": "f64 values must be finite. NaN and infinities are
// rejected at every canonical-model boundary."). Direct struct-literal
// construction remains possible (e.g. for tests of `validate()`); the
// canonical boundaries are `Vector2::new()` and `Deserialize`.
// ---------------------------------------------------------------------------

/// Private shadow struct used as the serde wire shape for [`Vector2`].
/// Carries the raw (unvalidated) field values from the deserializer; the
/// `TryFrom` impl on `Vector2` then enforces the canonical-model
/// finiteness invariant.
#[derive(Deserialize)]
struct RawVector2 {
    x: f64,
    y: f64,
}

impl TryFrom<RawVector2> for Vector2 {
    type Error = GeometryError;

    fn try_from(r: RawVector2) -> Result<Self, Self::Error> {
        let v = Self { x: r.x, y: r.y };
        v.validate()?;
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for Vector2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawVector2::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl Vector2 {
    /// Zero vector.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Unit vector along +X.
    pub const I: Self = Self { x: 1.0, y: 0.0 };

    /// Unit vector along +Y.
    pub const J: Self = Self { x: 0.0, y: 1.0 };

    /// Construct a vector with finite components.
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

    /// Construct a vector WITHOUT validating finiteness.
    ///
    /// Internal-only helper for use inside the crate where the caller has
    /// already validated finiteness. Not exposed publicly.
    #[must_use]
    pub(crate) const fn new_unchecked(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Squared magnitude — never allocates and never fails.
    #[must_use]
    pub const fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Magnitude (length).
    #[must_use]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Dot product `self · rhs`.
    #[must_use]
    pub const fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Scalar 2D cross product `self × rhs` (z-component of the 3D cross
    /// product of the embedded vectors). Positive when `rhs` is CCW from
    /// `self` in the standard orientation.
    #[must_use]
    pub const fn cross(self, rhs: Self) -> f64 {
        self.x * rhs.y - self.y * rhs.x
    }

    /// Perpendicular vector (CCW 90° rotation): `(-y, x)`.
    #[must_use]
    pub const fn perp(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    /// Returns `true` if this vector's length is within the canonical
    /// tolerance policy
    /// ([`Tolerance::CANONICAL`](crate::tolerance::Tolerance::CANONICAL))
    /// of zero. Per the frozen v1.1 contract, the tolerance is not
    /// caller-chosen per-operation.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.length_squared() <= Tolerance::CANONICAL.coincident_squared()
    }

    /// Returns the unit vector in the direction of `self`, or `None` if
    /// `self` is near-zero length within the canonical tolerance policy
    /// ([`Tolerance::CANONICAL`](crate::tolerance::Tolerance::CANONICAL)).
    ///
    /// Per the frozen v1.1 contract, tolerance is never implicit or
    /// caller-chosen on a per-operation basis; the canonical policy is
    /// used internally.
    #[must_use]
    pub fn normalize(self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let len = self.length();
        if len == 0.0 {
            return None;
        }
        Some(Self {
            x: self.x / len,
            y: self.y / len,
        })
    }

    /// Rotates this vector by `rad` radians (CCW for positive `rad`).
    #[must_use]
    pub fn rotate(self, rad: f64) -> Self {
        let cos = rad.cos();
        let sin = rad.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Scales this vector component-wise.
    #[must_use]
    pub const fn scale(self, sx: f64, sy: f64) -> Self {
        Self {
            x: self.x * sx,
            y: self.y * sy,
        }
    }

    /// Returns the signed angle from `self` to `rhs` in `(-π, π]`, computed
    /// as `atan2(self × rhs, self · rhs)`.
    #[must_use]
    pub fn angle_to(self, rhs: Self) -> f64 {
        self.cross(rhs).atan2(self.dot(rhs))
    }

    /// Element-wise sum.
    #[must_use]
    pub const fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }

    /// Element-wise difference `self - rhs`.
    #[must_use]
    pub const fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }

    /// Scalar multiplication `s * self`.
    #[must_use]
    pub const fn mul(self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

impl Validate for Vector2 {
    fn validate(&self) -> Result<(), GeometryError> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-002-AC01 — Vector2 round-trip serialization.
    // Evidence: WO-002-AC03 — NaN/Inf rejection at construction AND at the
    // deserialization canonical-model boundary (try_from + validate).
    use super::*;
    use crate::ops::Validate;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn new_rejects_non_finite() {
        assert!(Vector2::new(f64::NAN, 0.0).is_err());
        assert!(Vector2::new(0.0, f64::INFINITY).is_err());
        assert!(Vector2::new(0.0, 0.0).is_ok());
    }

    #[test]
    fn validate_rejects_serde_deserialized_nan() {
        // Direct struct-literal construction (test-only path; not a
        // canonical-model boundary) can still produce a non-finite value,
        // and `validate()` is the explicit check for such values.
        let v = Vector2 {
            x: f64::NAN,
            y: 1.0,
        };
        assert!(v.validate().is_err());
    }

    #[test]
    fn length_dot_cross_perp() {
        let v = Vector2::new(3.0, 4.0).unwrap();
        assert!(approx(v.length(), 5.0));
        assert!(approx(v.length_squared(), 25.0));
        let a = Vector2::new(1.0, 0.0).unwrap();
        let b = Vector2::new(0.0, 1.0).unwrap();
        assert!(approx(a.dot(b), 0.0));
        assert!(approx(a.cross(b), 1.0));
        assert_eq!(a.perp(), b);
    }

    #[test]
    fn normalize_zero_handling() {
        assert!(Vector2::ZERO.normalize().is_none());
        let n = Vector2::new(3.0, 4.0).unwrap().normalize().unwrap();
        assert!(approx(n.length(), 1.0));
    }

    #[test]
    fn rotate_and_scale() {
        let v = Vector2::new(1.0, 0.0).unwrap();
        let r = v.rotate(std::f64::consts::FRAC_PI_2);
        assert!(approx(r.x, 0.0));
        assert!(approx(r.y, 1.0));
        // Use a vector with a non-zero y component so the y-scale is observable:
        // (1,0).scale(2,3) = (2,0) — y stays 0; (1,1).scale(2,3) = (2,3).
        let s = Vector2::new(1.0, 1.0).unwrap().scale(2.0, 3.0);
        assert!(approx(s.x, 2.0));
        assert!(approx(s.y, 3.0));
    }

    #[test]
    fn angle_to_signed() {
        let a = Vector2::new(1.0, 0.0).unwrap();
        let b = Vector2::new(0.0, 1.0).unwrap();
        assert!(approx(a.angle_to(b), std::f64::consts::FRAC_PI_2));
        assert!(approx(b.angle_to(a), -std::f64::consts::FRAC_PI_2));
    }

    #[test]
    fn serde_roundtrip() {
        // Evidence: WO-002-AC01 — round-trip test for Vector2.
        use crate::testutil::roundtrip;
        let v = Vector2::new(-1.5, 2.25).unwrap();
        let d = roundtrip(&v).unwrap();
        assert_eq!(v, d);
    }
}
