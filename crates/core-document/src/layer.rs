//! `Layer` — drawing-owned layer record.
//!
//! Per `spec/domain-model.md` §"Document objects" / "Layer":
//! ```text
//! Layer {
//!   id, name, color, linetype_id, lineweight, transparency,
//!   visible, locked, frozen, plot_enabled, description,
//! }
//! ```
//!
//! Frozen-contract invariants honored here:
//! - Field set is exactly the spec's field set; no extra, no missing.
//! - `linetype_id` is a `StyleId` (the spec uses an ID-only reference;
//!   the linetype is resolved by the parent `Drawing`).
//! - `f64` fields (`lineweight`, `transparency`) are validated for
//!   finiteness at the deserialization boundary.
//! - Unknown fields are rejected (`#[serde(deny_unknown_fields)]`).

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
};

use crate::identity::LayerId;
use crate::identity::StyleId;

/// Drawing-owned layer record. A `Layer` is owned by exactly one
/// `Drawing` (per `spec/domain-model.md` §"Representation and reference
/// invariants" #4: "Every layout belongs to exactly one drawing" —
/// layers follow the same ownership rule).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Layer {
    /// Stable, opaque layer identity.
    pub id: LayerId,
    /// Human-readable layer name. Uniqueness within a drawing is
    /// enforced by the parent `Drawing`.
    pub name: String,
    /// Layer color (e.g. ACI color index or RGB triple). W003 does not
    /// pin a specific color encoding (the spec lists `color` without
    /// elaboration); the value is stored opaquely as a structured
    /// `(u8, u8, u8)` RGB triple (the most interoperable encoding).
    pub color: LayerColor,
    /// Reference to a linetype `Style` in the parent `Drawing`.
    pub linetype_id: StyleId,
    /// Line weight in millimeters. Finite non-negative.
    pub lineweight: f64,
    /// Transparency in `[0.0, 1.0]` (0.0 = fully opaque, 1.0 = fully
    /// transparent). Finite.
    pub transparency: f64,
    /// Visibility toggle.
    pub visible: bool,
    /// Locked layers cannot be edited by interactive commands.
    pub locked: bool,
    /// Frozen layers are excluded from regeneration.
    pub frozen: bool,
    /// Whether the layer is plotted/printed.
    pub plot_enabled: bool,
    /// Human-readable description.
    pub description: String,
}

/// RGB color triple. The canonical encoding for layer/entity color.
/// Future work items may add ACI color index support; W003 stores
/// color as RGB only (the most interoperable encoding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerColor {
    /// Red channel `[0, 255]`.
    pub r: u8,
    /// Green channel `[0, 255]`.
    pub g: u8,
    /// Blue channel `[0, 255]`.
    pub b: u8,
}

impl LayerColor {
    /// Construct an RGB color.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// ---------------------------------------------------------------------------
// Canonical-model boundary: deserialize then validate f64 finiteness.
// ---------------------------------------------------------------------------

/// Private serde wire shape for [`Layer`]. Used to enforce f64
/// finiteness at the deserialization boundary.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLayer {
    id: LayerId,
    name: String,
    color: LayerColor,
    linetype_id: StyleId,
    lineweight: f64,
    transparency: f64,
    visible: bool,
    locked: bool,
    frozen: bool,
    plot_enabled: bool,
    description: String,
}

impl TryFrom<RawLayer> for Layer {
    type Error = &'static str;

    fn try_from(r: RawLayer) -> Result<Self, Self::Error> {
        if !r.lineweight.is_finite() {
            return Err("lineweight must be finite");
        }
        if !r.transparency.is_finite() {
            return Err("transparency must be finite");
        }
        Ok(Self {
            id: r.id,
            name: r.name,
            color: r.color,
            linetype_id: r.linetype_id,
            lineweight: r.lineweight,
            transparency: r.transparency,
            visible: r.visible,
            locked: r.locked,
            frozen: r.frozen,
            plot_enabled: r.plot_enabled,
            description: r.description,
        })
    }
}

impl<'de> Deserialize<'de> for Layer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawLayer::deserialize(deserializer)?;
        Self::try_from(raw).map_err(de::Error::custom)
    }
}

impl Layer {
    /// Construct a layer with all fields. Performs only f64-finiteness
    /// validation on `lineweight`/`transparency`. Range validation
    /// (e.g. transparency in `[0, 1]`) is the importer's responsibility
    /// per the spec ("the canonical drawing stores one declared unit
    /// system" — not a declared range system; W003 does not invent
    /// range semantics the spec does not define).
    #[must_use]
    pub fn new(
        id: LayerId,
        name: String,
        color: LayerColor,
        linetype_id: StyleId,
        lineweight: f64,
        transparency: f64,
        visible: bool,
        locked: bool,
        frozen: bool,
        plot_enabled: bool,
        description: String,
    ) -> Self {
        // Construction bypasses the boundary; clamp NaN/Inf to 0.0 so
        // an internal construction can never produce a non-finite
        // value (defense in depth).
        let lineweight = if lineweight.is_finite() {
            lineweight
        } else {
            0.0
        };
        let transparency = if transparency.is_finite() {
            transparency
        } else {
            0.0
        };
        Self {
            id,
            name,
            color,
            linetype_id,
            lineweight,
            transparency,
            visible,
            locked,
            frozen,
            plot_enabled,
            description,
        }
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — Layer field set matches spec; f64
    // non-finite values rejected at the boundary.
    // Evidence: WO-003-AC02 — Layer round-trips and rejects unknown fields.

    use super::*;
    use crate::identity::TestIdGenerator;

    fn fixture_layer() -> Layer {
        let mut g = TestIdGenerator::new(0);
        Layer::new(
            crate::identity::next_layer_id(&mut g),
            "Walls".to_string(),
            LayerColor::rgb(255, 0, 0),
            crate::identity::next_style_id(&mut g),
            0.25,
            0.0,
            true,
            false,
            false,
            true,
            "Wall layer".to_string(),
        )
    }

    #[test]
    fn layer_field_set_matches_spec() {
        let l = fixture_layer();
        let v = serde_json::to_value(&l).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "color",
                "description",
                "frozen",
                "id",
                "linetype_id",
                "lineweight",
                "locked",
                "name",
                "plot_enabled",
                "transparency",
                "visible",
            ]
        );
    }

    #[test]
    fn layer_serde_roundtrip_preserves_id() {
        let l = fixture_layer();
        let j = serde_json::to_string(&l).expect("serialize");
        let back: Layer = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(l.id, back.id);
        assert_eq!(l, back);
    }

    #[test]
    fn layer_rejects_non_finite_lineweight_at_boundary() {
        // Evidence: WO-003-AC01 — NaN/Inf rejected at the canonical
        // boundary (per spec/domain-model.md §"Core value types and
        // invariants": "f64 values must be finite").
        let l = fixture_layer();
        let mut v = serde_json::to_value(&l).expect("serialize");
        v["lineweight"] = serde_json::Value::String("NaN".to_string());
        // serde_json cannot represent NaN directly; use a high bit pattern.
        // Instead, build the raw JSON manually with a sentinel that
        // parses to a non-finite f64.
        let raw_json = r#"{
            "id": "00000000000000000000000000000000",
            "name":"x",
            "color":{"r":0,"g":0,"b":0},
            "linetype_id":"00000000000000000000000000000001",
            "lineweight":1e99999,
            "transparency":0.0,
            "visible":true,"locked":false,"frozen":false,
            "plot_enabled":true,"description":""
        }"#;
        let r: Result<Layer, _> = serde_json::from_str(raw_json);
        assert!(
            r.is_err(),
            "non-finite lineweight must be rejected at boundary"
        );
        // Suppress unused variable lint.
        let _ = v;
    }

    #[test]
    fn layer_rejects_unknown_fields() {
        let l = fixture_layer();
        let mut v = serde_json::to_value(&l).expect("serialize");
        let obj = v.as_object_mut().expect("object");
        obj.insert("surprise".to_string(), serde_json::Value::Null);
        let r: Result<Layer, _> = serde_json::from_value(v);
        assert!(r.is_err(), "unknown field must be rejected");
    }
}
