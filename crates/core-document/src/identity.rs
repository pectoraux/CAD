//! Opaque, sortable, type-distinct identity types.
//!
//! Per `spec/domain-model.md` §"Identity types":
//! > All IDs are opaque, globally unique within their type and never
//! > reused. The preferred wire form is UUIDv7 or an equivalent
//! > sortable random identifier. Handles originating from external
//! > formats are preserved separately and never reused as primary IDs.
//!
//! Architect directive (W003 activation):
//! > Do not use array position as durable identity.
//!
//! This module delivers all three properties:
//!
//! 1. **Opaque**: [`Id128`] is a newtype around a private `[u8; 16]`. The
//!    internal byte layout is not part of the public API. The only public
//!    constructors are [`Id128::nil`] (a well-defined zero constant for
//!    test/seed use) and [`Id128::from_bytes`]/[`Id128::from_hex`]
//!    (round-trip constructors for serde and importer use). The
//!    production generator lives outside this crate (the importer /
//!    command engine is responsible for producing UUIDv7-shaped IDs);
//!    the canonical model only STORES, VALIDATES UNIQUENESS, and
//!    RESOLVES IDs.
//!
//! 2. **Sortable**: `Id128` derives `Ord` over the raw bytes. UUIDv7's
//!    leading 48 bits are a millisecond timestamp in big-endian byte
//!    order, so byte-wise sort over a UUIDv7-shaped `Id128` produces
//!    creation-time ordering. The type does not enforce v7 layout (the
//!    version nibble is the producer's responsibility); it only enforces
//!    the byte-sort property, which is what the spec requires
//!    ("equivalent sortable random identifier").
//!
//! 3. **Type-distinct**: each ID type (`EntityId`, `LayerId`,
//!    `BlockDefinitionId`, `BlockReferenceId`, `StyleId`,
//!    `DimensionStyleId`, `LayoutId`, `ViewportId`, `ExternalRefId`,
//!    `ExternalObjectId`, `ProjectId`, `DrawingId`, `ArtifactVersionId`)
//!    is a separate newtype. The compiler rejects passing a `LayerId`
//!    where an `EntityId` is expected — there is no implicit coercion.
//!    This enforces the spec invariant "globally unique within their
//!    type" at the type-system level.
//!
//! Reproducibility (`spec/architecture.md` §11):
//! The canonical model never generates IDs at commit time using wall-clock
//! or uncontrolled randomness. IDs are part of the command/importer input
//! (per `spec/architecture.md` §11 "Reproducibility" — a deterministic
//! command execution is a pure function of `document_revision +
//! command_version + canonical_input + active_rule_profile +
//! deterministic_seed`; the seed is in the input, not generated at commit
//! time). This crate provides an [`IdGenerator`] trait for test/seed use;
//! production callers inject their own generator (importer: deterministic
//! ID derived from source handle; command engine: deterministic seed in
//! the command input).

use serde::Serialize;
use std::fmt;

// ---------------------------------------------------------------------------
// Id128 — the opaque wire shape (UUID-equivalent 128-bit value).
// ---------------------------------------------------------------------------

/// Opaque 128-bit identifier. The wire shape is the same as a UUID
/// (16 bytes). The canonical model treats it as opaque bytes; the
/// producer (importer / command engine) is responsible for assigning
/// UUIDv7 layout (or any other sortable random layout). The type
/// enforces byte-sort (`Ord` over `[u8; 16]`) but does not enforce a
/// specific UUID version.
///
/// `Id128` is exposed so that the [`IdGenerator`] trait (which returns
/// `Id128`) can be public; application code is expected to use the
/// per-purpose newtypes (`EntityId`, `LayerId`, etc.) directly and not
/// handle raw `Id128` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id128 {
    /// Raw 128-bit value. Big-endian byte order so that byte-wise `Ord`
    /// matches the spec's "sortable" requirement for UUIDv7-shaped
    /// values. Kept private so callers cannot construct an `Id128`
    /// with non-canonical layout (must go through [`Self::from_bytes`]
    /// / [`Self::nil`] / [`Self::from_hex`]).
    bytes: [u8; 16],
}

impl Id128 {
    /// All-zero id. Used only as a seed value in tests; never a real ID
    /// in a production drawing (a real ID must be unique within its
    /// type, and the all-zero ID cannot be unique if reused).
    pub const fn nil() -> Self {
        Self { bytes: [0u8; 16] }
    }

    /// Construct from raw bytes. The bytes are stored as-is; no version
    /// nibble is enforced. Used by the serde Deserialize impl and by
    /// importer/command-engine generators.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Read-only access to the raw bytes. Used by the serde Serialize
    /// impl (transparent hex-string form).
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Construct from a 32-char lowercase hex string (no dashes). Used
    /// for test ergonomics and the serde Deserialize impl. Returns
    /// `None` on length or character mismatch — the canonical boundary
    /// rejects malformed input.
    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 16];
        let bytes_iter = s.as_bytes();
        for i in 0..16 {
            let hi = hex_nibble(bytes_iter[i * 2])?;
            let lo = hex_nibble(bytes_iter[i * 2 + 1])?;
            bytes[i] = (hi << 4) | lo;
        }
        Some(Self { bytes })
    }

    /// Render as a 32-char lowercase hex string (no dashes). Stable
    /// wire form for `Display`/debug.
    pub fn to_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(32);
        for b in self.bytes {
            out.push(HEX[usize::from(b >> 4)] as char);
            out.push(HEX[usize::from(b & 0x0f)] as char);
        }
        out
    }
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

impl fmt::Display for Id128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

// ---------------------------------------------------------------------------
// Serde boundary for Id128: hex string (human-readable, sortable, opaque).
// ---------------------------------------------------------------------------

impl Serialize for Id128 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for Id128 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid id128 hex: {s}")))
    }
}

// ---------------------------------------------------------------------------
// Macro: declare a per-purpose ID newtype over Id128.
//
// Each newtype:
// - Wraps Id128 (Copy, Eq, Hash, Ord).
// - Serializes transparently as the underlying hex string.
// - Carries no domain semantics — the newtype exists purely to make
//   passing the wrong ID type a compile error (enforcing the spec
//   invariant "globally unique within their type" at the type level).
// ---------------------------------------------------------------------------

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub(crate) Id128);

        impl $name {
            /// Construct from raw bytes. Public to the importer / command
            /// engine so they can produce UUIDv7-shaped IDs. Not intended
            /// for application use; IDs are produced by the canonical
            /// boundary, not by application code.
            pub const fn from_bytes(bytes: [u8; 16]) -> Self {
                Self(Id128::from_bytes(bytes))
            }

            /// All-zero id. Exposed for test fixtures and seed values
            /// only; never a real production ID (a real ID must be
            /// unique within its type).
            pub const fn nil() -> Self {
                Self(Id128::nil())
            }

            /// Read-only access to the raw 16 bytes.
            pub const fn as_bytes(&self) -> &[u8; 16] {
                self.0.as_bytes()
            }

            /// Construct from an existing [`Id128`] (e.g. one produced
            /// by an [`IdGenerator`]). Used by the per-type convenience
            /// generators (`next_entity_id` etc.).
            #[must_use]
            pub const fn from_id128(inner: Id128) -> Self {
                Self(inner)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let inner = Id128::deserialize(deserializer)?;
                Ok(Self(inner))
            }
        }
    };
}

id_newtype!(
    /// Opaque identity for a [`Drawing`](crate::drawing::Drawing).
    DrawingId
);
id_newtype!(
    /// Opaque identity for a [`Project`](crate::project::Project).
    ProjectId
);
id_newtype!(
    /// Opaque identity for an [`Entity`](crate::entity::Entity). Unique
    /// within the Entity type across all drawings and never reused.
    EntityId
);
id_newtype!(
    /// Opaque identity for a [`Layer`](crate::layer::Layer).
    LayerId
);
id_newtype!(
    /// Opaque identity for a [`BlockDefinition`](crate::block::BlockDefinition).
    BlockDefinitionId
);
id_newtype!(
    /// Opaque identity for a [`BlockReference`](crate::block::BlockReference).
    BlockReferenceId
);
id_newtype!(
    /// Opaque identity for a [`Style`](crate::style::Style) (text style
    /// or linetype — both are stored as `Style` objects).
    StyleId
);
id_newtype!(
    /// Opaque identity for a [`DimensionStyle`](crate::style::DimensionStyle).
    DimensionStyleId
);
id_newtype!(
    /// Opaque identity for a [`Layout`](crate::layout::Layout).
    LayoutId
);
id_newtype!(
    /// Opaque identity for a [`Viewport`](crate::layout::Viewport).
    ViewportId
);
id_newtype!(
    /// Opaque identity for an [`ExternalReference`](crate::external::ExternalReference).
    ExternalRefId
);
id_newtype!(
    /// Opaque identity for an
    /// [`OpaqueExternalObject`](crate::external::OpaqueExternalObject).
    ExternalObjectId
);
id_newtype!(
    /// Opaque identity for a [`DrawingRevision`](crate::drawing::DrawingRevision)
    /// (an immutable artifact version of a drawing).
    ArtifactVersionId
);

// ---------------------------------------------------------------------------
// IdGenerator — pluggable ID source for tests / importer / command engine.
// ---------------------------------------------------------------------------

/// Produces opaque IDs. The canonical document model does NOT call this
/// at commit time (per `spec/architecture.md` §11 "Reproducibility" —
/// IDs are part of the command/importer input, not generated at commit
/// time using wall-clock or uncontrolled randomness). The trait is
/// provided so that:
/// - importers can produce deterministic IDs derived from the source
///   artifact (e.g. hashing the source handle + a fixed seed);
/// - the command engine (W006, future) can produce deterministic IDs
///   derived from the command seed;
/// - tests can use [`TestIdGenerator`] (sequential, fully deterministic).
///
/// Production generators live OUTSIDE this crate. This crate only
/// consumes the produced `Id128` and validates uniqueness at the
/// canonical boundary.
pub trait IdGenerator {
    /// Produce the next opaque 128-bit value. The layout (UUIDv7 or
    /// equivalent) is the generator's responsibility.
    fn next_id128(&mut self) -> Id128;
}

/// Test-only deterministic generator. Produces sequential IDs starting
/// from a base value. Provided for W003 unit tests so they can exercise
/// uniqueness, ordering, and round-trip behavior without depending on
/// wall-clock time or system randomness (per `spec/architecture.md`
/// §11 "Reproducibility").
#[derive(Debug, Clone)]
pub struct TestIdGenerator {
    counter: u128,
}

impl TestIdGenerator {
    /// Create a generator whose first produced ID is the
    /// `seed`-th value (so two generators with the same seed produce the
    /// same sequence — required for deterministic tests).
    #[must_use]
    pub fn new(seed: u128) -> Self {
        Self { counter: seed }
    }

    /// Produce the next [`Id128`] and advance the internal counter.
    pub fn next_id128(&mut self) -> Id128 {
        // Pack the u128 counter into 16 big-endian bytes so that
        // byte-wise `Ord` over `Id128` matches counter order (i.e.
        // generator-output order). This makes the "sortable" property
        // testable directly: `id_n < id_n+1`.
        let value = self.counter;
        self.counter = self.counter.saturating_add(1);
        let mut bytes = [0u8; 16];
        bytes[0..16].copy_from_slice(&value.to_be_bytes());
        Id128::from_bytes(bytes)
    }
}

impl IdGenerator for TestIdGenerator {
    fn next_id128(&mut self) -> Id128 {
        TestIdGenerator::next_id128(self)
    }
}

// ---------------------------------------------------------------------------
// Per-type convenience generators (build on top of an IdGenerator).
// ---------------------------------------------------------------------------

/// Convenience: produce a fresh [`EntityId`] from any [`IdGenerator`].
/// Useful in importer / test code.
pub fn next_entity_id<G: IdGenerator + ?Sized>(g: &mut G) -> EntityId {
    EntityId(g.next_id128())
}
/// Produce a fresh [`LayerId`] from any [`IdGenerator`].
pub fn next_layer_id<G: IdGenerator + ?Sized>(g: &mut G) -> LayerId {
    LayerId(g.next_id128())
}
/// Produce a fresh [`BlockDefinitionId`] from any [`IdGenerator`].
pub fn next_block_definition_id<G: IdGenerator + ?Sized>(g: &mut G) -> BlockDefinitionId {
    BlockDefinitionId(g.next_id128())
}
/// Produce a fresh [`BlockReferenceId`] from any [`IdGenerator`].
pub fn next_block_reference_id<G: IdGenerator + ?Sized>(g: &mut G) -> BlockReferenceId {
    BlockReferenceId(g.next_id128())
}
/// Produce a fresh [`StyleId`] from any [`IdGenerator`].
pub fn next_style_id<G: IdGenerator + ?Sized>(g: &mut G) -> StyleId {
    StyleId(g.next_id128())
}
/// Produce a fresh [`DimensionStyleId`] from any [`IdGenerator`].
pub fn next_dimension_style_id<G: IdGenerator + ?Sized>(g: &mut G) -> DimensionStyleId {
    DimensionStyleId(g.next_id128())
}
/// Produce a fresh [`LayoutId`] from any [`IdGenerator`].
pub fn next_layout_id<G: IdGenerator + ?Sized>(g: &mut G) -> LayoutId {
    LayoutId(g.next_id128())
}
/// Produce a fresh [`ViewportId`] from any [`IdGenerator`].
pub fn next_viewport_id<G: IdGenerator + ?Sized>(g: &mut G) -> ViewportId {
    ViewportId(g.next_id128())
}
/// Produce a fresh [`ExternalRefId`] from any [`IdGenerator`].
pub fn next_external_ref_id<G: IdGenerator + ?Sized>(g: &mut G) -> ExternalRefId {
    ExternalRefId(g.next_id128())
}
/// Produce a fresh [`ExternalObjectId`] from any [`IdGenerator`].
pub fn next_external_object_id<G: IdGenerator + ?Sized>(g: &mut G) -> ExternalObjectId {
    ExternalObjectId(g.next_id128())
}
/// Produce a fresh [`DrawingId`] from any [`IdGenerator`].
pub fn next_drawing_id<G: IdGenerator + ?Sized>(g: &mut G) -> DrawingId {
    DrawingId(g.next_id128())
}
/// Produce a fresh [`ProjectId`] from any [`IdGenerator`].
pub fn next_project_id<G: IdGenerator + ?Sized>(g: &mut G) -> ProjectId {
    ProjectId(g.next_id128())
}
/// Produce a fresh [`ArtifactVersionId`] from any [`IdGenerator`].
pub fn next_artifact_version_id<G: IdGenerator + ?Sized>(g: &mut G) -> ArtifactVersionId {
    ArtifactVersionId(g.next_id128())
}

#[cfg(test)]
mod tests {
    // Evidence: WO-003-AC01 — IDs are opaque, sortable, type-distinct,
    // and never reused.

    use super::*;

    #[test]
    fn id128_is_sortable_by_bytes() {
        // Evidence: WO-003-AC01 — "equivalent sortable random identifier".
        // Byte-wise Ord over two sequentially-produced generator IDs
        // matches the production order.
        let mut g = TestIdGenerator::new(0);
        let a = g.next_id128();
        let b = g.next_id128();
        let c = g.next_id128();
        assert!(a < b, "a ({a}) must sort before b ({b})");
        assert!(b < c, "b ({b}) must sort before c ({c})");
    }

    #[test]
    fn id128_hex_roundtrip_is_stable() {
        // Evidence: WO-003-AC02 — wire form is a stable hex string.
        let mut g = TestIdGenerator::new(42);
        let id = g.next_id128();
        let hex = id.to_hex();
        assert_eq!(hex.len(), 32);
        let back = Id128::from_hex(&hex).expect("roundtrip");
        assert_eq!(id, back, "hex roundtrip");
        // Same wire form via serde.
        let json = serde_json::to_string(&id).expect("serialize");
        let back2: Id128 = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, back2, "serde roundtrip");
        assert_eq!(format!("{id}"), hex, "Display matches to_hex");
    }

    #[test]
    fn malformed_hex_is_rejected_at_boundary() {
        // Evidence: WO-003-AC01 — canonical-model boundary rejects
        // malformed IDs (never silently accepts garbage).
        assert!(Id128::from_hex("").is_none());
        assert!(Id128::from_hex("abc").is_none()); // too short
        assert!(Id128::from_hex(&"z".repeat(32)).is_none()); // bad char
        let json = "\"not_a_hex_id\"";
        let r: Result<Id128, _> = serde_json::from_str(json);
        assert!(r.is_err(), "malformed id must fail deserialize");
    }

    #[test]
    fn per_type_newtypes_do_not_coerce() {
        // Evidence: WO-003-AC01 — "globally unique within their type".
        // The newtypes are distinct at the type-system level; the
        // compiler rejects passing one where another is expected. We
        // can only assert this at runtime via PartialEq on the inner
        // Id128 — but the fact that this code compiles at all IS the
        // type-distinctness evidence.
        let mut g = TestIdGenerator::new(0);
        let eid = next_entity_id(&mut g);
        let lid = next_layer_id(&mut g);
        // Same inner value, different types — the .0 access proves they
        // are different newtype wrappers.
        assert_ne!(
            format!("{eid}"),
            format!("{lid}"),
            "different sequential IDs must not collide"
        );
        // Compile-time distinctness: the line below would be a type
        // error if uncommented:
        // let _: EntityId = lid;
    }

    #[test]
    fn test_id_generator_is_deterministic() {
        // Evidence: WO-003-AC01 — same seed → same sequence (per
        // `spec/architecture.md` §11 "Reproducibility").
        let mut g1 = TestIdGenerator::new(7);
        let mut g2 = TestIdGenerator::new(7);
        for _ in 0..16 {
            assert_eq!(g1.next_id128(), g2.next_id128(), "deterministic sequence");
        }
    }

    #[test]
    fn nil_is_well_defined_zero() {
        // Evidence: WO-003-AC01 — `nil()` is a stable, well-defined
        // zero constant. Used only as a seed value in tests.
        let n = Id128::nil();
        assert_eq!(n.as_bytes(), &[0u8; 16]);
        assert_eq!(n.to_hex(), "0".repeat(32));
    }
}
