# WORK-021 — BOM and Report Engine

Status: BLOCKED-BY-W018,W019,W020
Architecture: v1.1 frozen
Dependencies: WORK-018, WORK-019, WORK-020
Checkpoint: CP5

## Objective
Implement query-driven reports and BOM generation over the engineering graph.

## Acceptance criteria
- WO-021-AC01 — Reports are declarative query definitions.
- WO-021-AC02 — BOM is reproducible from the canonical graph.
- WO-021-AC03 — Report output is deterministic.
- WO-021-AC04 — Generated drawing/table views are traceable to source entities.

## Identity

- Work Order: `WORK-021`
- Architecture version: `v1.1`

## Allowed changes

crates/domain-electrical reporting. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: new electrical semantics not in model, UI mutation, AI. Do not edit another Work Order to make this implementation pass.

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
WORK-021 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
