//! Cross-cutting geometry traits and the [`Intersection2`] result enum.
//!
//! These traits describe the *capability surface* shared by located
//! primitives. The signatures are fixed by this module (callers cannot
//! rebind them per-call), which keeps the predicate semantics deterministic
//! and tolerance-explicit (see [`crate::tolerance::Tolerance`]).

use crate::bbox::BoundingBox2;
use crate::error::GeometryError;
use crate::line::LineSegment2;
use crate::point::Point2;
use crate::tolerance::Tolerance;
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

/// `validate()` is invoked at canonical-model boundaries to reject
/// deserialized or assembled values that contain NaN/Inf or are degenerate.
///
/// Per the frozen v1.1 domain model: "f64 values must be finite. NaN and
/// infinities are rejected at every canonical-model boundary."
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
/// Returns `Err(GeometryError::Degenerate(_))` when the image of `self` under
/// `transform` cannot be represented in `self`'s frozen primitive form within
/// the supplied `tol` policy. For example, a [`crate::circle::Circle2`] under
/// non-uniform scaling becomes an ellipse and is not representable as a
/// `Circle2`; a [`crate::transform::Transform2D`] composed with another may
/// produce shear unrepresentable in the frozen `R·diag` form. The tolerance is
/// explicit per the frozen v1.1 contract: "tolerance is never implicit or
/// caller-chosen on a per-operation basis."
pub trait Transformable2 {
    /// Returns a copy of `self` with `transform` applied, or an error when the
    /// image is not representable in `self`'s frozen primitive form.
    #[must_use]
    fn transform(&self, transform: &Transform2D, tol: Tolerance) -> Result<Self, GeometryError>
    where
        Self: Sized;
}

/// Distance from `self` to a `Rhs` located primitive.
pub trait DistanceTo2<Rhs = Point2> {
    /// Returns the (unsigned) minimum Euclidean distance between `self` and
    /// `rhs`. Zero when the primitives touch/overlap.
    #[must_use]
    fn distance_to(&self, rhs: &Rhs) -> f64;
}

/// Closest point of `self` to `point` (orthogonal projection, clamped to the
/// primitive where applicable).
///
/// The `tol` policy is used by impls that internally normalize directions or
/// detect degeneracy (e.g. projecting onto a [`crate::circle::Circle2`] from
/// its center). It is explicit per the frozen v1.1 contract.
pub trait Project2 {
    /// Returns the point of `self` nearest to `point`.
    #[must_use]
    fn project_point(&self, point: &Point2, tol: Tolerance) -> Point2;
}

/// Set-membership containment test for a `Rhs` located primitive inside
/// `self`. For curve primitives this tests *on-curve* membership; for area
/// primitives it tests inside-the-region. See impl docs.
///
/// The `tol` policy is explicit per the frozen v1.1 contract: "tolerance is
/// never implicit or caller-chosen on a per-operation basis."
pub trait Contains2<Rhs = Point2> {
    /// Returns `true` if `rhs` lies on/inside `self` per the impl's semantics,
    /// within the supplied `tol`.
    #[must_use]
    fn contains(&self, rhs: &Rhs, tol: Tolerance) -> bool;
}

/// Intersection between two located primitives. The predicate is robust and
/// takes an explicit [`Tolerance`] — there is no implicit tolerance.
pub trait Intersect2<Rhs> {
    /// Returns the intersection of `self` and `rhs`.
    #[must_use]
    fn intersect(&self, rhs: &Rhs, tolerance: Tolerance) -> Intersection2;
}
