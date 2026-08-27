# WORK-002 — Geometry Primitives and Predicates

Status: BLOCKED-BY-W001
Architecture: v1.1 frozen
Dependencies: WORK-001
Checkpoint: CP1

## Objective
Implement the deterministic 2D geometry foundation required by the canonical CAD model.

## Scope
Point, vector, transform, line, segment, arc, circle, ellipse, polyline, spline representation; distance, projection, containment, intersection, bounding box and transform operations.

## Acceptance criteria
- WO-002-AC01 — Geometry types have stable semantics and serialization tests.
- WO-002-AC02 — Predicates are deterministic.
- WO-002-AC03 — Degenerate inputs are explicitly handled.
- WO-002-AC04 — Property tests cover symmetry/invariance properties where applicable.
- WO-002-AC05 — No document/electrical/UI/file-format dependencies enter geometry.

## Out of scope
Interactive snapping, selection, rendering, DWG/DXF, electrical semantics.

## Identity

- Work Order: `WORK-002`
- Architecture version: `v1.1`

## Allowed changes

crates/core-geometry. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: document model, commands, UI, persistence, interop, electrical. Do not edit another Work Order to make this implementation pass.

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
WORK-002 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
