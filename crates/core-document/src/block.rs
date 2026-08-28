//! `BlockDefinition` and `BlockReference`.
//!
//! Per `spec/domain-model.md` §"Document objects":
//! ```text
//! BlockDefinition {
//!   id, name, base_point, entities[], attributes[], dynamic_definition | null,
//! }
//! BlockReference {
//!   id, block_definition_id, transform, attribute_values[], explodable,
//! }
//! ```
//!
//! And per `spec/domain-model.md` §"Representation and reference invariants":
//! 1. Every entity has exactly one owning drawing and at most one owning
//!    block definition.
//! 2. Every block reference points to exactly one existing block definition.
//! 3. A block definition cannot directly contain a block reference cycle.
//!
//! Frozen-contract invariants honored here:
//! - `entities` is stored as `Vec<EntityId>` (ID-only references — per
//!   the spec, block definitions reference entities by ID, not by
//!   embedded value). The actual `Entity` records live in the parent
//!   `Drawing`'s entity table.
//! - `attributes` is `Vec<AttributeId>` (attribute definitions live
//!   elsewhere — the spec doesn't define an `AttributeDefinition` type
//!   yet; W003 stores the IDs only and treats them as opaque
//!   references. Implementing the `AttributeDefinition` type would
//!   invent unspecified semantics, which the W003 work order forbids).
//! - `dynamic_definition | null` — the spec doesn't define the
//!   `DynamicDefinition` type; W003 treats it as `Option<()>` (a
//!   null placeholder). Implementing a real type would invent
//!   semantics.
//! - `base_point` is a [`Point2`](aeccad_core_geometry::Point2).
//! - Cycle detection (invariant #3) is implemented as a validator
//!   function on the parent `Drawing` (a block can only reference
//!   entities whose `owner_block_id` matches this block; cycles via
//!   nested block references are checked across the `Drawing`'s full
//!   block-reference graph; see [`crate::drawing::Drawing::validate`]).

use aeccad_core_geometry::{Point2, Transform2D};
use serde::{Deserialize, Serialize};

use crate::identity::{BlockDefinitionId, BlockReferenceId};

/// Opaque reference to an attribute definition. The spec lists `attributes[]`
/// on `BlockDefinition` without defining the `AttributeDefinition` type;
/// W003 treats the ID as opaque bytes (a placeholder for a future work
/// item that introduces the type without breaking the W003 contract).
pub type AttributeId = [u8; 16];

/// Drawing-owned block definition. A block definition is a reusable
/// collection of entities (plus attribute definitions) placed via
/// [`BlockReference`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockDefinition {
    /// Stable, opaque block-definition identity.
    pub id: BlockDefinitionId,
    /// Human-readable block name. Unique within the parent drawing.
    pub name: String,
    /// Block base point in the block's local coordinate system.
    pub base_point: Point2,
    /// Entities owned by this block. Stored as ID-only references; the
    /// entities live in the parent `Drawing`'s entity table (their
    /// `owner_block_id` MUST match this block's id).
    pub entities: Vec<crate::identity::EntityId>,
    /// Attribute definitions owned by this block. Stored as opaque
    /// IDs pending a future work item that introduces the
    /// `AttributeDefinition` type.
    pub attributes: Vec<AttributeId>,
    /// Optional dynamic block definition. W003 stores `Option<()>`
    /// (null placeholder) — a future work item will define the type
    /// without breaking the W003 contract.
    pub dynamic_definition: Option<()>,
}

/// Drawing-owned block reference (an "insert"). References a
/// [`BlockDefinition`] by stable ID and applies a transform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockReference {
    /// Stable, opaque block-reference identity.
    pub id: BlockReferenceId,
    /// Referenced block definition. MUST exist in the parent `Drawing`.
    pub block_definition_id: BlockDefinitionId,
    /// Placement transform. Reused from `aeccad-core-geometry`.
    pub transform: Transform2D,
    /// Attribute values bound to the referenced block's attribute
    /// definitions. W003 stores opaque byte strings (a placeholder
    /// for a future work item that introduces the typed
    /// `AttributeValue` type without breaking the W003 contract).
    pub attribute_values: Vec<Vec<u8>>,
    /// Whether the reference can be exploded into its constituent
    /// entities by a future command (W007).
    pub explodable: bool,
}

impl BlockDefinition {
    /// Construct a block definition with the given fields. Performs no
    /// cross-entity validation (that is the parent `Drawing`'s job).
    #[must_use]
    pub fn new(
        id: BlockDefinitionId,
        name: String,
        base_point: Point2,
        entities: Vec<crate::identity::EntityId>,
        attributes: Vec<AttributeId>,
        dynamic_definition: Option<()>,
    ) -> Self {
        Self {
            id,
            name,
            base_point,
            entities,
            attributes,
            dynamic_definition,
        }
    }
}

impl BlockReference {
    /// Construct a block reference. The `block_definition_id` is
    /// validated against the parent `Drawing`'s block table at
    /// `Drawing::validate` time.
    #[must_use]
    pub fn new(
        id: BlockReferenceId,
        block_definition_id: BlockDefinitionId,
        transform: Transform2D,
        attribute_values: Vec<Vec<u8>>,
        explodable: bool,
    ) -> Self {
        Self {
            id,
            block_definition_id,
            transform,
            attribute_values,
            explodable,
        }
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — block reference points to exactly one
    // existing block definition (validated by Drawing::validate; here
    // we test the field is present and ID-typed).
    // Evidence: WO-003-AC02 — block def/ref serde round-trips.
    // Evidence: WO-003-AC03 — opaque `attributes`/`attribute_values`/
    // `dynamic_definition` placeholders are preserved verbatim through
    // a round trip (no content lost).

    use super::*;
    use crate::identity::TestIdGenerator;

    fn fixture_block() -> (BlockDefinition, BlockReference) {
        let mut g = TestIdGenerator::new(0);
        let block_id = crate::identity::next_block_definition_id(&mut g);
        let entity_ids: Vec<_> = (0..3)
            .map(|_| crate::identity::next_entity_id(&mut g))
            .collect();
        let attribute_ids: Vec<AttributeId> = (0..2).map(|i| [i as u8; 16]).collect();
        let block = BlockDefinition::new(
            block_id,
            "MYBLOCK".to_string(),
            Point2::new(1.0, 2.0).unwrap(),
            entity_ids,
            attribute_ids,
            None,
        );
        let reference = BlockReference::new(
            crate::identity::next_block_reference_id(&mut g),
            block_id,
            Transform2D::IDENTITY,
            vec![vec![0xA1, 0xB2], vec![0xC3, 0xD4]],
            true,
        );
        (block, reference)
    }

    #[test]
    fn block_definition_field_set_matches_spec() {
        let (b, _) = fixture_block();
        let v = serde_json::to_value(&b).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "attributes",
                "base_point",
                "dynamic_definition",
                "entities",
                "id",
                "name"
            ]
        );
    }

    #[test]
    fn block_reference_field_set_matches_spec() {
        let (_, r) = fixture_block();
        let v = serde_json::to_value(&r).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "attribute_values",
                "block_definition_id",
                "explodable",
                "id",
                "transform"
            ]
        );
    }

    #[test]
    fn block_def_and_ref_roundtrip_preserve_identity_and_refs() {
        let (b, r) = fixture_block();
        let bj = serde_json::to_string(&b).expect("serialize");
        let rj = serde_json::to_string(&r).expect("serialize");
        let b_back: BlockDefinition = serde_json::from_str(&bj).expect("deserialize");
        let r_back: BlockReference = serde_json::from_str(&rj).expect("deserialize");
        assert_eq!(b.id, b_back.id, "block id roundtrip");
        assert_eq!(b.entities, b_back.entities, "entity refs roundtrip");
        assert_eq!(b.attributes, b_back.attributes, "attribute refs roundtrip");
        assert_eq!(b.dynamic_definition, b_back.dynamic_definition);
        assert_eq!(r.id, r_back.id, "reference id roundtrip");
        assert_eq!(
            r.block_definition_id, r_back.block_definition_id,
            "block_definition_id roundtrip"
        );
        assert_eq!(
            r.attribute_values, r_back.attribute_values,
            "opaque attribute values roundtrip"
        );
    }

    #[test]
    fn block_rejects_unknown_fields() {
        let (b, r) = fixture_block();
        let mut v = serde_json::to_value(&b).expect("serialize");
        v.as_object_mut()
            .expect("object")
            .insert("surprise".to_string(), serde_json::Value::Null);
        assert!(serde_json::from_value::<BlockDefinition>(v).is_err());
        let mut v = serde_json::to_value(&r).expect("serialize");
        v.as_object_mut()
            .expect("object")
            .insert("surprise".to_string(), serde_json::Value::Null);
        assert!(serde_json::from_value::<BlockReference>(v).is_err());
    }
}
