# WORK-003 — Canonical Document Model

Status: BLOCKED-BY-W002
Architecture: v1.0 frozen
Dependencies: WORK-002
Checkpoint: CP1

## Objective
Implement the authoritative in-memory canonical document graph and entity/object identity model.

## Scope
Drawing, Entity, Layer, BlockDefinition, BlockReference, Layout, Viewport, styles, external-reference metadata, opaque external object preservation containers.

## Acceptance criteria
- All IDs/reference invariants are tested.
- Serialization round trips preserve identity and relationships.
- Unsupported opaque objects can survive load/save of the canonical representation.
- Generic CAD modules do not import electrical modules.
- No UI code owns document authority.
