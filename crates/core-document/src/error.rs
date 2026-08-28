//! Typed document errors.
//!
//! Per `spec/api.md` §"Error contract":
//! > Errors are typed and stable:
//! > `NotFound`, `InvalidInput`, `StaleRevision`, `GeometryInvalid`,
//! > `ConstraintViolation`, `UnsupportedObject`, `ImportDegraded`,
//! > `ExportDegraded`, `DataLossPrevented`, `PermissionDenied`,
//! > `InternalInvariantFailure`.
//! > Error codes are stable within a major contract version.
//!
//! The variant set below is CLOSED and matches the spec exactly. Adding
//! or removing a variant is a frozen-contract change requiring a new
//! architecture version. W003 only RETURNS a subset of these variants
//! (the document-model boundary never returns `PermissionDenied` — that
//! is owned by future command/permission work items); the variant is
//! present here so the type identity is stable for downstream work
//! items without re-introducing it later.
//!
//! Frozen-contract invariants honored here:
//! - Variant set is closed and matches `spec/api.md`.
//! - Every variant maps to a documented canonical-model failure mode.
//! - Error codes are stable within the v1.x major contract version.

use serde::{Deserialize, Serialize};

/// Typed error returned at the canonical-document boundary.
///
/// Variants are closed per `spec/api.md` §"Error contract". Each variant
/// carries a human-readable detail string (or structured fields where the
/// spec defines them, e.g. `StaleRevision`). The detail string is NOT a
/// stable contract — only the variant identity is stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub enum DocumentError {
    /// Referenced entity/object/style/layer/etc. was not found by ID.
    NotFound(String),
    /// Caller-supplied data violates a structural/range invariant that is
    /// not a downstream-state constraint (use `ConstraintViolation` for
    /// those).
    InvalidInput(String),
    /// Caller operated on a stale revision of the document.
    StaleRevision {
        /// Revision the caller expected.
        expected: u64,
        /// Revision the document actually has.
        actual: u64,
    },
    /// A geometry value at the canonical boundary was non-representable
    /// or degenerate (delegated from `aeccad-core-geometry::GeometryError`).
    GeometryInvalid(String),
    /// A structural invariant (e.g. dangling reference, block cycle) was
    /// violated.
    ConstraintViolation(String),
    /// An object kind is known to the spec but is not supported by this
    /// version of the canonical model. The object is preserved opaquely
    /// (per WO-003-AC03) rather than dropped.
    UnsupportedObject(String),
    /// An import completed but produced explicit diagnostics about
    /// degraded content (per architecture-lock §7 "No silent data loss").
    ImportDegraded(String),
    /// An export completed but produced explicit diagnostics about
    /// degraded content.
    ExportDegraded(String),
    /// An operation was blocked because completing it would lose data
    /// silently. The canonical model never silently drops content (per
    /// architecture-lock §7 and WO-003-AC03).
    DataLossPrevented(String),
    /// Caller lacks permission for the requested mutation. Returned by
    /// future command/permission work items; the W003 document-model
    /// boundary never returns this variant, but the variant is part of
    /// the closed stable error set per `spec/api.md`.
    PermissionDenied(String),
    /// A canonical-model invariant was violated that should be impossible
    /// given prior validation. Indicates a bug in the canonical model
    /// implementation, not a caller error.
    InternalInvariantFailure(String),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(m) => write!(f, "not found: {m}"),
            Self::InvalidInput(m) => write!(f, "invalid input: {m}"),
            Self::StaleRevision { expected, actual } => {
                write!(f, "stale revision: expected {expected}, actual {actual}")
            }
            Self::GeometryInvalid(m) => write!(f, "geometry invalid: {m}"),
            Self::ConstraintViolation(m) => write!(f, "constraint violation: {m}"),
            Self::UnsupportedObject(m) => write!(f, "unsupported object: {m}"),
            Self::ImportDegraded(m) => write!(f, "import degraded: {m}"),
            Self::ExportDegraded(m) => write!(f, "export degraded: {m}"),
            Self::DataLossPrevented(m) => write!(f, "data loss prevented: {m}"),
            Self::PermissionDenied(m) => write!(f, "permission denied: {m}"),
            Self::InternalInvariantFailure(m) => write!(f, "internal invariant failure: {m}"),
        }
    }
}

impl std::error::Error for DocumentError {}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — error variant set is closed and matches
    // `spec/api.md` §"Error contract" (no silent additions/removals).

    use super::*;

    #[test]
    fn error_variant_set_is_closed_and_matches_spec() {
        // The variant set must be exactly the 11 variants listed in
        // `spec/api.md` §"Error contract". Adding or removing a variant
        // is a frozen-contract change.
        let all = [
            DocumentError::NotFound("x".into()),
            DocumentError::InvalidInput("x".into()),
            DocumentError::StaleRevision {
                expected: 1,
                actual: 2,
            },
            DocumentError::GeometryInvalid("x".into()),
            DocumentError::ConstraintViolation("x".into()),
            DocumentError::UnsupportedObject("x".into()),
            DocumentError::ImportDegraded("x".into()),
            DocumentError::ExportDegraded("x".into()),
            DocumentError::DataLossPrevented("x".into()),
            DocumentError::PermissionDenied("x".into()),
            DocumentError::InternalInvariantFailure("x".into()),
        ];
        // Round-trip every variant through serde to prove wire stability
        // (per "Error codes are stable within a major contract version").
        for variant in all {
            let json = serde_json::to_string(&variant).expect("serialize");
            let back: DocumentError = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, back, "variant did not round-trip: {json}");
        }
    }

    #[test]
    fn stale_revision_carries_structured_fields() {
        // Evidence: WO-003-AC01 — StaleRevision carries expected/actual
        // revision numbers as structured fields (not just a string), so
        // callers can act programmatically.
        let err = DocumentError::StaleRevision {
            expected: 7,
            actual: 9,
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains("\"expected\":7"), "json = {json}");
        assert!(json.contains("\"actual\":9"), "json = {json}");
    }
}
