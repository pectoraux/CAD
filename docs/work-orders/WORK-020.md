# WORK-020 — Tagging, Wire Numbering and Cross-reference Automation

Status: BLOCKED-BY-W018,W019
Architecture: v1.1 frozen
Dependencies: WORK-018, WORK-019
Checkpoint: CP5

## Objective
Implement deterministic project-wide automation for tags, wire numbers and cross-references.

## Acceptance criteria
- WO-020-AC01 — Same project state + same rules => same results.
- WO-020-AC02 — Existing manual/fixed values are treated according to explicit strategy, never overwritten silently.
- WO-020-AC03 — Cross-document relationships are traceable.
- WO-020-AC04 — Preview/dry-run is supported before project-wide mutation.

## Identity

- Work Order: `WORK-020`
- Architecture version: `v1.1`

## Allowed changes

crates/domain-electrical automation. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: new rule engine, reports, UI commit bypass. Do not edit another Work Order to make this implementation pass.

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
WORK-020 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
