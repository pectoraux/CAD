//! core-geometry — frozen architectural module boundary (W002).
//!
//! This crate provides the deterministic 2D geometry foundation for the
//! canonical CAD model. Per the frozen v1.1 architecture (`spec/architecture.md`
//! §2 "core-geometry"): "Primitives, transforms, predicates, intersections,
//! measurements, bounding boxes and tessellation inputs. No UI, persistence,
//! DWG or electrical dependencies."
//!
//! Frozen-contract invariants honored here:
//! - All coordinates use IEEE-754 `f64` at the API boundary.
//! - `f64` values must be finite; NaN and infinities are rejected at every
//!   canonical-model boundary. Each geometry type's `Deserialize` impl
//!   delegates to a private `RawXxx` shadow and then calls
//!   [`Validate`](ops::Validate), so deserialization is a canonical-model
//!   boundary that automatically rejects non-finite / degenerate values
//!   (per `spec/domain-model.md` §"Core value types and invariants").
//! - Predicates that require robust classification use the explicit,
//!   singleton canonical tolerance policy
//!   [`Tolerance::CANONICAL`](tolerance::Tolerance::CANONICAL); tolerance
//!   is never implicit or caller-chosen on a per-operation basis. There
//!   is NO per-call `tol: Tolerance` parameter on any predicate-style
//!   method signature in this crate.
//! - `Transform2D` uses the EXACT field names `translation`, `rotation_rad`,
//!   `scale_x`, `scale_y` per `spec/domain-model.md`.
//! - Geometry primitives are `Copy` stack value types (no heap allocations
//!   on the hot path, per architecture §6).
//! - Exact vs approximate operations are separated: [`Project2`](ops::Project2)
//!   and [`DistanceTo2`](ops::DistanceTo2) are EXACT; primitives whose
//!   closest-point / distance cannot be computed exactly in closed form
//!   (e.g. `Ellipse2`, `Spline2`) implement
//!   [`ApproximateProject2`](ops::ApproximateProject2) /
//!   [`ApproximateDistanceTo2`](ops::ApproximateDistanceTo2) instead.
//! - Reproducibility: no wall-clock, no `HashMap` iteration, no uncontrolled
//!   randomness; the property-test PRNG uses a fixed seed
//!   (see [`testutil::Prng`]).
//!
//! This is a LEAF crate: it depends only on `serde` (with the `derive`
//! feature). It must not depend on any `aeccad-*` crate.

#![forbid(unsafe_code)]
// Idiomatic qualified paths like `point::Point2` keep the call site readable
// and self-documenting; pedantic's module-name-repetition check is too eager
// for a geometry crate where type names mirror module names by design.
#![allow(clippy::module_name_repetitions)]
// Foundation geometry uses standard mathematical notation (x, y, a, b, i, j)
// and index arithmetic (u64 -> usize for PRNG output). These pedantic lints
// are noise here and are documented-relaxed, not silenced for correctness.
#![allow(
    clippy::many_single_char_names,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]
// `# Errors` / `# Panics` doc sections on every `Result`/panic-ing fn are
// heavy for a low-level geometry foundation; invariants are documented at the
// type/method level. `must_use` friction: `Result`/`Option` are already
// `#[must_use]` by the language, so the redundant-attr and candidate lints
// are documented-relaxed.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::double_must_use,
    clippy::similar_names
)]
// Geometry predicates test for EXACT zero (e.g. `scale_x == 0.0` for singular
// detection, `cross == 0.0` for parallelism, `t == 0.0`/`1.0` for segment
// endpoints). These intentional exact-zero checks are distinct from the
// tolerance-based comparisons used elsewhere (`< Tolerance::DEFAULT`), so
// `float_cmp` is documented-relaxed. Tests also compare floats through an
// `approx()` tolerance helper where approximate equality is the intent.
#![allow(clippy::float_cmp)]

pub mod bbox;
pub mod circle;
pub mod ellipse;
pub mod error;
pub mod line;
pub mod ops;
pub mod point;
pub mod polyline;
pub mod predicate;
pub mod spline;
#[cfg(test)]
pub mod testutil;
pub mod tolerance;
pub mod transform;
pub mod vector;

// ---------------------------------------------------------------------------
// Public surface re-exports (flat) — ergonomic for callers.
// ---------------------------------------------------------------------------

pub use bbox::BoundingBox2;
pub use circle::{Arc2, Circle2};
pub use ellipse::Ellipse2;
pub use error::GeometryError;
pub use line::{Line2, LineSegment2};
pub use ops::{
    ApproximateDistanceTo2, ApproximateProject2, Bounded2, Contains2, DistanceTo2, Intersect2,
    Intersection2, Project2, Transformable2, Validate,
};
pub use point::Point2;
pub use polyline::{Polyline2, point_in_polygon};
pub use predicate::{
    Orientation, are_coincident, are_collinear, orientation, point_left_of_line, segments_parallel,
};
pub use spline::Spline2;
pub use tolerance::Tolerance;
pub use transform::Transform2D;
pub use vector::Vector2;

/// Returns the module name for baseline architecture tests.
///
/// Kept from W001 (baseline gate asserts this constant matches the
/// `spec/architecture.md` §2 boundary name).
pub const MODULE_NAME: &str = "core-geometry";

#[cfg(test)]
mod tests {
    // Evidence: WO-001-AC02 — module boundary matches `spec/architecture.md` §2.
    // Evidence: WO-001-AC04 — deterministic baseline unit test harness.
    // Evidence: WO-002-AC05 — no document/electrical/UI/file-format deps
    // enter geometry (parse own Cargo.toml at test time).

    use std::collections::HashSet;

    #[test]
    fn module_boundary_matches_spec() {
        assert_eq!(super::MODULE_NAME, "core-geometry");
    }

    #[test]
    fn no_aeccad_dependencies_in_geometry_crate() {
        // Evidence: WO-002-AC05 — only `serde` (with derive) is allowed.
        // Parse the crate manifest at test time and assert that no
        // `aeccad-*` crate appears in [dependencies] or [dev-dependencies].
        let manifest = include_str!("../Cargo.toml");
        let mut deps_section = false;
        let mut dev_deps_section = false;
        let mut found: HashSet<String> = HashSet::new();
        for raw in manifest.lines() {
            let line = raw.trim();
            if line.starts_with('[') {
                deps_section = line == "[dependencies]";
                dev_deps_section = line == "[dev-dependencies]";
                continue;
            }
            if (deps_section || dev_deps_section) && line.starts_with("aeccad-") {
                let name = line
                    .split(['=', ' ', '\t'])
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                    .to_string();
                if !name.is_empty() {
                    found.insert(name);
                }
            }
        }
        assert!(
            found.is_empty(),
            "core-geometry must not depend on any aeccad-* crate, found: {found:?}"
        );
    }

    #[test]
    fn serde_is_the_only_external_dependency() {
        // Evidence: WO-002-AC05 — only `serde` (with derive) is allowed as an
        // external dep. Parse the crate manifest and assert that the only
        // [dependencies] entry is `serde`.
        let manifest = include_str!("../Cargo.toml");
        let mut deps_section = false;
        let mut names: Vec<String> = Vec::new();
        for raw in manifest.lines() {
            let line = raw.trim();
            if line.starts_with('[') {
                deps_section = line == "[dependencies]";
                continue;
            }
            if deps_section
                && !line.is_empty()
                && !line.starts_with('#')
                && let Some(name) = line.split(['=', ' ', '\t']).next()
                && !name.is_empty()
            {
                names.push(name.to_string());
            }
        }
        assert_eq!(
            names.iter().filter(|n| *n != "serde").count(),
            0,
            "core-geometry must depend only on serde; found extra deps: {names:?}"
        );
        assert!(
            names.iter().any(|n| n == "serde"),
            "core-geometry must depend on serde"
        );
    }

    // Evidence: WO-002-AC04 — property tests covering symmetry/invariance.
    #[test]
    fn distance_symmetry_property() {
        // d(a, b) == d(b, a) for points.
        use crate::point::Point2;
        use crate::testutil::Prng;
        let mut p = Prng::new();
        for _ in 0..256 {
            let a = Point2::new(p.signed_f64(1000.0), p.signed_f64(1000.0)).unwrap();
            let b = Point2::new(p.signed_f64(1000.0), p.signed_f64(1000.0)).unwrap();
            let d1 = a.distance_to(b);
            let d2 = b.distance_to(a);
            assert!(
                (d1 - d2).abs() < 1e-9,
                "distance not symmetric: {d1} vs {d2}"
            );
        }
    }

    #[test]
    fn bbox_contains_all_points_property() {
        // Bounding box of a point list contains all source points.
        use crate::bbox::BoundingBox2;
        use crate::point::Point2;
        use crate::testutil::Prng;
        let mut p = Prng::new();
        for _ in 0..256 {
            let n = 1 + (p.next_u64() as usize % 16);
            let mut pts = Vec::with_capacity(n);
            for _ in 0..n {
                pts.push(Point2::new(p.signed_f64(1000.0), p.signed_f64(1000.0)).unwrap());
            }
            let bb = BoundingBox2::from_points(&pts).unwrap();
            for q in &pts {
                assert!(bb.contains(q), "bbox should contain source point {q:?}");
            }
        }
    }

    #[test]
    fn transform_identity_leaves_point_unchanged_property() {
        // Evidence: WO-002-AC04 — transform identity invariance for points.
        use crate::Transform2D;
        use crate::ops::Transformable2;
        use crate::point::Point2;
        use crate::testutil::Prng;
        let id = Transform2D::identity();
        let mut prng = Prng::new();
        for _ in 0..256 {
            let p = Point2::new(prng.signed_f64(1000.0), prng.signed_f64(1000.0)).unwrap();
            let q = p.transform(&id).unwrap();
            assert!((q.x - p.x).abs() < 1e-9);
            assert!((q.y - p.y).abs() < 1e-9);
        }
    }
}
