//! `Project` and `DrawingRevision`.
//!
//! Per `spec/domain-model.md` §"Project" and §"DrawingRevision":
//! ```text
//! Project {
//!   id: ProjectId,
//!   name: string,
//!   description: string | null,
//!   status: ACTIVE | ARCHIVED,
//!   created_at,
//!   updated_at,
//! }
//! DrawingRevision {
//!   id: ArtifactVersionId,
//!   drawing_id: DrawingId,
//!   revision_number: u64,
//!   content_hash,
//!   created_at,
//!   parent_revision_id | null,
//! }
//! ```
//!
//! Frozen-contract invariants honored here:
//! - `Project.status` is a closed enum (`Active | Archived`). The spec
//!   uses `ACTIVE | ARCHIVED`; W003 stores the Rust-canonical form
//!   `Active | Archived`.
//! - `created_at` / `updated_at` are `String` (ISO 8601 or
//!   equivalent). Per `spec/architecture.md` §11 "Reproducibility",
//!   no wall-clock time may affect a committed CAD result. The
//!   canonical model stores timestamps as opaque strings provided by
//!   the importer / command engine; it does not generate them at
//!   commit time.
//! - `DrawingRevision` is immutable (a revision is a snapshot). The
//!   spec says: "A document revision is immutable. Commands advance
//!   revision exactly once on successful mutation." W003 represents
//!   this immutability by giving `DrawingRevision` only `pub` read
//!   fields and no `&mut self` methods.
//! - Unknown fields are rejected.

use serde::{Deserialize, Serialize};

use crate::identity::{ArtifactVersionId, DrawingId, ProjectId};

/// `Project.status` closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProjectStatus {
    Active,
    Archived,
}

/// A Project owns Drawings and (optionally, future) ElectricalProject
/// configuration. `ElectricalProject` is a capability/configuration
/// aggregate, NOT a second Project identity — per the spec: "A Project
/// owns Drawings and optional ElectricalProject configuration.
/// `ElectricalProject` is a capability/configuration aggregate, not a
/// second Project identity." ElectricalProject is out of W003 scope
/// (W016 will introduce it); W003 implements only the Project shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Stable, opaque project identity.
    pub id: ProjectId,
    /// Human-readable project name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Project status (closed enum).
    pub status: ProjectStatus,
    /// Creation timestamp (opaque ISO 8601 string provided by the
    /// importer / command engine). Not generated at commit time.
    pub created_at: String,
    /// Last-update timestamp.
    pub updated_at: String,
}

/// Immutable snapshot of a drawing at a specific revision. Per the
/// spec: "A document revision is immutable. Commands advance revision
/// exactly once on successful mutation." W003 represents immutability
/// by giving `DrawingRevision` only `pub` read fields and no `&mut
/// self` methods.
///
/// The `RevisionService` (per `spec/api.md` §"RevisionService") owns
/// the revision history; W003 defines the type only (the service is
/// a future work item).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrawingRevision {
    /// Stable, opaque revision identity.
    pub id: ArtifactVersionId,
    /// Owning drawing.
    pub drawing_id: DrawingId,
    /// Monotonically increasing revision number. Matches
    /// `Drawing.revision` at the time the revision was captured.
    pub revision_number: u64,
    /// Content hash of the canonical representation at this revision
    /// (deterministic — per `spec/architecture.md` §11
    /// "Reproducibility", the hash is a pure function of canonical
    /// state, not of wall-clock or locale).
    pub content_hash: String,
    /// Creation timestamp (opaque ISO 8601 string).
    pub created_at: String,
    /// Parent revision (the immediately-preceding revision), or `None`
    /// for the initial revision.
    pub parent_revision_id: Option<ArtifactVersionId>,
}

impl Project {
    /// Construct a project. Does not generate timestamps (per §11
    /// Reproducibility).
    #[must_use]
    pub fn new(
        id: ProjectId,
        name: String,
        description: Option<String>,
        status: ProjectStatus,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            status,
            created_at,
            updated_at,
        }
    }
}

impl DrawingRevision {
    /// Construct a revision. Immutability is enforced by the absence of
    /// `&mut self` methods on this type.
    #[must_use]
    pub fn new(
        id: ArtifactVersionId,
        drawing_id: DrawingId,
        revision_number: u64,
        content_hash: String,
        created_at: String,
        parent_revision_id: Option<ArtifactVersionId>,
    ) -> Self {
        Self {
            id,
            drawing_id,
            revision_number,
            content_hash,
            created_at,
            parent_revision_id,
        }
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC02 — Project + DrawingRevision round-trip.
    // Evidence: WO-003-AC01 — Project.status is a closed enum; unknown
    // values rejected.

    use super::*;
    use crate::identity::TestIdGenerator;

    fn fixture_project() -> Project {
        let mut g = TestIdGenerator::new(0);
        Project::new(
            crate::identity::next_project_id(&mut g),
            "Substation North".to_string(),
            Some("Test project".to_string()),
            ProjectStatus::Active,
            "2026-08-28T08:00:00Z".to_string(),
            "2026-08-28T08:00:00Z".to_string(),
        )
    }

    fn fixture_revision() -> DrawingRevision {
        let mut g = TestIdGenerator::new(0);
        DrawingRevision::new(
            crate::identity::next_artifact_version_id(&mut g),
            crate::identity::next_drawing_id(&mut g),
            1,
            "sha256:abc".to_string(),
            "2026-08-28T08:00:00Z".to_string(),
            None,
        )
    }

    #[test]
    fn project_field_set_matches_spec() {
        let p = fixture_project();
        let v = serde_json::to_value(&p).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "created_at",
                "description",
                "id",
                "name",
                "status",
                "updated_at"
            ]
        );
    }

    #[test]
    fn drawing_revision_field_set_matches_spec() {
        let r = fixture_revision();
        let v = serde_json::to_value(&r).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "content_hash",
                "created_at",
                "drawing_id",
                "id",
                "parent_revision_id",
                "revision_number"
            ]
        );
    }

    #[test]
    fn project_roundtrips() {
        let p = fixture_project();
        let j = serde_json::to_string(&p).expect("serialize");
        let back: Project = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(p, back);
    }

    #[test]
    fn drawing_revision_roundtrips() {
        let r = fixture_revision();
        let j = serde_json::to_string(&r).expect("serialize");
        let back: DrawingRevision = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(r, back);
    }

    #[test]
    fn project_status_rejects_unknown_variant() {
        crate::value_types::assert_unknown_variant_rejected::<ProjectStatus>("\"Draft\"");
    }

    #[test]
    fn project_rejects_unknown_fields() {
        let p = fixture_project();
        let mut v = serde_json::to_value(&p).expect("serialize");
        v.as_object_mut()
            .expect("object")
            .insert("surprise".to_string(), serde_json::Value::Null);
        assert!(serde_json::from_value::<Project>(v).is_err());
    }
}
