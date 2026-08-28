//! `Entity` — the canonical entity shell.
//!
//! Per `spec/domain-model.md` §"Generic CAD entities" / "Entity":
//! ```text
//! Entity {
//!   id: EntityId,
//!   layer_id: LayerId,
//!   owner_block_id: BlockDefinitionId | null,
//!   transform: Transform2D,
//!   visibility: VisibilityState,
//!   common_style: StyleRef,
//!   source_provenance: Provenance,
//! }
//! ```
//!
//! The W003 work-order scope is the **entity shell** — the
//! identity/ownership/metadata fields that every entity carries. Concrete
//! geometry specializations (Line, Polyline, Arc, Circle, Ellipse,
//! Spline, Point, Ray, XLine, Text, MText, Hatch, Dimension, Leader,
//! MLeader, Insert, Attribute, Solid2D) are listed in the frozen spec
//! but are owned by FUTURE Work Items:
//! - annotation specializations (Text, MText, Hatch, Dimension, Leader,
//!   MLeader) — W008;
//! - entity-placement commands (Insert, Attribute) — W007;
//! - primitive geometry on entities — W006 command engine.
//!
//! W003 implements ONLY the shell. This is not "inventing unspecified
//! semantics" — it is the literal `Entity` contract from the frozen
//! `spec/domain-model.md`. Per the W003 stop conditions, adding
//! specialization payloads now would be out-of-scope work.
//!
//! Frozen-contract invariants honored here:
//! - The shell's field set is exactly the spec's field set; no extra
//!   fields, no missing fields.
//! - `Transform2D` is reused from `aeccad-core-geometry` (the only
//!   allowed aeccad-* dep). Its field layout is frozen by the geometry
//!   crate.
//! - `owner_block_id: Option<BlockDefinitionId>` — `None` means the
//!   entity lives in model space; `Some(..)` means it lives in the
//!   block definition's local coordinate space.
//! - Unknown fields are rejected at the deserialization boundary
//!   (`#[serde(deny_unknown_fields)]`).

use aeccad_core_geometry::Transform2D;
use serde::{Deserialize, Serialize};

use crate::identity::{BlockDefinitionId, EntityId, LayerId};
use crate::value_types::{Provenance, StyleRef, VisibilityState};

/// Canonical entity shell.
///
/// Carries the identity, ownership, transform, visibility, style
/// reference and provenance fields per `spec/domain-model.md`
/// §"Entity". Concrete geometry specializations are owned by future
/// Work Items; this shell is the foundation they build on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    /// Stable, opaque, never-reused entity identity.
    pub id: EntityId,
    /// Owning layer. Must resolve to a `Layer` in the parent `Drawing`.
    pub layer_id: LayerId,
    /// Owning block definition, or `None` for an entity in model space.
    /// Per `spec/domain-model.md` §"Representation and reference
    /// invariants" #1: "Every entity has exactly one owning drawing and
    /// at most one owning block definition."
    pub owner_block_id: Option<BlockDefinitionId>,
    /// Entity-local affine transform. Reused from `aeccad-core-geometry`.
    pub transform: Transform2D,
    /// Visibility state (closed enum per [`VisibilityState`]).
    pub visibility: VisibilityState,
    /// Reference to a style (by stable ID, not by embedded value).
    pub common_style: StyleRef,
    /// Origin record (closed `ProvenanceKind`).
    pub source_provenance: Provenance,
}

impl Entity {
    /// Construct an entity shell. Performs no validation beyond
    /// accepting the fields; cross-entity invariants (layer exists,
    /// block exists, no cycles) are validated by the parent
    /// `Drawing`'s `validate()` method (see [`crate::drawing::Drawing`]).
    #[must_use]
    pub fn new(
        id: EntityId,
        layer_id: LayerId,
        owner_block_id: Option<BlockDefinitionId>,
        transform: Transform2D,
        visibility: VisibilityState,
        common_style: StyleRef,
        source_provenance: Provenance,
    ) -> Self {
        Self {
            id,
            layer_id,
            owner_block_id,
            transform,
            visibility,
            common_style,
            source_provenance,
        }
    }

    /// `true` if the entity lives in model space (`owner_block_id` is
    /// `None`).
    #[must_use]
    pub fn is_in_model_space(&self) -> bool {
        self.owner_block_id.is_none()
    }

    /// `true` if the entity lives in the given block definition.
    #[must_use]
    pub fn is_in_block(&self, block_id: &BlockDefinitionId) -> bool {
        self.owner_block_id.as_ref() == Some(block_id)
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — Entity shell invariants.
    // Evidence: WO-003-AC02 — Entity shell serde round-trip preserves
    // identity (id, layer_id, owner_block_id) and the related fields.

    use super::*;
    use crate::identity::TestIdGenerator;
    use aeccad_core_geometry::Vector2;

    fn fixture_entity() -> Entity {
        let mut g = TestIdGenerator::new(0);
        let id = crate::identity::next_entity_id(&mut g);
        let layer_id = crate::identity::next_layer_id(&mut g);
        let block_id = crate::identity::next_block_definition_id(&mut g);
        let style_id = crate::identity::next_style_id(&mut g);
        Entity::new(
            id,
            layer_id,
            Some(block_id),
            Transform2D::new(Vector2::new(1.0, 2.0).unwrap(), 0.5, 2.0, 3.0).unwrap(),
            VisibilityState::Visible,
            StyleRef::new(style_id),
            Provenance::created(),
        )
    }

    #[test]
    fn entity_shell_field_set_matches_spec() {
        // Evidence: WO-003-AC01 — the serialized field set is exactly
        // {id, layer_id, owner_block_id, transform, visibility,
        //  common_style, source_provenance}. No extra, no missing.
        let e = fixture_entity();
        let j = serde_json::to_value(&e).expect("serialize");
        let obj = j.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "common_style",
                "id",
                "layer_id",
                "owner_block_id",
                "source_provenance",
                "transform",
                "visibility",
            ]
        );
    }

    #[test]
    fn entity_serde_roundtrip_preserves_identity() {
        // Evidence: WO-003-AC02 — id/layer_id/owner_block_id all
        // survive a serde round trip unchanged.
        let e = fixture_entity();
        let j = serde_json::to_string(&e).expect("serialize");
        let back: Entity = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(e.id, back.id, "id roundtrip");
        assert_eq!(e.layer_id, back.layer_id, "layer_id roundtrip");
        assert_eq!(
            e.owner_block_id, back.owner_block_id,
            "owner_block_id roundtrip"
        );
        assert_eq!(e.common_style, back.common_style, "common_style roundtrip");
    }

    #[test]
    fn entity_in_model_space_when_owner_is_none() {
        let mut g = TestIdGenerator::new(0);
        let id = crate::identity::next_entity_id(&mut g);
        let layer_id = crate::identity::next_layer_id(&mut g);
        let style_id = crate::identity::next_style_id(&mut g);
        let e = Entity::new(
            id,
            layer_id,
            None,
            Transform2D::IDENTITY,
            VisibilityState::Visible,
            StyleRef::new(style_id),
            Provenance::created(),
        );
        assert!(e.is_in_model_space());
        let some_block = crate::identity::next_block_definition_id(&mut g);
        assert!(!e.is_in_block(&some_block));
    }

    #[test]
    fn entity_rejects_unknown_fields() {
        // Evidence: WO-003-AC01 — "Unknown/extra fields are forbidden in
        // canonical persisted DTOs."
        let e = fixture_entity();
        let mut j = serde_json::to_value(&e).expect("serialize");
        let obj = j.as_object_mut().expect("object");
        obj.insert("surprise_field".to_string(), serde_json::Value::Null);
        let r: Result<Entity, _> = serde_json::from_value(j);
        assert!(r.is_err(), "unknown field must be rejected");
    }
}
