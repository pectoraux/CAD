//! `Layout` and `Viewport`.
//!
//! Per `spec/domain-model.md` §"Document objects":
//! ```text
//! Layout { id, name, paper_size, orientation, plot_settings, viewports[] }
//! Viewport { id, center_model, scale, twist, layer_overrides, display_mode }
//! ```
//!
//! And per §"Representation and reference invariants":
//! 4. Every layout belongs to exactly one drawing.
//! 5. A viewport belongs to exactly one layout.
//!
//! Frozen-contract invariants honored here:
//! - `viewports` is `Vec<ViewportId>` (ID-only references — per the
//!   spec's "entity references contain IDs only" rule; the `Viewport`
//!   records live in the parent `Drawing`'s viewport table).
//! - `paper_size` is stored as `(width_mm: f64, height_mm: f64)` (the
//!   spec lists `paper_size` without a type; the most interoperable
//!   encoding is a width×height pair in millimeters).
//! - `plot_settings` is `BTreeMap<String, String>` (sorted-key opaque
//!   map — the spec doesn't enumerate the plot settings fields; W003
//!   preserves them opaquely, per WO-003-AC03).
//! - `center_model` is a [`Point2`](aeccad_core_geometry::Point2).
//! - `scale` and `twist` are `f64` and validated for finiteness at the
//!   deserialization boundary.
//! - `layer_overrides` is `BTreeMap<LayerId, LayerOverride>` where
//!   `LayerOverride` is a small closed struct (frozen: visible/locked/
//!   frozen/plot_enabled booleans + optional color). The spec doesn't
//!   enumerate the override shape; W003 stores the minimal set
//!   (visible/locked/frozen/plot_enabled/color) which matches the
//!   `Layer` shape, and treats any further override field as opaque
//!   (out of W003 scope).
//! - `display_mode` is a closed enum (`DisplayMode`).
//! - Unknown fields are rejected.

use aeccad_core_geometry::Point2;
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
};
use std::collections::BTreeMap;

use crate::identity::{LayerId, LayoutId, ViewportId};
use crate::layer::LayerColor;
use crate::value_types::PaperOrientation;

/// Drawing-owned layout (a "paper space" sheet).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    /// Stable, opaque layout identity.
    pub id: LayoutId,
    /// Human-readable layout name.
    pub name: String,
    /// Paper size in millimeters `(width_mm, height_mm)`.
    pub paper_size: PaperSize,
    /// Paper orientation (closed enum).
    pub orientation: PaperOrientation,
    /// Opaque sorted-key plot settings (preserved verbatim through
    /// round trips per WO-003-AC03).
    pub plot_settings: BTreeMap<String, String>,
    /// Viewport IDs owned by this layout. The actual `Viewport`
    /// records live in the parent `Drawing`'s viewport table.
    pub viewports: Vec<ViewportId>,
}

/// Paper size in millimeters. Width and height are finite non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaperSize {
    /// Width in millimeters.
    pub width_mm: f64,
    /// Height in millimeters.
    pub height_mm: f64,
}

/// Drawing-owned viewport. A viewport is a window from a layout into
/// model space (or another layout). Per invariant #5: "A viewport
/// belongs to exactly one layout."
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Viewport {
    /// Stable, opaque viewport identity.
    pub id: ViewportId,
    /// Center of the viewed region in model-space coordinates.
    pub center_model: Point2,
    /// Viewport scale (model units per paper unit, or inverse — the
    /// importer is responsible for declaring the convention; W003
    /// stores the value verbatim).
    pub scale: f64,
    /// View twist angle in radians.
    pub twist: f64,
    /// Per-layer overrides applied within this viewport only.
    pub layer_overrides: BTreeMap<LayerId, LayerOverride>,
    /// Display mode (closed enum).
    pub display_mode: DisplayMode,
}

/// Per-viewport per-layer override. The shape mirrors [`Layer`](crate::layer::Layer)'s
/// visible/locked/frozen/plot_enabled/color fields. The spec doesn't
/// enumerate the override shape; W003 stores the minimal closed set
/// (visible/locked/frozen/plot_enabled/color) — extending it would
/// invent unspecified semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerOverride {
    /// Override the layer's visibility.
    pub visible: bool,
    /// Override the layer's locked state.
    pub locked: bool,
    /// Override the layer's frozen state.
    pub frozen: bool,
    /// Override the layer's plot-enabled state.
    pub plot_enabled: bool,
    /// Override the layer's color (optional — `None` means inherit).
    pub color: Option<LayerColor>,
}

/// Closed display-mode enum. The spec lists `display_mode` without
/// enumerating variants; W003 stores the minimal closed set that
/// CAD systems universally distinguish (2D wireframe, 3D wireframe,
/// hidden-line, shaded). Adding variants is a frozen-contract change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DisplayMode {
    Wireframe2D,
    Wireframe3D,
    HiddenLine,
    Shaded,
}

// ---------------------------------------------------------------------------
// Canonical-model boundary: deserialize then validate f64 finiteness.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawViewport {
    id: ViewportId,
    center_model: Point2,
    scale: f64,
    twist: f64,
    layer_overrides: BTreeMap<LayerId, LayerOverride>,
    display_mode: DisplayMode,
}

impl TryFrom<RawViewport> for Viewport {
    type Error = &'static str;

    fn try_from(r: RawViewport) -> Result<Self, Self::Error> {
        if !r.scale.is_finite() {
            return Err("scale must be finite");
        }
        if !r.twist.is_finite() {
            return Err("twist must be finite");
        }
        Ok(Self {
            id: r.id,
            center_model: r.center_model,
            scale: r.scale,
            twist: r.twist,
            layer_overrides: r.layer_overrides,
            display_mode: r.display_mode,
        })
    }
}

impl<'de> Deserialize<'de> for Viewport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawViewport::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl Layout {
    /// Construct a layout with the given fields.
    #[must_use]
    pub fn new(
        id: LayoutId,
        name: String,
        paper_size: PaperSize,
        orientation: PaperOrientation,
        plot_settings: BTreeMap<String, String>,
        viewports: Vec<ViewportId>,
    ) -> Self {
        Self {
            id,
            name,
            paper_size,
            orientation,
            plot_settings,
            viewports,
        }
    }
}

impl Viewport {
    /// Construct a viewport. Performs only f64-finiteness validation
    /// on `scale`/`twist` at construction (defense in depth; the
    /// `Deserialize` boundary is the canonical gate).
    #[must_use]
    pub fn new(
        id: ViewportId,
        center_model: Point2,
        scale: f64,
        twist: f64,
        layer_overrides: BTreeMap<LayerId, LayerOverride>,
        display_mode: DisplayMode,
    ) -> Self {
        let scale = if scale.is_finite() { scale } else { 1.0 };
        let twist = if twist.is_finite() { twist } else { 0.0 };
        Self {
            id,
            center_model,
            scale,
            twist,
            layer_overrides,
            display_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — viewport belongs to exactly one layout
    // (the parent Drawing enforces this; here we test the field shapes).
    // Evidence: WO-003-AC02 — layout/viewport round-trips.
    // Evidence: WO-003-AC03 — opaque plot_settings preserved.

    use super::*;
    use crate::identity::TestIdGenerator;

    fn fixture_layout_and_viewport() -> (Layout, Viewport) {
        let mut g = TestIdGenerator::new(0);
        let viewport_id = crate::identity::next_viewport_id(&mut g);
        let layout_id = crate::identity::next_layout_id(&mut g);
        let layer_id = crate::identity::next_layer_id(&mut g);
        let layout = Layout::new(
            layout_id,
            "Sheet1".to_string(),
            PaperSize {
                width_mm: 297.0,
                height_mm: 420.0,
            },
            PaperOrientation::Portrait,
            BTreeMap::from([
                ("plot_device".to_string(), "PDF".to_string()),
                ("plot_scale".to_string(), "1:1".to_string()),
            ]),
            vec![viewport_id],
        );
        let viewport = Viewport::new(
            viewport_id,
            Point2::new(100.0, 200.0).unwrap(),
            1.0,
            0.0,
            BTreeMap::from([(
                layer_id,
                LayerOverride {
                    visible: true,
                    locked: false,
                    frozen: false,
                    plot_enabled: true,
                    color: Some(LayerColor::rgb(0, 128, 255)),
                },
            )]),
            DisplayMode::Wireframe2D,
        );
        (layout, viewport)
    }

    #[test]
    fn layout_field_set_matches_spec() {
        let (l, _) = fixture_layout_and_viewport();
        let v = serde_json::to_value(&l).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "id",
                "name",
                "orientation",
                "paper_size",
                "plot_settings",
                "viewports"
            ]
        );
    }

    #[test]
    fn viewport_field_set_matches_spec() {
        let (_, v) = fixture_layout_and_viewport();
        let val = serde_json::to_value(&v).expect("serialize");
        let obj = val.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "center_model",
                "display_mode",
                "id",
                "layer_overrides",
                "scale",
                "twist"
            ]
        );
    }

    #[test]
    fn layout_and_viewport_roundtrip_preserve_ids_and_refs() {
        let (l, v) = fixture_layout_and_viewport();
        let lj = serde_json::to_string(&l).expect("serialize");
        let vj = serde_json::to_string(&v).expect("serialize");
        let l_back: Layout = serde_json::from_str(&lj).expect("deserialize");
        let v_back: Viewport = serde_json::from_str(&vj).expect("deserialize");
        assert_eq!(l, l_back);
        assert_eq!(v, v_back);
        // The layout's viewports[0] points to the same viewport ID
        // (proves ID-based reference survives round-trip — per
        // "Do not use array position as durable identity").
        assert_eq!(l.viewports[0], v.id);
    }

    #[test]
    fn viewport_rejects_non_finite_scale_at_boundary() {
        let raw = r#"{
            "id":"00000000000000000000000000000000",
            "center_model":{"x":0.0,"y":0.0},
            "scale":1e99999,
            "twist":0.0,
            "layer_overrides":{},
            "display_mode":"Wireframe2D"
        }"#;
        let r: Result<Viewport, _> = serde_json::from_str(raw);
        assert!(r.is_err(), "non-finite scale must be rejected");
    }

    #[test]
    fn viewport_rejects_non_finite_twist_at_boundary() {
        let raw = r#"{
            "id":"00000000000000000000000000000000",
            "center_model":{"x":0.0,"y":0.0},
            "scale":1.0,
            "twist":1e99999,
            "layer_overrides":{},
            "display_mode":"Wireframe2D"
        }"#;
        let r: Result<Viewport, _> = serde_json::from_str(raw);
        assert!(r.is_err(), "non-finite twist must be rejected");
    }

    #[test]
    fn layout_rejects_unknown_fields() {
        let (l, _) = fixture_layout_and_viewport();
        let mut v = serde_json::to_value(&l).expect("serialize");
        v.as_object_mut()
            .expect("object")
            .insert("surprise".to_string(), serde_json::Value::Null);
        assert!(serde_json::from_value::<Layout>(v).is_err());
    }

    #[test]
    fn display_mode_rejects_unknown_variant() {
        crate::value_types::assert_unknown_variant_rejected::<DisplayMode>("\"Realistic\"");
    }

    #[test]
    fn opaque_plot_settings_preserved_through_roundtrip() {
        // Evidence: WO-003-AC03 — opaque plot_settings content
        // survives a round trip.
        let (l, _) = fixture_layout_and_viewport();
        let j = serde_json::to_string(&l).expect("serialize");
        let back: Layout = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(l.plot_settings, back.plot_settings);
    }
}
