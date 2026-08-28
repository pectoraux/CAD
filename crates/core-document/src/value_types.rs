//! Closed value types and small reference structs.
//!
//! Per `spec/domain-model.md` §"Closed value types", the canonical model
//! defines a closed set of enums and small structs. This module
//! implements exactly those. Adding a variant to any enum is a
//! frozen-contract change requiring a new architecture version.
//!
//! Frozen-contract invariants honored here:
//! - Enum variant sets are CLOSED (per `spec/domain-model.md` §"Closed
//!   value types" and §"Relationship invariants" #14: "All persisted
//!   enum/state values are closed sets defined by this specification;
//!   unknown values are rejected at the canonical-model boundary").
//! - `f64` values must be finite (NaN/Inf rejected at the canonical-model
//!   boundary). The only `f64` field here is `Transform2D`'s
//!   `rotation_rad`/`scale_x`/`scale_y` — those are validated by
//!   `aeccad-core-geometry::Transform2D` (delegated; no duplicate
//!   validation here).
//! - "Unknown/extra fields are forbidden in canonical persisted DTOs."
//!   All structs here use `#[serde(deny_unknown_fields)]` so unknown
//!   fields are rejected at the deserialization boundary.
//! - "Handles originating from external formats are preserved separately
//!   and never reused as primary IDs" — `Provenance.source_handle` is
//!   the external-handle field; it is preserved opaquely and never
//!   elevated to a primary ID.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Closed enums.
// ---------------------------------------------------------------------------

/// `VisibilityState = Visible | Hidden`
///
/// Closed per `spec/domain-model.md` §"Closed value types". Unknown
/// values are rejected at the deserialization boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisibilityState {
    Visible,
    Hidden,
}

/// `SpaceRef = ModelSpace | Layout(LayoutId)`
///
/// Closed per `spec/domain-model.md` §"Closed value types". A drawing's
/// `current_space` is either the global model space or a specific layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpaceRef {
    ModelSpace,
    Layout(crate::identity::LayoutId),
}

/// `SourceKind = Xref | Image | PdfUnderlay | DgnUnderlay | Other`
///
/// Closed per `spec/domain-model.md` §"Closed value types". Classifies the
/// kind of an external reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SourceKind {
    Xref,
    Image,
    PdfUnderlay,
    DgnUnderlay,
    Other,
}

/// `PreservationStatus = PreservedOpaque | RenderedOpaque | DegradedOpaque | NotPreserved`
///
/// Closed per `spec/domain-model.md` §"Closed value types". Records how
/// well an [`OpaqueExternalObject`](crate::external::OpaqueExternalObject)
/// survived the import round trip. Per architecture-lock §5
/// "Unknown-object preservation" and §7 "No silent data loss", a
/// `NotPreserved` value MUST be accompanied by an explicit diagnostic;
/// the canonical model never silently drops content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PreservationStatus {
    PreservedOpaque,
    RenderedOpaque,
    DegradedOpaque,
    NotPreserved,
}

/// `ProvenanceKind = Imported | Created | Derived | AIPlanned`
///
/// Closed per `spec/domain-model.md` §"Closed value types". Distinguishes
/// the origin of an entity/object/revision. Per
/// `spec/domain-model.md` §"Representation and reference invariants"
/// #8: "Provenance records distinguish `Imported | Created | Derived |
/// AIPlanned` and retain source artifact/revision where available."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProvenanceKind {
    Imported,
    Created,
    Derived,
    AIPlanned,
}

/// `DrawingUnits = MetricMM | MetricCM | MetricM | ImperialIn | ImperialFt`
///
/// Closed per `spec/domain-model.md` §"Closed value types". A drawing
/// stores exactly one declared unit system (per `spec/domain-model.md`
/// §"Units": "the canonical drawing stores one declared unit system").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DrawingUnits {
    MetricMM,
    MetricCM,
    MetricM,
    ImperialIn,
    ImperialFt,
}

/// `PaperOrientation = Portrait | Landscape`
///
/// Closed per `spec/domain-model.md` §"Closed value types". Used by
/// [`Layout`](crate::layout::Layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PaperOrientation {
    Portrait,
    Landscape,
}

// ---------------------------------------------------------------------------
// Reference structs.
// ---------------------------------------------------------------------------

/// `Provenance { kind, source_artifact_hash: string | null, source_revision: string | null, source_handle: string | null }`
///
/// Per `spec/domain-model.md` §"Closed value types". Records where an
/// entity/object came from. The `source_handle` field preserves
/// external-format handles opaquely — per the spec: "Handles originating
/// from external formats are preserved separately and never reused as
/// primary IDs."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Origin kind. Closed set per [`ProvenanceKind`].
    pub kind: ProvenanceKind,
    /// Optional hash of the source artifact (e.g. file content hash).
    /// `null` for `Created` provenance.
    pub source_artifact_hash: Option<String>,
    /// Optional source revision identifier (e.g. DWG revision).
    /// `null` for `Created` provenance.
    pub source_revision: Option<String>,
    /// Optional source handle (e.g. DWG entity handle). Preserved
    /// opaquely; never elevated to a primary ID.
    pub source_handle: Option<String>,
}

impl Provenance {
    /// Construct a `Created` provenance (no source fields).
    #[must_use]
    pub fn created() -> Self {
        Self {
            kind: ProvenanceKind::Created,
            source_artifact_hash: None,
            source_revision: None,
            source_handle: None,
        }
    }

    /// Construct an `Imported` provenance with the source fields.
    #[must_use]
    pub fn imported(
        source_artifact_hash: Option<String>,
        source_revision: Option<String>,
        source_handle: Option<String>,
    ) -> Self {
        Self {
            kind: ProvenanceKind::Imported,
            source_artifact_hash,
            source_revision,
            source_handle,
        }
    }
}

/// `StyleRef { style_id: StyleId }`
///
/// Per `spec/domain-model.md` §"Closed value types". An entity's
/// `common_style` field is a `StyleRef`, not an embedded `Style` —
/// per the spec: "entity references contain IDs only". Style mutation
/// is a command (W006, future) and is revisioned; the entity only holds
/// the stable ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleRef {
    /// The referenced style's stable ID.
    pub style_id: crate::identity::StyleId,
}

impl StyleRef {
    /// Construct a `StyleRef` pointing at the given style ID.
    #[must_use]
    pub const fn new(style_id: crate::identity::StyleId) -> Self {
        Self { style_id }
    }
}

/// `RatingSet = map<string, string> with canonical sorted keys`
///
/// Per `spec/domain-model.md` §"Closed value types". Used by electrical
/// components (out of W003 scope but defined here because the type is
/// part of the closed value-type set; future electrical work items will
/// consume it). Sorted-key map = `BTreeMap<String, String>`.
pub type RatingSet = BTreeMap<String, String>;

// ---------------------------------------------------------------------------
// Unknown-enum-variant rejection test (invariant #14).
// ---------------------------------------------------------------------------

/// Asserts that an unknown enum variant string fails `Deserialize`. Used
/// by every closed-enum's test to prove the boundary rejects unknown
/// values (per `spec/domain-model.md` §"Relationship invariants" #14).
#[cfg(test)]
pub(crate) fn assert_unknown_variant_rejected<T: serde::de::DeserializeOwned>(json: &str) {
    let r: Result<T, _> = serde_json::from_str(json);
    assert!(
        r.is_err(),
        "unknown enum variant must be rejected at canonical boundary; json = {json}"
    );
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — closed enum sets reject unknown values.
    // Evidence: WO-003-AC02 — enum variants round-trip and unknown
    // variants are rejected at the deserialization boundary (per
    // spec/domain-model.md §"Relationship invariants" #14).

    use super::*;

    #[test]
    fn visibility_state_round_trips() {
        for v in [VisibilityState::Visible, VisibilityState::Hidden] {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: VisibilityState = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(v, back);
        }
        // Evidence: WO-003-AC01 — unknown variant rejected.
        assert_unknown_variant_rejected::<VisibilityState>("\"Invisible\"");
    }

    #[test]
    fn space_ref_round_trips() {
        // ModelSpace.
        let m = SpaceRef::ModelSpace;
        let j = serde_json::to_string(&m).expect("serialize");
        let back: SpaceRef = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(m, back);
        // Layout(id).
        let mut g = crate::identity::TestIdGenerator::new(0);
        let id = crate::identity::next_layout_id(&mut g);
        let l = SpaceRef::Layout(id);
        let j = serde_json::to_string(&l).expect("serialize");
        let back: SpaceRef = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(l, back);
        // Unknown variant rejected.
        assert_unknown_variant_rejected::<SpaceRef>("\"PaperSpace\"");
    }

    #[test]
    fn source_kind_round_trips() {
        let all = [
            SourceKind::Xref,
            SourceKind::Image,
            SourceKind::PdfUnderlay,
            SourceKind::DgnUnderlay,
            SourceKind::Other,
        ];
        for v in all {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: SourceKind = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(v, back);
        }
        assert_unknown_variant_rejected::<SourceKind>("\"PointCloud\"");
    }

    #[test]
    fn preservation_status_round_trips() {
        let all = [
            PreservationStatus::PreservedOpaque,
            PreservationStatus::RenderedOpaque,
            PreservationStatus::DegradedOpaque,
            PreservationStatus::NotPreserved,
        ];
        for v in all {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: PreservationStatus = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(v, back);
        }
        assert_unknown_variant_rejected::<PreservationStatus>("\"Lost\"");
    }

    #[test]
    fn provenance_kind_round_trips() {
        let all = [
            ProvenanceKind::Imported,
            ProvenanceKind::Created,
            ProvenanceKind::Derived,
            ProvenanceKind::AIPlanned,
        ];
        for v in all {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: ProvenanceKind = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(v, back);
        }
        assert_unknown_variant_rejected::<ProvenanceKind>("\"Discovered\"");
    }

    #[test]
    fn drawing_units_round_trips() {
        let all = [
            DrawingUnits::MetricMM,
            DrawingUnits::MetricCM,
            DrawingUnits::MetricM,
            DrawingUnits::ImperialIn,
            DrawingUnits::ImperialFt,
        ];
        for v in all {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: DrawingUnits = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(v, back);
        }
        assert_unknown_variant_rejected::<DrawingUnits>("\"MetricKM\"");
    }

    #[test]
    fn paper_orientation_round_trips() {
        let all = [PaperOrientation::Portrait, PaperOrientation::Landscape];
        for v in all {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: PaperOrientation = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(v, back);
        }
        assert_unknown_variant_rejected::<PaperOrientation>("\"Square\"");
    }

    #[test]
    fn provenance_round_trips_and_rejects_unknown_fields() {
        let p = Provenance::imported(
            Some("sha256:abc".to_string()),
            Some("r1".to_string()),
            Some("DWG_HANDLE_1F".to_string()),
        );
        let j = serde_json::to_string(&p).expect("serialize");
        let back: Provenance = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(p, back);
        // Unknown field rejected.
        let extra = r#"{
            "kind":"Created",
            "source_artifact_hash":null,
            "source_revision":null,
            "source_handle":null,
            "surprise_field":"boom"
        }"#;
        let r: Result<Provenance, _> = serde_json::from_str(extra);
        assert!(r.is_err(), "unknown field must be rejected");
    }

    #[test]
    fn style_ref_round_trips() {
        let mut g = crate::identity::TestIdGenerator::new(0);
        let id = crate::identity::next_style_id(&mut g);
        let r = StyleRef::new(id);
        let j = serde_json::to_string(&r).expect("serialize");
        let back: StyleRef = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(r, back);
    }

    #[test]
    fn rating_set_keys_are_sorted() {
        // Evidence: WO-003-AC02 — RatingSet is canonically sorted by key.
        // BTreeMap iterates in sorted key order.
        let mut rs: RatingSet = BTreeMap::new();
        rs.insert("zebra".to_string(), "1".to_string());
        rs.insert("alpha".to_string(), "2".to_string());
        rs.insert("mango".to_string(), "3".to_string());
        let keys: Vec<&String> = rs.keys().collect();
        assert_eq!(
            keys.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "mango", "zebra"]
        );
        // Round-trips through serde with sorted keys.
        let j = serde_json::to_string(&rs).expect("serialize");
        assert!(j.find("\"alpha\"").unwrap() < j.find("\"mango\"").unwrap());
        assert!(j.find("\"mango\"").unwrap() < j.find("\"zebra\"").unwrap());
    }
}
