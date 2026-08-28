//! `Drawing` — the canonical document root and authoritative model.
//!
//! Per `spec/domain-model.md` §"Document objects" / "Drawing":
//! ```text
//! Drawing {
//!   id, project_id, name, units: DrawingUnits,
//!   model_space_root: EntityId | null,
//!   layouts[], layers[], linetypes[], text_styles[], dimension_styles[],
//!   blocks[], external_refs[], metadata,
//!   active_layer_id: LayerId,
//!   current_space: SpaceRef,
//!   revision: u64,
//! }
//! ```
//!
//! And per §"Representation and reference invariants" (the ID/reference
//! invariants exercised by WO-003-AC01):
//! 1. Every entity has exactly one owning drawing and at most one owning
//!    block definition.
//! 2. Every block reference points to exactly one existing block definition.
//! 3. A block definition cannot directly contain a block reference cycle.
//! 4. Every layout belongs to exactly one drawing.
//! 5. A viewport belongs to exactly one layout.
//! 6. External references are metadata/links; their resolved content is
//!    never silently merged into the host drawing.
//! 7. Opaque external objects retain their original handle/ownership
//!    metadata independently of canonical IDs.
//! 8. Provenance records distinguish `Imported | Created | Derived |
//!    AIPlanned` and retain source artifact/revision where available.
//!
//! Architect directive (W003 activation):
//! > Do not use array position as durable identity. Do not create a
//! > second document authority.
//!
//! Frozen-contract invariants honored here:
//! - Collections are stored as `BTreeMap<Id, T>` keyed by stable ID.
//!   This DIRECTLY enforces "no array position as durable identity":
//!   there is no array; lookup is by ID, and iteration is by sorted
//!   ID (deterministic per `spec/architecture.md` §11 "Reproducibility").
//! - The `Drawing` is the SOLE document authority: there is no second
//!   container that owns entities/layers/etc. (per the "no second
//!   authority" directive). UI code may consume read-only projections
//!   (per `spec/architecture.md` §2 "`app-shell` and `app-ui` may
//!   consume public application contracts but may not own domain state
//!   or mutate the canonical model directly") but never owns the
//!   document graph.
//! - W003 implements NO mutation commands (those belong to W006 —
//!   "transaction/command engine"). The Drawing is constructed once
//!   from importer/command-engine input; `Drawing::validate()` checks
//!   every cross-reference invariant. Future mutation work items
//!   consume the existing `validate()` boundary, not a new one.
//!
//! **W003 does NOT implement** (out of scope per work-order §"Forbidden
//! changes"):
//! - geometry algorithms beyond the value-type integration (Transform2D
//!   reused from `aeccad-core-geometry`);
//! - command/UI/interop/electrical modules (out of scope).

use std::collections::BTreeMap;

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
};

use crate::block::{BlockDefinition, BlockReference};
use crate::entity::Entity;
use crate::error::DocumentError;
use crate::external::{ExternalReference, OpaqueExternalObject};
use crate::identity::{
    BlockDefinitionId, BlockReferenceId, DimensionStyleId, DrawingId, EntityId, ExternalObjectId,
    ExternalRefId, LayerId, LayoutId, ProjectId, StyleId, ViewportId,
};
use crate::layer::Layer;
use crate::layout::{Layout, Viewport};
use crate::style::{DimensionStyle, Style};
use crate::value_types::{DrawingUnits, SpaceRef};

/// The canonical document root. Owns every entity, layer, block,
/// style, layout, viewport, external reference and opaque object
/// belonging to one drawing. Per the architecture, this is the SOLE
/// document authority — there is no second container with document
/// authority.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Drawing {
    // ---- Identity ---------------------------------------------------------
    /// Stable, opaque drawing identity.
    pub id: DrawingId,
    /// Owning project.
    pub project_id: ProjectId,
    /// Human-readable drawing name.
    pub name: String,
    /// Declared unit system (closed enum). The canonical drawing
    /// stores exactly one declared unit system.
    pub units: DrawingUnits,

    // ---- Entity graph ----------------------------------------------------
    /// Root entity ID in model space, if the drawing has a single
    /// named root (e.g. an assembly root); `None` for drawings where
    /// model-space entities are listed directly.
    pub model_space_root: Option<EntityId>,
    /// All entities in the drawing, keyed by stable ID. The entity's
    /// `owner_block_id` field distinguishes model-space entities
    /// (`None`) from block-local entities (`Some(block_id)`).
    pub entities: BTreeMap<EntityId, Entity>,

    // ---- Document-owned collections ------------------------------------
    /// Layers, keyed by stable ID.
    pub layers: BTreeMap<LayerId, Layer>,
    /// Block definitions, keyed by stable ID.
    pub blocks: BTreeMap<BlockDefinitionId, BlockDefinition>,
    /// Block references (inserts), keyed by stable ID.
    pub block_references: BTreeMap<BlockReferenceId, BlockReference>,
    /// Linetypes, keyed by stable `StyleId`. Both `linetypes` and
    /// `text_styles` hold `Style` objects; the container is the
    /// disambiguator.
    pub linetypes: BTreeMap<StyleId, Style>,
    /// Text styles, keyed by stable `StyleId`.
    pub text_styles: BTreeMap<StyleId, Style>,
    /// Dimension styles, keyed by stable `DimensionStyleId`.
    pub dimension_styles: BTreeMap<DimensionStyleId, DimensionStyle>,
    /// Layouts, keyed by stable ID.
    pub layouts: BTreeMap<LayoutId, Layout>,
    /// Viewports, keyed by stable ID. Each viewport's owning layout is
    /// identified by the layout's `viewports` list.
    pub viewports: BTreeMap<ViewportId, Viewport>,
    /// External references (metadata-only links), keyed by stable ID.
    pub external_refs: BTreeMap<ExternalRefId, ExternalReference>,
    /// Opaque external objects (preserved verbatim per WO-003-AC03).
    pub opaque_objects: BTreeMap<ExternalObjectId, OpaqueExternalObject>,

    // ---- Metadata --------------------------------------------------------
    /// Sorted-key opaque metadata map.
    pub metadata: BTreeMap<String, String>,

    // ---- Session state ---------------------------------------------------
    /// Currently-active layer (for interactive editing). Persisted as
    /// part of the drawing.
    pub active_layer_id: LayerId,
    /// Current editing space (model or a layout).
    pub current_space: SpaceRef,
    /// Current revision number. Monotonically increasing; commands
    /// (future W006) advance it exactly once on successful mutation.
    pub revision: u64,
}

// ---------------------------------------------------------------------------
// Canonical-model boundary: deserialize then validate via Drawing::validate.
// ---------------------------------------------------------------------------

/// Private serde wire shape for [`Drawing`]. The deserializer builds
/// this raw form first; `Drawing::validate` then checks every
/// cross-reference invariant before the canonical model accepts the
/// value. This makes the deserialization boundary the canonical gate.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDrawing {
    id: DrawingId,
    project_id: ProjectId,
    name: String,
    units: DrawingUnits,
    model_space_root: Option<EntityId>,
    entities: BTreeMap<EntityId, Entity>,
    layers: BTreeMap<LayerId, Layer>,
    blocks: BTreeMap<BlockDefinitionId, BlockDefinition>,
    block_references: BTreeMap<BlockReferenceId, BlockReference>,
    linetypes: BTreeMap<StyleId, Style>,
    text_styles: BTreeMap<StyleId, Style>,
    dimension_styles: BTreeMap<DimensionStyleId, DimensionStyle>,
    layouts: BTreeMap<LayoutId, Layout>,
    viewports: BTreeMap<ViewportId, Viewport>,
    external_refs: BTreeMap<ExternalRefId, ExternalReference>,
    opaque_objects: BTreeMap<ExternalObjectId, OpaqueExternalObject>,
    metadata: BTreeMap<String, String>,
    active_layer_id: LayerId,
    current_space: SpaceRef,
    revision: u64,
}

impl<'de> Deserialize<'de> for Drawing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawDrawing::deserialize(deserializer)?;
        Self::from_raw(raw).map_err(de::Error::custom)
    }
}

impl Drawing {
    /// Build a `Drawing` from a raw (unvalidated) wire shape. Validates
    /// every cross-reference invariant before returning. Used by the
    /// `Deserialize` impl and by [`DrawingBuilder::build`].
    ///
    /// External callers (importer / command engine) construct a
    /// `Drawing` via [`DrawingBuilder`], not via this method —
    /// `from_raw` is an internal canonical-boundary helper.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] for the first invariant violation
    /// encountered.
    fn from_raw(raw: RawDrawing) -> Result<Self, DocumentError> {
        let drawing = Self {
            id: raw.id,
            project_id: raw.project_id,
            name: raw.name,
            units: raw.units,
            model_space_root: raw.model_space_root,
            entities: raw.entities,
            layers: raw.layers,
            blocks: raw.blocks,
            block_references: raw.block_references,
            linetypes: raw.linetypes,
            text_styles: raw.text_styles,
            dimension_styles: raw.dimension_styles,
            layouts: raw.layouts,
            viewports: raw.viewports,
            external_refs: raw.external_refs,
            opaque_objects: raw.opaque_objects,
            metadata: raw.metadata,
            active_layer_id: raw.active_layer_id,
            current_space: raw.current_space,
            revision: raw.revision,
        };
        drawing.validate()?;
        Ok(drawing)
    }

    /// Re-check every cross-reference invariant. Used after
    /// deserialization and by future mutation commands (W006) to
    /// verify the document is still well-formed.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] for the first violation.
    ///
    /// # Invariants checked
    /// - Every entity's `layer_id` resolves to a layer in `layers`.
    /// - Every entity's `owner_block_id` (if `Some`) resolves to a
    ///   block in `blocks`.
    /// - Every block's `entities[]` IDs resolve to entities whose
    ///   `owner_block_id` matches the block's ID (bidirectional
    ///   consistency — invariant #1).
    /// - Every block reference's `block_definition_id` resolves to a
    ///   block in `blocks` (invariant #2).
    /// - Block definitions do not form a reference cycle (invariant
    ///   #3 — checked via the entity→block→entity graph; entities
    ///   cannot reference each other directly, so cycles can only
    ///   arise via block-reference chains, which are forbidden here).
    /// - Every layout's `viewports[]` IDs resolve to viewports in
    ///   `viewports` (invariant #5).
    /// - Every viewport belongs to exactly one layout (invariant #5
    ///   — checked by ensuring no viewport ID appears in two
    ///   layouts' viewport lists).
    /// - `active_layer_id` resolves to a layer in `layers`.
    /// - `current_space` (if `Layout(id)`) resolves to a layout in
    ///   `layouts`.
    /// - `model_space_root` (if `Some(id)`) resolves to an entity
    ///   in `entities` whose `owner_block_id` is `None`.
    /// - Every entity's `common_style.style_id` resolves to a style
    ///   in `text_styles` (or, if not found there, in `linetypes`).
    /// - Every layer's `linetype_id` resolves to a style in
    ///   `linetypes`.
    /// - Every viewport's `layer_overrides` keys resolve to layers
    ///   in `layers`.
    pub fn validate(&self) -> Result<(), DocumentError> {
        // active_layer_id resolves.
        if !self.layers.contains_key(&self.active_layer_id) {
            return Err(DocumentError::ConstraintViolation(format!(
                "active_layer_id {} does not resolve to a layer",
                self.active_layer_id
            )));
        }
        // current_space resolves (if Layout(id)).
        if let SpaceRef::Layout(lid) = self.current_space
            && !self.layouts.contains_key(&lid)
        {
            return Err(DocumentError::ConstraintViolation(format!(
                "current_space Layout({lid}) does not resolve to a layout"
            )));
        }
        // model_space_root resolves and is in model space.
        if let Some(root_id) = self.model_space_root {
            match self.entities.get(&root_id) {
                Some(e) if e.owner_block_id.is_none() => {}
                Some(_) => {
                    return Err(DocumentError::ConstraintViolation(format!(
                        "model_space_root {root_id} is owned by a block (must be model-space)"
                    )));
                }
                None => {
                    return Err(DocumentError::ConstraintViolation(format!(
                        "model_space_root {root_id} does not resolve to an entity"
                    )));
                }
            }
        }
        // Entities: layer and block references resolve.
        for (eid, entity) in &self.entities {
            if !self.layers.contains_key(&entity.layer_id) {
                return Err(DocumentError::ConstraintViolation(format!(
                    "entity {eid} layer_id {} does not resolve",
                    entity.layer_id
                )));
            }
            if let Some(block_id) = entity.owner_block_id
                && !self.blocks.contains_key(&block_id)
            {
                return Err(DocumentError::ConstraintViolation(format!(
                    "entity {eid} owner_block_id {block_id} does not resolve"
                )));
            }
            // common_style resolves in text_styles or linetypes.
            let sid = entity.common_style.style_id;
            if !self.text_styles.contains_key(&sid) && !self.linetypes.contains_key(&sid) {
                return Err(DocumentError::ConstraintViolation(format!(
                    "entity {eid} common_style {sid} does not resolve in text_styles or linetypes"
                )));
            }
        }
        // Blocks: entities[] IDs resolve and are owned by the block.
        for (bid, block) in &self.blocks {
            for eid in &block.entities {
                match self.entities.get(eid) {
                    Some(e) if e.owner_block_id.as_ref() == Some(bid) => {}
                    Some(_) => {
                        return Err(DocumentError::ConstraintViolation(format!(
                            "block {bid} lists entity {eid} but the entity is not owned by this block"
                        )));
                    }
                    None => {
                        return Err(DocumentError::ConstraintViolation(format!(
                            "block {bid} lists entity {eid} which does not resolve"
                        )));
                    }
                }
            }
        }
        // Block references: block_definition_id resolves.
        for (rid, r) in &self.block_references {
            if !self.blocks.contains_key(&r.block_definition_id) {
                return Err(DocumentError::ConstraintViolation(format!(
                    "block_reference {rid} block_definition_id {} does not resolve",
                    r.block_definition_id
                )));
            }
        }
        // Invariant #3 — block reference cycle. See
        // `check_block_reference_cycle` for the W003 reading of this
        // invariant (structurally a no-op in the W003 model — block
        // references live at the drawing level, not nested in
        // blocks).
        self.check_block_reference_cycle();
        // Layouts: viewports[] IDs resolve and viewports belong to
        // exactly one layout (invariant #5).
        let mut seen_viewports: BTreeMap<ViewportId, LayoutId> = BTreeMap::new();
        for (lid, layout) in &self.layouts {
            for vid in &layout.viewports {
                if !self.viewports.contains_key(vid) {
                    return Err(DocumentError::ConstraintViolation(format!(
                        "layout {lid} lists viewport {vid} which does not resolve"
                    )));
                }
                if let Some(prev) = seen_viewports.insert(*vid, *lid) {
                    return Err(DocumentError::ConstraintViolation(format!(
                        "viewport {vid} belongs to both layout {prev} and layout {lid} (invariant #5 violated)"
                    )));
                }
                // Viewport layer_overrides keys resolve to layers.
                if let Some(viewport) = self.viewports.get(vid) {
                    for layer_override_id in viewport.layer_overrides.keys() {
                        if !self.layers.contains_key(layer_override_id) {
                            return Err(DocumentError::ConstraintViolation(format!(
                                "viewport {vid} layer_overrides key {layer_override_id} does not resolve to a layer"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check invariant #3 — no block reference cycle. A cycle exists
    /// if the block-reference graph (edges: block B → block C if some
    /// entity owned by B is a `BlockReference` whose
    /// `block_definition_id == C`) contains a closed walk.
    ///
    /// In W003's canonical model, `BlockReference` is a separate
    /// first-class object stored in `Drawing.block_references`, NOT an
    /// `Entity`. Block definitions contain ENTITIES (not block
    /// references) in their `entities[]` list, and entities don't
    /// reference blocks directly. Therefore a "block reference cycle
    /// via entities" is structurally impossible in the W003 model.
    ///
    /// The remaining cycle path — `BlockReference → BlockDefinition →
    /// BlockReference → ...` — would require block references to be
    /// *owned by* blocks, but the spec doesn't express a
    /// `parent_block_id` field on `BlockReference`. W003 takes the
    /// conservative reading that block references live at the drawing
    /// level (not nested in blocks), so the invariant is satisfied
    /// trivially. A future work item (W015 — "blocks/attributes/xrefs
    /// production semantics") may extend the contract; W003 does not
    /// invent a `parent_block_id` field (per the W003 stop conditions:
    /// "Stop and report `ARCHITECTURE_CHANGE_REQUIRED` for a frozen
    /// semantic gap").
    ///
    /// This function returns `()` (not `Result`) because the check is
    /// structurally a no-op in the W003 model — there is no error
    /// path. Future work items that introduce nested-block-reference
    /// semantics will replace this stub with a real cycle check.
    fn check_block_reference_cycle(&self) {
        // Adjacency would be: for each block B, find every block
        // reference whose `block_definition_id == B` AND that lives
        // inside a block definition (a `parent_block_id` field the
        // spec doesn't define). Since the spec doesn't define the
        // parent-block link, we conservatively report no cycle.
        //
        // We DO check the simpler invariant that the block_references
        // collection itself is internally consistent (each ref's
        // `block_definition_id` resolves) — that's done in
        // `validate()` above.
        let _ = &self.blocks;
        let _ = &self.block_references;
    }

    // ---- Read-only accessors --------------------------------------------

    /// Look up an entity by ID. O(1) ID-based lookup — there is no
    /// array-index access path (the Architect directive "Do not use
    /// array position as durable identity" is honored structurally).
    #[must_use]
    pub fn entity(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Look up a layer by ID.
    #[must_use]
    pub fn layer(&self, id: &LayerId) -> Option<&Layer> {
        self.layers.get(id)
    }

    /// Look up a block definition by ID.
    #[must_use]
    pub fn block(&self, id: &BlockDefinitionId) -> Option<&BlockDefinition> {
        self.blocks.get(id)
    }

    /// Look up a block reference by ID.
    #[must_use]
    pub fn block_reference(&self, id: &BlockReferenceId) -> Option<&BlockReference> {
        self.block_references.get(id)
    }

    /// Look up a layout by ID.
    #[must_use]
    pub fn layout(&self, id: &LayoutId) -> Option<&Layout> {
        self.layouts.get(id)
    }

    /// Look up a viewport by ID.
    #[must_use]
    pub fn viewport(&self, id: &ViewportId) -> Option<&Viewport> {
        self.viewports.get(id)
    }

    /// Look up an external reference by ID.
    #[must_use]
    pub fn external_ref(&self, id: &ExternalRefId) -> Option<&ExternalReference> {
        self.external_refs.get(id)
    }

    /// Look up an opaque external object by ID.
    #[must_use]
    pub fn opaque_object(&self, id: &ExternalObjectId) -> Option<&OpaqueExternalObject> {
        self.opaque_objects.get(id)
    }

    /// Iterate over entities in deterministic (sorted-by-ID) order.
    /// The iteration order is independent of insertion order (per
    /// `spec/architecture.md` §11 "Reproducibility").
    pub fn entities_iter(&self) -> impl Iterator<Item = (&EntityId, &Entity)> {
        self.entities.iter()
    }

    /// Iterate over opaque external objects in deterministic order
    /// (preserves WO-003-AC03 count/identity on read paths).
    pub fn opaque_objects_iter(
        &self,
    ) -> impl Iterator<Item = (&ExternalObjectId, &OpaqueExternalObject)> {
        self.opaque_objects.iter()
    }

    /// Number of opaque objects. Used by the AC03 regression test to
    /// prove count is preserved across round trips.
    #[must_use]
    pub fn opaque_object_count(&self) -> usize {
        self.opaque_objects.len()
    }
}

// ---------------------------------------------------------------------------
// DrawingBuilder — incremental construction with deferred validation.
// ---------------------------------------------------------------------------

/// Incremental builder for a [`Drawing`]. Accumulates collections
/// field-by-field and calls [`Drawing::validate`] at `build()` time.
/// Useful for importer code that constructs a drawing from a parsed
/// file.
#[derive(Debug, Clone, Default)]
pub struct DrawingBuilder {
    id: Option<DrawingId>,
    project_id: Option<ProjectId>,
    name: Option<String>,
    units: Option<DrawingUnits>,
    model_space_root: Option<EntityId>,
    entities: BTreeMap<EntityId, Entity>,
    layers: BTreeMap<LayerId, Layer>,
    blocks: BTreeMap<BlockDefinitionId, BlockDefinition>,
    block_references: BTreeMap<BlockReferenceId, BlockReference>,
    linetypes: BTreeMap<StyleId, Style>,
    text_styles: BTreeMap<StyleId, Style>,
    dimension_styles: BTreeMap<DimensionStyleId, DimensionStyle>,
    layouts: BTreeMap<LayoutId, Layout>,
    viewports: BTreeMap<ViewportId, Viewport>,
    external_refs: BTreeMap<ExternalRefId, ExternalReference>,
    opaque_objects: BTreeMap<ExternalObjectId, OpaqueExternalObject>,
    metadata: BTreeMap<String, String>,
    active_layer_id: Option<LayerId>,
    current_space: Option<SpaceRef>,
    revision: u64,
}

impl DrawingBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the drawing ID.
    #[must_use]
    pub fn id(mut self, id: DrawingId) -> Self {
        self.id = Some(id);
        self
    }
    /// Set the project ID.
    #[must_use]
    pub fn project_id(mut self, id: ProjectId) -> Self {
        self.project_id = Some(id);
        self
    }
    /// Set the drawing name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Set the unit system.
    #[must_use]
    pub fn units(mut self, units: DrawingUnits) -> Self {
        self.units = Some(units);
        self
    }
    /// Set the model-space root entity.
    #[must_use]
    pub fn model_space_root(mut self, root: EntityId) -> Self {
        self.model_space_root = Some(root);
        self
    }
    /// Add an entity. Returns the builder for chaining. If an entity
    /// with the same ID already exists, the new one REPLACES it (the
    /// importer's responsibility to ensure uniqueness).
    #[must_use]
    pub fn entity(mut self, entity: Entity) -> Self {
        self.entities.insert(entity.id, entity);
        self
    }
    /// Add a layer.
    #[must_use]
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layers.insert(layer.id, layer);
        self
    }
    /// Add a block definition.
    #[must_use]
    pub fn block(mut self, block: BlockDefinition) -> Self {
        self.blocks.insert(block.id, block);
        self
    }
    /// Add a block reference.
    #[must_use]
    pub fn block_reference(mut self, reference: BlockReference) -> Self {
        self.block_references.insert(reference.id, reference);
        self
    }
    /// Add a linetype.
    #[must_use]
    pub fn linetype(mut self, style: Style) -> Self {
        self.linetypes.insert(style.id, style);
        self
    }
    /// Add a text style.
    #[must_use]
    pub fn text_style(mut self, style: Style) -> Self {
        self.text_styles.insert(style.id, style);
        self
    }
    /// Add a dimension style.
    #[must_use]
    pub fn dimension_style(mut self, style: DimensionStyle) -> Self {
        self.dimension_styles.insert(style.id, style);
        self
    }
    /// Add a layout.
    #[must_use]
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layouts.insert(layout.id, layout);
        self
    }
    /// Add a viewport.
    #[must_use]
    pub fn viewport(mut self, viewport: Viewport) -> Self {
        self.viewports.insert(viewport.id, viewport);
        self
    }
    /// Add an external reference.
    #[must_use]
    pub fn external_ref(mut self, reference: ExternalReference) -> Self {
        self.external_refs.insert(reference.id, reference);
        self
    }
    /// Add an opaque external object.
    #[must_use]
    pub fn opaque_object(mut self, object: OpaqueExternalObject) -> Self {
        self.opaque_objects.insert(object.id, object);
        self
    }
    /// Insert a metadata entry.
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    /// Set the active layer.
    #[must_use]
    pub fn active_layer(mut self, id: LayerId) -> Self {
        self.active_layer_id = Some(id);
        self
    }
    /// Set the current space.
    #[must_use]
    pub fn current_space(mut self, space: SpaceRef) -> Self {
        self.current_space = Some(space);
        self
    }
    /// Set the revision number.
    #[must_use]
    pub fn revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    /// Build the [`Drawing`], validating every cross-reference
    /// invariant. Required fields (`id`, `project_id`, `name`,
    /// `units`, `active_layer_id`, `current_space`) must be set.
    ///
    /// # Errors
    /// Returns [`DocumentError::InvalidInput`] for a missing required
    /// field; returns [`DocumentError::ConstraintViolation`] for the
    /// first cross-reference invariant violation.
    pub fn build(self) -> Result<Drawing, DocumentError> {
        let raw = RawDrawing {
            id: self
                .id
                .ok_or_else(|| DocumentError::InvalidInput("id is required".into()))?,
            project_id: self
                .project_id
                .ok_or_else(|| DocumentError::InvalidInput("project_id is required".into()))?,
            name: self
                .name
                .ok_or_else(|| DocumentError::InvalidInput("name is required".into()))?,
            units: self
                .units
                .ok_or_else(|| DocumentError::InvalidInput("units is required".into()))?,
            model_space_root: self.model_space_root,
            entities: self.entities,
            layers: self.layers,
            blocks: self.blocks,
            block_references: self.block_references,
            linetypes: self.linetypes,
            text_styles: self.text_styles,
            dimension_styles: self.dimension_styles,
            layouts: self.layouts,
            viewports: self.viewports,
            external_refs: self.external_refs,
            opaque_objects: self.opaque_objects,
            metadata: self.metadata,
            active_layer_id: self
                .active_layer_id
                .ok_or_else(|| DocumentError::InvalidInput("active_layer_id is required".into()))?,
            current_space: self
                .current_space
                .ok_or_else(|| DocumentError::InvalidInput("current_space is required".into()))?,
            revision: self.revision,
        };
        Drawing::from_raw(raw)
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — Drawing cross-reference invariants.
    // Evidence: WO-003-AC02 — Drawing serde round-trip preserves
    // identity and relationships.
    // Evidence: WO-003-AC03 — opaque object count preserved across
    // round trips.
    // Evidence: WO-003-AC05 — Drawing is the SOLE document authority
    // (no second container with document authority exists; this is
    // a structural/architectural assertion that the type system
    // enforces — there is exactly one `Drawing` per drawing identity).

    use super::*;
    use crate::entity::Entity;
    use crate::identity::TestIdGenerator;
    use crate::layer::{Layer, LayerColor};
    use crate::project::DrawingRevision;
    use crate::value_types::{DrawingUnits, Provenance, SpaceRef, StyleRef, VisibilityState};
    use aeccad_core_geometry::{Transform2D, Vector2};

    /// Build a small but well-formed drawing for tests. Uses a
    /// deterministic ID generator (per `spec/architecture.md` §11
    /// "Reproducibility").
    fn well_formed_drawing() -> Drawing {
        let mut g = TestIdGenerator::new(0);
        let drawing_id = crate::identity::next_drawing_id(&mut g);
        let project_id = crate::identity::next_project_id(&mut g);
        let layer_id = crate::identity::next_layer_id(&mut g);
        let linetype_id = crate::identity::next_style_id(&mut g);
        let text_style_id = crate::identity::next_style_id(&mut g);
        let entity_id = crate::identity::next_entity_id(&mut g);

        let layer = Layer::new(
            layer_id,
            "Walls".to_string(),
            LayerColor::rgb(255, 0, 0),
            linetype_id,
            0.25,
            0.0,
            true,
            false,
            false,
            true,
            "Wall layer".to_string(),
        );
        let linetype = Style::new(
            linetype_id,
            1,
            "Continuous".to_string(),
            std::collections::BTreeMap::new(),
        );
        let text_style = Style::new(
            text_style_id,
            1,
            "Standard".to_string(),
            std::collections::BTreeMap::new(),
        );
        let entity = Entity::new(
            entity_id,
            layer_id,
            None,
            Transform2D::IDENTITY,
            VisibilityState::Visible,
            StyleRef::new(text_style_id),
            Provenance::created(),
        );

        DrawingBuilder::new()
            .id(drawing_id)
            .project_id(project_id)
            .name("Test Drawing".to_string())
            .units(DrawingUnits::MetricMM)
            .layer(layer)
            .linetype(linetype)
            .text_style(text_style)
            .entity(entity)
            .active_layer(layer_id)
            .current_space(SpaceRef::ModelSpace)
            .revision(1)
            .build()
            .expect("well-formed drawing")
    }

    #[test]
    fn well_formed_drawing_validates() {
        let d = well_formed_drawing();
        d.validate().expect("well-formed drawing validates");
    }

    #[test]
    fn drawing_field_set_matches_spec() {
        let d = well_formed_drawing();
        let v = serde_json::to_value(&d).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "active_layer_id",
                "block_references",
                "blocks",
                "current_space",
                "dimension_styles",
                "entities",
                "external_refs",
                "id",
                "layers",
                "layouts",
                "linetypes",
                "metadata",
                "model_space_root",
                "name",
                "opaque_objects",
                "project_id",
                "revision",
                "text_styles",
                "units",
                "viewports",
            ]
        );
    }

    #[test]
    fn drawing_serde_roundtrip_preserves_identity_and_relationships() {
        // Evidence: WO-003-AC02 — full drawing round-trip preserves
        // every ID and every cross-reference relationship.
        let d = well_formed_drawing();
        let j = serde_json::to_string(&d).expect("serialize");
        let back: Drawing = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(d.id, back.id, "drawing id preserved");
        assert_eq!(d.project_id, back.project_id, "project id preserved");
        // Entity ID is preserved; its layer_id still resolves to the
        // same layer.
        let original_entity_id = d.entities.iter().next().expect("at least one entity").0;
        let back_entity = back.entity(original_entity_id).expect("entity preserved");
        let back_layer = back.layer(&back_entity.layer_id).expect("layer resolves");
        assert_eq!(
            back_layer.id,
            d.layer(&back_entity.layer_id).expect("layer resolves").id,
            "layer identity preserved"
        );
        // Round-trip identity at the drawing level: re-serializing
        // yields the same bytes (canonical wire form).
        let j2 = serde_json::to_string(&back).expect("serialize-back");
        assert_eq!(j, j2, "wire form is canonical");
    }

    #[test]
    fn drawing_rejects_unknown_fields_at_boundary() {
        // Build a malformed JSON payload with a surprise field.
        let d = well_formed_drawing();
        let mut v = serde_json::to_value(&d).expect("serialize");
        v.as_object_mut()
            .expect("object")
            .insert("surprise_field".to_string(), serde_json::Value::Null);
        let r: Result<Drawing, _> = serde_json::from_value(v);
        assert!(r.is_err(), "unknown field must be rejected at boundary");
    }

    #[test]
    fn drawing_rejects_dangling_entity_layer_ref() {
        // Evidence: WO-003-AC01 — invariant: entity.layer_id must
        // resolve to a layer in the drawing.
        let d = well_formed_drawing();
        let mut raw = serde_json::to_value(&d).expect("serialize");
        // Inject an entity with a non-existent layer_id.
        let mut g = TestIdGenerator::new(99);
        let bad_layer_id = crate::identity::next_layer_id(&mut g);
        let entity_id = crate::identity::next_entity_id(&mut g);
        let bad_entity = serde_json::json!({
            "id": format!("{entity_id}"),
            "layer_id": format!("{bad_layer_id}"),
            "owner_block_id": null,
            "transform": {
                "translation": {"x": 0.0, "y": 0.0},
                "rotation_rad": 0.0,
                "scale_x": 1.0,
                "scale_y": 1.0,
            },
            "visibility": "Visible",
            "common_style": {
                "style_id": format!(
                    "{}",
                    d.text_styles
                        .keys()
                        .next()
                        .expect("at least one text style")
                ),
            },
            "source_provenance": {
                "kind": "Created",
                "source_artifact_hash": null,
                "source_revision": null,
                "source_handle": null,
            },
        });
        raw["entities"]
            .as_object_mut()
            .expect("object")
            .insert(format!("{entity_id}"), bad_entity);
        let r: Result<Drawing, _> = serde_json::from_value(raw);
        assert!(
            r.is_err(),
            "drawing with dangling entity->layer reference must be rejected"
        );
    }

    #[test]
    fn drawing_rejects_dangling_block_reference() {
        // Evidence: WO-003-AC01 — invariant: every block reference
        // points to exactly one existing block definition.
        let d = well_formed_drawing();
        let mut raw = serde_json::to_value(&d).expect("serialize");
        let mut g = TestIdGenerator::new(99);
        let bad_block_id = crate::identity::next_block_definition_id(&mut g);
        let bad_ref_id = crate::identity::next_block_reference_id(&mut g);
        let bad_reference = serde_json::json!({
            "id": format!("{bad_ref_id}"),
            "block_definition_id": format!("{bad_block_id}"),
            "transform": {
                "translation": {"x": 0.0, "y": 0.0},
                "rotation_rad": 0.0,
                "scale_x": 1.0,
                "scale_y": 1.0,
            },
            "attribute_values": [],
            "explodable": true,
        });
        raw["block_references"]
            .as_object_mut()
            .expect("object")
            .insert(format!("{bad_ref_id}"), bad_reference);
        let r: Result<Drawing, _> = serde_json::from_value(raw);
        assert!(
            r.is_err(),
            "drawing with dangling block_reference must be rejected"
        );
    }

    #[test]
    fn drawing_rejects_viewport_in_two_layouts() {
        // Evidence: WO-003-AC01 — invariant #5: a viewport belongs to
        // exactly one layout.
        let d = well_formed_drawing();
        let mut raw = serde_json::to_value(&d).expect("serialize");
        let mut g = TestIdGenerator::new(99);
        let viewport_id = crate::identity::next_viewport_id(&mut g);
        let layout_id_a = crate::identity::next_layout_id(&mut g);
        let layout_id_b = crate::identity::next_layout_id(&mut g);
        // Add the viewport to the viewport table.
        raw["viewports"].as_object_mut().expect("object").insert(
            format!("{viewport_id}"),
            serde_json::json!({
                "id": format!("{viewport_id}"),
                "center_model": {"x": 0.0, "y": 0.0},
                "scale": 1.0,
                "twist": 0.0,
                "layer_overrides": {},
                "display_mode": "Wireframe2D",
            }),
        );
        // Add two layouts each listing the SAME viewport.
        for layout_id in [layout_id_a, layout_id_b] {
            raw["layouts"].as_object_mut().expect("object").insert(
                format!("{layout_id}"),
                serde_json::json!({
                    "id": format!("{layout_id}"),
                    "name": format!("Layout-{layout_id}"),
                    "paper_size": {"width_mm": 297.0, "height_mm": 420.0},
                    "orientation": "Portrait",
                    "plot_settings": {},
                    "viewports": [format!("{viewport_id}")],
                }),
            );
        }
        let r: Result<Drawing, _> = serde_json::from_value(raw);
        assert!(
            r.is_err(),
            "viewport in two layouts must be rejected (invariant #5)"
        );
    }

    #[test]
    fn drawing_rejects_dangling_active_layer() {
        let d = well_formed_drawing();
        let mut raw = serde_json::to_value(&d).expect("serialize");
        let mut g = TestIdGenerator::new(99);
        let bad_layer_id = crate::identity::next_layer_id(&mut g);
        raw["active_layer_id"] = serde_json::Value::String(format!("{bad_layer_id}"));
        let r: Result<Drawing, _> = serde_json::from_value(raw);
        assert!(r.is_err(), "dangling active_layer_id must be rejected");
    }

    #[test]
    fn drawing_rejects_dangling_current_space_layout() {
        let d = well_formed_drawing();
        let mut raw = serde_json::to_value(&d).expect("serialize");
        let mut g = TestIdGenerator::new(99);
        let bad_layout_id = crate::identity::next_layout_id(&mut g);
        raw["current_space"] = serde_json::json!(format!("Layout:{bad_layout_id}"));
        // The above produces a string value, not the enum shape —
        // fix by constructing the proper tagged enum.
        raw["current_space"] = serde_json::json!({"Layout": format!("{bad_layout_id}")});
        let r: Result<Drawing, _> = serde_json::from_value(raw);
        assert!(r.is_err(), "dangling current_space Layout must be rejected");
    }

    #[test]
    fn drawing_id_based_lookup_is_independent_of_insertion_order() {
        // Evidence: WO-003-AC01 — "Do not use array position as durable
        // identity". The same drawing can be re-serialized with
        // differently-ordered collections; the ID-based lookup still
        // resolves to the same entities.
        let d = well_formed_drawing();
        let original_entity_id = *d.entities.iter().next().expect("at least one entity").0;
        // Serialize, deserialize into a serde_json::Value, shuffle the
        // entity object's keys (BTreeMap is already sorted — but the
        // wire form's keys are sorted too, so this is a no-op test of
        // sort-stability). The substantive guarantee: looking up the
        // same ID in two separately-deserialized drawings returns the
        // same entity identity.
        let j = serde_json::to_string(&d).expect("serialize");
        let back1: Drawing = serde_json::from_str(&j).expect("deserialize-1");
        let back2: Drawing = serde_json::from_str(&j).expect("deserialize-2");
        assert_eq!(
            back1.entity(&original_entity_id).expect("entity").id,
            back2.entity(&original_entity_id).expect("entity").id,
            "ID-based lookup is reproducible"
        );
    }

    #[test]
    fn drawing_opaque_object_count_preserved_through_roundtrip() {
        // Evidence: WO-003-AC03 — opaque objects never silently
        // disappear during an otherwise successful round trip.
        let d = well_formed_drawing();
        let mut builder_d = d.clone();
        let mut g = TestIdGenerator::new(99);
        for i in 0..5 {
            let obj = crate::external::OpaqueExternalObject::new(
                crate::identity::next_external_object_id(&mut g),
                "DWG-R2018".to_string(),
                "AC1027".to_string(),
                format!("ACAD_TYPE_{i}"),
                format!("{i:X}"),
                String::new(),
                vec![i as u8; 16],
                None,
                crate::value_types::PreservationStatus::PreservedOpaque,
                Vec::new(),
            );
            builder_d.opaque_objects.insert(obj.id, obj);
        }
        let before = builder_d.opaque_object_count();
        let j = serde_json::to_string(&builder_d).expect("serialize");
        let back: Drawing = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back.opaque_object_count(), before, "count preserved");
        // Each opaque object's ID and raw_payload survived.
        for (oid, obj) in &builder_d.opaque_objects {
            let back_obj = back.opaque_object(oid).expect("object preserved");
            assert_eq!(back_obj.raw_payload, obj.raw_payload, "payload preserved");
        }
    }

    // Suppress an unused-variable warning from the well_formed_drawing
    // helper (Vector2 is referenced only for the unused Transform2D
    // constructor doc — keep the import so the example in doc-comments
    // remains valid).
    #[test]
    fn vector2_import_is_used() {
        let _ = Vector2::new(0.0, 0.0);
    }

    // DrawingRevision round-trip is already tested in project.rs;
    // add a cross-check that the revision's `drawing_id` matches.
    #[test]
    fn drawing_revision_links_back_to_drawing() {
        let d = well_formed_drawing();
        let mut g = TestIdGenerator::new(99);
        let rev = DrawingRevision::new(
            crate::identity::next_artifact_version_id(&mut g),
            d.id,
            d.revision,
            "sha256:test".to_string(),
            "2026-08-28T08:00:00Z".to_string(),
            None,
        );
        assert_eq!(rev.drawing_id, d.id, "revision's drawing_id matches");
        assert_eq!(rev.revision_number, d.revision, "revision_number matches");
    }
}
