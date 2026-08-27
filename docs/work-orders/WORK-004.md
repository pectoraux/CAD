# WORK-004 — Spatial Index and Selection

Status: BLOCKED-BY-W003
Architecture: v1.1 frozen
Dependencies: WORK-003
Checkpoint: CP2

## Objective
Implement scalable hit-testing, window selection and selection-set semantics.

## Acceptance criteria
- WO-004-AC01 — Point/window selection is deterministic.
- WO-004-AC02 — Spatial index updates are transactionally synchronized with document changes.
- WO-004-AC03 — Hit-test results are stable under identical state/input.
- WO-004-AC04 — Reference benchmark demonstrates no O(N) pointer-move scan for ordinary view interactions.

## Identity

- Work Order: `WORK-004`
- Architecture version: `v1.1`

## Allowed changes

crates/core-selection. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: snap, commands, UI mutation, interop, electrical. Do not edit another Work Order to make this implementation pass.

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
WORK-004 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
