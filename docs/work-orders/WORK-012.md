# WORK-012 — DWG Core Entities and Opaque Preservation

Status: BLOCKED-BY-W011
Architecture: v1.1 frozen
Dependencies: WORK-011
Checkpoint: CP4

## Objective
Decode the P0 DWG entity/table/object profile into the canonical model and preserve unsupported objects when safe.

## Acceptance criteria
- WO-012-AC01 — LINE, LWPOLYLINE/POLYLINE, ARC, CIRCLE, ELLIPSE, SPLINE, TEXT, MTEXT, HATCH, DIMENSION, LEADER/MLEADER, BLOCK/INSERT/ATTRIB/ATTDEF, layers/styles/layout structures are covered per the compatibility profile.
- WO-012-AC02 — Object handles/ownership are preserved separately from internal IDs.
- WO-012-AC03 — Unsupported objects produce diagnostics and preservation records.
- WO-012-AC04 — Parser fuzzing shows no panics/unsafe behavior on malformed inputs.

## Identity

- Work Order: `WORK-012`
- Architecture version: `v1.1`

## Allowed changes

crates/interop-dwg and mapping/preservation tests. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: writer certification, UI, electrical. Do not edit another Work Order to make this implementation pass.

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
WORK-012 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
