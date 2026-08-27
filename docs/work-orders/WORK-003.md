# WORK-003 — Canonical Document Model

Status: BLOCKED-BY-W002
Architecture: v1.1 frozen
Dependencies: WORK-002
Checkpoint: CP1

## Objective
Implement the authoritative in-memory canonical document graph and entity/object identity model.

## Scope
Drawing, Entity, Layer, BlockDefinition, BlockReference, Layout, Viewport, styles, external-reference metadata, opaque external object preservation containers.

## Acceptance criteria
- WO-003-AC01 — All IDs/reference invariants are tested.
- WO-003-AC02 — Serialization round trips preserve identity and relationships.
- WO-003-AC03 — Unsupported opaque objects can survive load/save of the canonical representation.
- WO-003-AC04 — Generic CAD modules do not import electrical modules.
- WO-003-AC05 — No UI code owns document authority.

## Identity

- Work Order: `WORK-003`
- Architecture version: `v1.1`

## Allowed changes

crates/core-document. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: geometry algorithms beyond needed value-type integration, commands, UI, interop, electrical. Do not edit another Work Order to make this implementation pass.

## Required tests/evidence

- Unit tests for all new deterministic behavior.
- Property/fuzz tests for geometry or binary parsing where applicable.
- Integration tests for cross-module behavior where applicable.
- Regression fixtures for every discovered edge case.
- Architecture/static checks proving forbidden dependencies are absent.
- Evidence identifiers must map to this Work Order's acceptance criteria in `spec/traceability.md`.

## Scope boundary

Later Work Orders remain out of scope even when their code would be convenient to add now. A future-facing abstraction is allowed only when required by this Work Order and explicitly documented without implementing future behavior.

## Stop conditions

Stop and report `ARCHITECTURE_CHANGE_REQUIRED` for a frozen semantic gap, new authority, new command/entity/state, dependency-boundary change, proprietary hard dependency, or weakened correctness/data-loss guarantee. Report `IMPLEMENTATION_BLOCKED` when a prerequisite is missing or the repository baseline is inconsistent.

## Definition of done

All acceptance criteria pass with concrete evidence; no out-of-scope code exists; required checks are green; the branch is ready for independent Architect Review.

## Final response

```text
WORK-003 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
