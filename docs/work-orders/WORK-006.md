# WORK-006 — Command and Transaction Engine

Status: BLOCKED-BY-W003
Architecture: v1.1 frozen
Dependencies: WORK-003
Checkpoint: CP2

## Objective
Implement the only authoritative document mutation path.

## Acceptance criteria
- WO-006-AC01 — Typed command envelope and result contract exist.
- WO-006-AC02 — Preconditions are enforced before mutation.
- WO-006-AC03 — Transactions are idempotent.
- WO-006-AC04 — Undo/redo representation is deterministic.
- WO-006-AC05 — Replay of identical valid transactions does not double-apply.
- WO-006-AC06 — UI and AI are unable to bypass command execution.

## Identity

- Work Order: `WORK-006`
- Architecture version: `v1.1`

## Allowed changes

crates/core-commands. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: UI, import/export, electrical, direct persistence. Do not edit another Work Order to make this implementation pass.

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
WORK-006 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
