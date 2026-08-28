//! core-document — the authoritative canonical document model (W003).
//!
//! Per `spec/architecture.md` §2 "`core-document`":
//! > Canonical document graph, entities, IDs, layers, blocks, styles,
//! > layouts, references and metadata. Depends on geometry.
//!
//! And per `spec/architecture-lock.md` §1 "Canonical model authority":
//! > the internal AEC CAD document model is the application source of
//! > truth. DWG/DXF/PDF are external representations.
//!
//! This crate implements the W003 work-order scope (architecture v1.1
//! FROZEN): the canonical document graph and entity/object identity
//! model. It is the SOLE document authority — there is no second
//! container that owns entities/layers/blocks/etc. (per the Architect
//! directive: "Do not create a second document authority").
//!
//! # Frozen-contract invariants honored here
//!
//! - **Opaque, sortable, type-distinct IDs** (per
//!   `spec/domain-model.md` §"Identity types"): every ID type is a
//!   separate newtype wrapping a 128-bit opaque value. The compiler
//!   rejects passing a `LayerId` where an `EntityId` is expected.
//!   IDs are NEVER used as array indices — collections are
//!   `BTreeMap<Id, T>` keyed by stable ID (per the Architect
//!   directive: "Do not use array position as durable identity").
//! - **Closed enum sets** (per `spec/domain-model.md` §"Closed value
//!   types" and §"Relationship invariants" #14): every enum's variant
//!   set is closed and rejects unknown values at the deserialization
//!   boundary (`#[serde(deny_unknown_fields)]`).
//! - **f64 finiteness** (per `spec/domain-model.md` §"Core value types
//!   and invariants"): all `f64` fields are validated for finiteness
//!   at the `Deserialize` boundary via private `Raw` shadows + `TryFrom`.
//! - **Unknown-field rejection** (per `spec/domain-model.md` §"Closed
//!   value types": "Unknown/extra fields are forbidden in canonical
//!   persisted DTOs."): every DTO uses `#[serde(deny_unknown_fields)]`.
//! - **External-handle preservation** (per `spec/domain-model.md`
//!   §"Identity types": "Handles originating from external formats are
//!   preserved separately and never reused as primary IDs."):
//!   `Provenance.source_handle` and
//!   `OpaqueExternalObject.external_handle` are preserved as opaque
//!   metadata fields, never elevated to primary IDs.
//! - **Opaque-object preservation** (per architecture-lock §5 and §7
//!   and `spec/domain-model.md` §"Interoperability entities":
//!   `OpaqueExternalObject.raw_payload` and `proxy_graphics` are
//!   preserved verbatim through round trips; the spec invariant
//!   "opaque objects can never silently disappear during an otherwise
//!   successful round trip" is enforced structurally (the
//!   `Drawing.opaque_objects` collection round-trips as a sorted-key
//!   map — count and content are preserved).
//! - **Reproducibility** (per `spec/architecture.md` §11): no
//!   wall-clock time, no `HashMap` iteration, no uncontrolled
//!   randomness. The canonical model does not generate IDs at commit
//!   time; IDs are part of the importer/command-engine input. The
//!   test-only [`TestIdGenerator`](crate::identity::TestIdGenerator)
//!   is fully deterministic.
//! - **Authority hierarchy** (per `spec/architecture.md` §8): this
//!   crate implements the canonical domain model; it is the
//!   authority. Code in `app-shell` / `app-ui` may consume
//!   read-only projections of `Drawing` but may NOT own domain state
//!   or mutate the canonical model directly (per §9 and WO-003-AC05).
//!   W003 enforces this structurally: there is no `&mut self` method
//!   on `Drawing` that mutates content (mutator methods belong to
//!   future W006 command-engine code; W003 provides only the
//!   immutable `Drawing::validate()` boundary and read-only
//!   accessors).
//!
//! # Out-of-scope (per W003 work-order §"Forbidden changes")
//!
//! - geometry algorithms beyond Transform2D value-type integration
//!   (W002 owns geometry);
//! - commands / mutation transactions (W006 owns the command engine);
//! - UI (Tauri shell + React UI — out of W003 scope);
//! - interop (DWG/DXF reader/writer — W010/W011/W012 own interop);
//! - electrical modules (W016+ own electrical).
//!
//! # Module map
//!
//! - [`identity`] — `Id128` opaque wire type + per-purpose ID newtypes
//!   + `IdGenerator` trait + `TestIdGenerator`.
//! - [`value_types`] — closed enums (`VisibilityState`, `SpaceRef`,
//!   `SourceKind`, `PreservationStatus`, `ProvenanceKind`,
//!   `DrawingUnits`, `PaperOrientation`) + `Provenance`, `StyleRef`,
//!   `RatingSet`.
//! - [`error`] — closed `DocumentError` variant set per `spec/api.md`
//!   §"Error contract".
//! - [`entity`] — `Entity` shell (id/layer_id/owner_block_id/transform/
//!   visibility/common_style/source_provenance).
//! - [`layer`] — `Layer` + `LayerColor`.
//! - [`block`] — `BlockDefinition` + `BlockReference`.
//! - [`style`] — `Style` + `DimensionStyle`.
//! - [`layout`] — `Layout` + `Viewport` + `PaperSize` +
//!   `LayerOverride` + `DisplayMode`.
//! - [`external`] — `ExternalReference` + `OpaqueExternalObject`.
//! - [`project`] — `Project` + `ProjectStatus` + `DrawingRevision`.
//! - [`drawing`] — `Drawing` root container + `DrawingBuilder` +
//!   `Drawing::validate()` (the canonical-model boundary).

#![forbid(unsafe_code)]
// Pedantic relaxations mirroring the geometry crate's policy:
// - ID and DTO type names mirror module names by design (e.g.
//   `drawing::Drawing`); module-name-repetition is intentional.
// - Document DTOs use single-char field names where the spec uses
//   them (e.g. `LayerColor.r/g/b`); many-single-char-names is noise
//   here.
// - Cast lints are not relevant (no casts in this crate).
// - `must_use` friction: `Result`/`Option` are already `#[must_use]`
//   by the language; redundant-attr and candidate lints are noise.
// - `# Errors`/`# Panics` doc sections on every `Result`/panic-ing
//   fn are heavy for a DTO crate; invariants are documented at the
//   type/method level.
#![allow(
    clippy::module_name_repetitions,
    clippy::many_single_char_names,
    clippy::must_use_candidate,
    clippy::double_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::similar_names,
    // The canonical document DTOs match the frozen `spec/domain-model.md`
    // field sets exactly. `Layer` has 4 booleans (visible/locked/frozen/
    // plot_enabled) and `LayerOverride` mirrors that set — these are
    // spec-mandated, not a design choice. Reducing them would deviate
    // from the frozen contract.
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    // The canonical document constructors (Layer::new,
    // OpaqueExternalObject::new, etc.) take all spec-mandated fields as
    // positional arguments. Splitting them into a builder pattern
    // everywhere would obscure the spec-faithful field list. The
    // spec-loyal `DrawingBuilder` exists for incremental construction;
    // the per-type constructors are spec-mirrors.
    clippy::too_many_arguments,
    // Doc-comment identifiers (e.g. `font_name`, `linetype_pattern`) are
    // quoted-as-strings in many places to emphasize they are wire-form
    // string keys, not Rust identifiers. Backticking every quoted-string
    // key would clutter the prose; the meaning is unambiguous.
    clippy::doc_markdown
)]

pub mod block;
pub mod drawing;
pub mod entity;
pub mod error;
pub mod external;
pub mod identity;
pub mod layer;
pub mod layout;
pub mod project;
pub mod style;
pub mod value_types;

// ---------------------------------------------------------------------------
// Public surface re-exports (flat) — ergonomic for callers.
// ---------------------------------------------------------------------------

pub use block::{AttributeId, BlockDefinition, BlockReference};
pub use drawing::{Drawing, DrawingBuilder};
pub use entity::Entity;
pub use error::DocumentError;
pub use external::{ExternalReference, OpaqueExternalObject};
pub use identity::{
    ArtifactVersionId, BlockDefinitionId, BlockReferenceId, DimensionStyleId, DrawingId, EntityId,
    ExternalObjectId, ExternalRefId, IdGenerator, LayerId, LayoutId, ProjectId, StyleId,
    TestIdGenerator, ViewportId,
};
pub use layer::{Layer, LayerColor};
pub use layout::{DisplayMode, LayerOverride, Layout, PaperSize, Viewport};
pub use project::{DrawingRevision, Project, ProjectStatus};
pub use style::{DimensionStyle, Style};
pub use value_types::{
    DrawingUnits, PaperOrientation, PreservationStatus, Provenance, ProvenanceKind, RatingSet,
    SourceKind, SpaceRef, StyleRef, VisibilityState,
};

/// Returns the module name for baseline architecture tests.
///
/// Kept from W001 (baseline gate asserts this constant matches the
/// `spec/architecture.md` §2 boundary name).
pub const MODULE_NAME: &str = "core-document";

#[cfg(test)]
mod tests {
    // Evidence: WO-001-AC02 — module boundary matches `spec/architecture.md` §2.
    // Evidence: WO-001-AC04 — deterministic baseline unit test harness.
    // Evidence: WO-003-AC04 — Generic CAD modules do not import
    // electrical modules (parse own Cargo.toml at test time).
    // Evidence: WO-003-AC05 — No UI code owns document authority
    // (parse own Cargo.toml at test time; assert no UI/Tauri deps).

    use std::collections::HashSet;

    #[test]
    fn module_boundary_matches_spec() {
        assert_eq!(super::MODULE_NAME, "core-document");
    }

    #[test]
    fn no_aeccad_dependencies_outside_core_geometry() {
        // Evidence: WO-003-AC04 — Generic CAD modules do not import
        // electrical modules (or any aeccad-* module outside the
        // frozen architecture's allowed set). Per
        // `scripts/verify-architecture-dependencies.sh`, the only
        // permitted `aeccad-*` dependency for `core-document` is
        // `aeccad-core-geometry`. Parse the crate manifest at test
        // time and assert that no other `aeccad-*` crate appears in
        // [dependencies] or [dev-dependencies].
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
            found.iter().all(|n| n == "aeccad-core-geometry"),
            "core-document may depend only on aeccad-core-geometry among aeccad-* crates; found: {found:?}"
        );
        assert!(
            found.contains("aeccad-core-geometry"),
            "core-document must depend on aeccad-core-geometry (its parent in the architecture DAG)"
        );
    }

    #[test]
    fn no_ui_or_tauri_dependencies() {
        // Evidence: WO-003-AC05 — No UI code owns document authority.
        // The canonical document model has no UI/Tauri/React/TypeScript
        // dependency. Parse the crate manifest and assert that no
        // UI-shaped crate appears in [dependencies] (UI belongs to
        // app-shell / app-ui, which CONSUME the canonical model but
        // do not own it).
        let manifest = include_str!("../Cargo.toml");
        let mut deps_section = false;
        let mut forbidden: Vec<String> = Vec::new();
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
                let n = name.to_ascii_lowercase();
                if n.contains("tauri")
                    || n.contains("tao")
                    || n.contains("wry")
                    || n.contains("iced")
                    || n.contains("egui")
                    || n.contains("slint")
                    || n.contains("dioxus")
                    || n.contains("leptos")
                    || n.contains("yew")
                    || n.contains("sycamore")
                    || n.contains("gtk")
                    || n.contains("qt")
                {
                    forbidden.push(name.to_string());
                }
            }
        }
        assert!(
            forbidden.is_empty(),
            "core-document must not depend on any UI/Tauri crate (UI does not own document authority); found: {forbidden:?}"
        );
    }

    #[test]
    fn no_second_document_authority_type() {
        // Evidence: WO-003-AC05 — there is no second document authority.
        // The `Drawing` struct (in `drawing.rs`) is the SOLE public
        // container that owns the entity table. Internal helpers
        // (`RawDrawing` for the serde boundary, `DrawingBuilder` for
        // incremental construction) carry an entity collection as a
        // PRIVATE field — they are not authorities, they are
        // construction/wire-shape scaffolds that feed into `Drawing`
        // through `Drawing::from_raw` / `DrawingBuilder::build`. This
        // test asserts that the `pub entities: BTreeMap<EntityId,
        // Entity>` declaration appears in EXACTLY ONE struct (the
        // `Drawing`), which structurally enforces the "no second
        // authority" directive.
        let drawing_src = include_str!("./drawing.rs");
        let entity_src = include_str!("./entity.rs");
        let layer_src = include_str!("./layer.rs");
        let block_src = include_str!("./block.rs");
        let layout_src = include_str!("./layout.rs");
        let style_src = include_str!("./style.rs");
        let external_src = include_str!("./external.rs");
        let project_src = include_str!("./project.rs");
        let value_types_src = include_str!("./value_types.rs");
        let identity_src = include_str!("./identity.rs");
        let error_src = include_str!("./error.rs");
        let sources = [
            drawing_src,
            entity_src,
            layer_src,
            block_src,
            layout_src,
            style_src,
            external_src,
            project_src,
            value_types_src,
            identity_src,
            error_src,
        ];
        let mut public_authority_declarations = 0;
        for src in sources {
            for line in src.lines() {
                let l = line.trim_start();
                // A "public authority" declaration is a struct field
                // declared with `pub entities: BTreeMap<EntityId,
                // Entity>`. Internal helpers use non-pub fields
                // (`entities: BTreeMap<...>` without `pub`) and so
                // are excluded — they are not part of the public API.
                if l.starts_with("pub entities") && l.contains("BTreeMap<EntityId, Entity>") {
                    public_authority_declarations += 1;
                }
            }
        }
        assert_eq!(
            public_authority_declarations, 1,
            "exactly one public struct (Drawing) may own the entity table as a public field; \
             found {public_authority_declarations} declarations"
        );
    }

    #[test]
    fn external_dependency_set_is_minimal() {
        // Evidence: WO-003-AC04 — the only non-aeccad dependency
        // permitted in [dependencies] is `serde` (with derive). This
        // mirrors the W002 geometry crate's discipline and keeps the
        // canonical model free of file-format / network / system-time
        // dependencies.
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
        let non_aeccad: Vec<&String> = names.iter().filter(|n| !n.starts_with("aeccad-")).collect();
        assert_eq!(
            non_aeccad.iter().filter(|n| **n != "serde").count(),
            0,
            "core-document may depend only on serde (and aeccad-core-geometry) in [dependencies]; found extra: {non_aeccad:?}"
        );
    }
}
