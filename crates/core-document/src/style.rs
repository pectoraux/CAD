//! `Style` and `DimensionStyle`.
//!
//! Per `spec/domain-model.md` §"Styles":
//! > `Style` and `DimensionStyle` are first-class drawing-owned
//! > objects. A style has a stable ID and version; entity references
//! > contain IDs only. Style mutation is a command and is revisioned.
//!
//! And per §"Document objects" / "Drawing":
//! ```text
//! Drawing {
//!   ...
//!   linetypes[], text_styles[], dimension_styles[],
//!   ...
//! }
//! ```
//!
//! The spec lists `linetypes[]` and `text_styles[]` as separate Drawing
//! fields but does NOT define a separate `Linetype` type. W003 reads
//! this as: both containers hold `Style` objects; the container is the
//! disambiguator (a `Style` stored in `linetypes` is a linetype; a
//! `Style` stored in `text_styles` is a text style). This is the most
//! spec-literal reading that does not invent a new type.
//!
//! Frozen-contract invariants honored here:
//! - `Style` has the stable ID and version per the spec.
//! - The spec does not enumerate the body fields of `Style` (font, size,
//!   pattern, etc.). W003 therefore stores the style body as an
//!   opaque `BTreeMap<String, String>` (sorted-key map, per the
//!   `RatingSet` precedent in `spec/domain-model.md` §"Closed value
//!   types"). Implementing specific body fields (font name, text
//!   height, linetype pattern, etc.) would invent unspecified
//!   semantics; the W003 work order forbids that. The body-map
//!   placeholder lets importers preserve style content opaquely
//!   (per WO-003-AC03 — opaque preservation) without dropping it.
//! - `DimensionStyle` follows the same pattern: stable ID + version +
//!   opaque sorted-key body.
//! - Unknown fields are rejected (`#[serde(deny_unknown_fields)]`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::identity::{DimensionStyleId, StyleId};

/// Drawing-owned style (a text style when stored in
/// `Drawing.text_styles`; a linetype when stored in
/// `Drawing.linetypes`). The body is preserved as an opaque sorted-key
/// map so importers can carry source-format-specific style content
/// without losing it (per WO-003-AC03).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// Stable, opaque style identity.
    pub id: StyleId,
    /// Monotonically increasing style version. Style mutation is a
    /// command (future W006) and is revisioned; the version number
    /// distinguishes revisions.
    pub version: u64,
    /// Optional human-readable name.
    pub name: String,
    /// Opaque sorted-key body. Keys are stable canonical names chosen
    /// by the importer (e.g. "font_name", "text_height",
    /// "linetype_pattern"); values are canonical string encodings.
    /// The body is preserved verbatim through round trips — no
    /// content is dropped (per WO-003-AC03).
    pub body: BTreeMap<String, String>,
}

/// Drawing-owned dimension style. Follows the same shape as [`Style`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionStyle {
    /// Stable, opaque dimension-style identity.
    pub id: DimensionStyleId,
    /// Monotonically increasing version.
    pub version: u64,
    /// Optional human-readable name.
    pub name: String,
    /// Opaque sorted-key body.
    pub body: BTreeMap<String, String>,
}

impl Style {
    /// Construct a style with the given id, version, name and body.
    #[must_use]
    pub fn new(id: StyleId, version: u64, name: String, body: BTreeMap<String, String>) -> Self {
        Self {
            id,
            version,
            name,
            body,
        }
    }
}

impl DimensionStyle {
    /// Construct a dimension style.
    #[must_use]
    pub fn new(
        id: DimensionStyleId,
        version: u64,
        name: String,
        body: BTreeMap<String, String>,
    ) -> Self {
        Self {
            id,
            version,
            name,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC02 — styles round-trip and the opaque body
    // preserves content.
    // Evidence: WO-003-AC03 — opaque style body survives round-trip.

    use super::*;
    use crate::identity::TestIdGenerator;

    #[test]
    fn style_field_set_matches_spec_contract() {
        let mut g = TestIdGenerator::new(0);
        let s = Style::new(
            crate::identity::next_style_id(&mut g),
            1,
            "Standard".to_string(),
            BTreeMap::from([
                ("font_name".to_string(), "Arial".to_string()),
                ("text_height".to_string(), "2.5".to_string()),
            ]),
        );
        let v = serde_json::to_value(&s).expect("serialize");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["body", "id", "name", "version"]);
        let back: Style = serde_json::from_value(v).expect("deserialize");
        assert_eq!(s, back);
    }

    #[test]
    fn style_body_preserved_verbatim_through_roundtrip() {
        // Evidence: WO-003-AC03 — opaque style body content survives.
        let mut g = TestIdGenerator::new(0);
        let body = BTreeMap::from([
            ("linetype_pattern".to_string(), "ACAD_ISO10W100".to_string()),
            ("scale".to_string(), "1.0".to_string()),
            ("oblique_angle".to_string(), "0.2617993".to_string()),
            // An unusual key that a future spec might not define —
            // must survive untouched.
            ("__source_private".to_string(), "DWG_2024".to_string()),
        ]);
        let s = Style::new(
            crate::identity::next_style_id(&mut g),
            5,
            "LinetypeA".to_string(),
            body.clone(),
        );
        let j = serde_json::to_string(&s).expect("serialize");
        let back: Style = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back.body, body, "body preserved verbatim");
    }

    #[test]
    fn dimension_style_roundtrips() {
        let mut g = TestIdGenerator::new(0);
        let d = DimensionStyle::new(
            crate::identity::next_dimension_style_id(&mut g),
            2,
            "ISO-25".to_string(),
            BTreeMap::from([
                ("arrow_size".to_string(), "2.5".to_string()),
                ("text_offset".to_string(), "1.0".to_string()),
            ]),
        );
        let j = serde_json::to_string(&d).expect("serialize");
        let back: DimensionStyle = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(d, back);
    }

    #[test]
    fn styles_reject_unknown_fields() {
        let mut g = TestIdGenerator::new(0);
        let s = Style::new(
            crate::identity::next_style_id(&mut g),
            1,
            "x".to_string(),
            BTreeMap::new(),
        );
        let mut v = serde_json::to_value(&s).expect("serialize");
        v.as_object_mut()
            .expect("object")
            .insert("surprise".to_string(), serde_json::Value::Null);
        assert!(serde_json::from_value::<Style>(v).is_err());
    }
}
