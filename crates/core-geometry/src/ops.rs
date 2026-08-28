//! Cross-cutting geometry traits and the [`Intersection2`] result enum.
//!
//! These traits describe the *capability surface* shared by located
//! primitives. Per the frozen v1.1 contract
//! (`spec/domain-model.md` §"Core value types and invariants"):
//!
//! > Geometry predicates that require robust classification MUST use an
//! > explicit tolerance policy; tolerance is never implicit or
//! > caller-chosen on a per-operation basis.
//!
//! To enforce that contract, NO trait method in this module takes a
//! per-call `tol: Tolerance` parameter. Every predicate-style operation
//! uses [`crate::tolerance::Tolerance::CANONICAL`] internally; the only
//! public tolerance value is the const [`Tolerance::CANONICAL`].
//!
//! ## Exact vs approximate operations
//!
//! The traits [`Project2`], [`DistanceTo2`], [`Contains2`], and
//! [`Intersect2`] describe EXACT operations: their results are
//! mathematically determined up to floating-point rounding, with no
//! sampling fallback and no iterative refinement.
//!
//! Primitives whose closest-point / distance cannot be computed exactly
//! in closed form (e.g. [`crate::ellipse::Ellipse2`],
//! [`crate::spline::Spline2`]) implement [`ApproximateProject2`] and
//! [`ApproximateDistanceTo2`] instead. These traits are deliberately
//! separate so callers cannot silently use an approximate result where
//! an exact result is required.

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::line::LineSegment2;
use crate::point::Point2;
use crate::transform::Transform2D;

/// Result of a 2D intersection between two primitives.
#[derive(Debug, Clone, PartialEq)]
pub enum Intersection2 {
    /// No common points.
    Empty,
    /// Exactly one point in common.
    Point(Point2),
    /// A finite set of points in common (e.g. two crossings).
    Points(Vec<Point2>),
    /// A segment in common (overlapping collinear segments).
    Segment(LineSegment2),
    /// Infinitely many common points (the primitives coincide).
    Coincident,
}

/// `validate()` is the canonical-model boundary check: it rejects
/// deserialized or assembled values that contain NaN/Inf or are
/// geometrically degenerate.
///
/// Per the frozen v1.1 domain model: "f64 values must be finite. NaN and
/// infinities are rejected at every canonical-model boundary." This
/// trait is invoked automatically by every geometry type's
/// `Deserialize` impl (see each type's `try_from` raw shadow), and may
/// also be called explicitly on values obtained by other means (e.g.
/// struct-literal construction in tests).
pub trait Validate {
    /// Returns `Ok(())` if `self` is finite and structurally valid, else an
    /// appropriate [`GeometryError`].
    fn validate(&self) -> Result<(), GeometryError>;
}

/// Axis-aligned bounding box of a located primitive.
pub trait Bounded2 {
    /// Returns the conservative axis-aligned bounding box covering `self`.
    #[must_use]
    fn bounding_box(&self) -> BoundingBox2;
}

/// Located-primitive affine transformation under a [`Transform2D`].
///
/// Returns `Err(GeometryError::Degenerate(_))` when the image of `self`
/// under `transform` cannot be represented in `self`'s frozen primitive
/// form. For example, a [`crate::circle::Circle2`] under non-uniform
/// scaling becomes an ellipse and is not representable as a `Circle2`; a
/// [`crate::transform::Transform2D`] composed with another may produce
/// shear unrepresentable in the frozen `R·diag` form. Such rejections
/// use the canonical tolerance policy
/// ([`crate::tolerance::Tolerance::CANONICAL`]); the tolerance is NOT
/// caller-chosen per-operation (frozen v1.1 contract).
pub trait Transformable2 {
    /// Returns a copy of `self` with `transform` applied, or an error when the
    /// image is not representable in `self`'s frozen primitive form.
    #[must_use]
    fn transform(&self, transform: &Transform2D) -> Result<Self, GeometryError>
    where
        Self: Sized;
}

/// EXACT distance from `self` to a `Rhs` located primitive.
///
/// Primitives whose distance cannot be computed exactly in closed form
/// (e.g. [`crate::ellipse::Ellipse2`], [`crate::spline::Spline2`]) do
/// NOT implement this trait; they implement [`ApproximateDistanceTo2`]
/// instead, so callers cannot silently use an approximate result where
/// an exact result is required.
pub trait DistanceTo2<Rhs = Point2> {
    /// Returns the (unsigned) minimum Euclidean distance between `self` and
    /// `rhs`. Zero when the primitives touch/overlap.
    #[must_use]
    fn distance_to(&self, rhs: &Rhs) -> f64;
}

/// EXACT closest point of `self` to `point` (orthogonal projection,
/// clamped to the primitive where applicable).
///
/// Per the frozen v1.1 contract, predicates do not take a per-call
/// tolerance parameter; the canonical policy
/// ([`crate::tolerance::Tolerance::CANONICAL`]) is used internally
/// where needed. Primitives whose closest-point cannot be computed
/// exactly (e.g. [`crate::ellipse::Ellipse2`], [`crate::spline::Spline2`])
/// do NOT implement this trait; they implement [`ApproximateProject2`]
/// instead.
pub trait Project2 {
    /// Returns the point of `self` nearest to `point`.
    #[must_use]
    fn project_point(&self, point: &Point2) -> Point2;
}

/// Set-membership containment test for a `Rhs` located primitive inside
/// `self`. For curve primitives this tests *on-curve* membership; for area
/// primitives it tests inside-the-region. See impl docs.
///
/// The canonical tolerance policy
/// ([`crate::tolerance::Tolerance::CANONICAL`]) is used internally;
/// tolerance is NOT caller-chosen per-operation (frozen v1.1 contract).
pub trait Contains2<Rhs = Point2> {
    /// Returns `true` if `rhs` lies on/inside `self` per the impl's
    /// semantics, within the canonical tolerance policy.
    #[must_use]
    fn contains(&self, rhs: &Rhs) -> bool;
}

/// Intersection between two located primitives. The predicate is robust
/// and uses the canonical tolerance policy internally; there is no
/// per-call tolerance parameter (frozen v1.1 contract).
pub trait Intersect2<Rhs> {
    /// Returns the intersection of `self` and `rhs`.
    #[must_use]
    fn intersect(&self, rhs: &Rhs) -> Intersection2;
}

/// APPROXIMATE closest-point projection.
///
/// Implementations use deterministic sampling or iterative refinement
/// and are NOT mathematically exact. The result may differ from the
/// true closest point by a small epsilon. This trait is deliberately
/// separate from [`Project2`] (the EXACT projection trait) so callers
/// cannot silently use an approximate result where an exact result is
/// required.
///
/// Implementations document their approximation strategy and a
/// deterministic sample/iteration count so the result is reproducible
/// across runs (per architecture §11 "Reproducibility").
pub trait ApproximateProject2 {
    /// Returns an APPROXIMATE point of `self` nearest to `point`.
    /// Not exact; see the impl's documentation for the approximation
    /// strategy and bounded error.
    #[must_use]
    fn project_point_approx(&self, point: &Point2) -> Point2;
}

/// APPROXIMATE distance.
///
/// Companion to [`ApproximateProject2`]. Implementations are NOT exact;
/// they delegate to [`ApproximateProject2::project_point_approx`] and
/// compute the distance from the approximate closest point. Separated
/// from [`DistanceTo2`] (the EXACT distance trait) so callers cannot
/// silently use an approximate result where an exact result is
/// required.
pub trait ApproximateDistanceTo2<Rhs = Point2> {
    /// Returns an APPROXIMATE distance from `self` to `rhs`.
    #[must_use]
    fn distance_to_approx(&self, rhs: &Rhs) -> f64;
}
