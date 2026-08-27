# WORK-019 — Manufacturer Catalog and Part Assignment

Status: BLOCKED-BY-W017
Architecture: v1.1 frozen
Dependencies: WORK-017
Checkpoint: CP5

## Objective
Implement manufacturer/product/catalog abstractions and deterministic part assignment.

## Acceptance criteria
- WO-019-AC01 — Catalog is separate from geometry.
- WO-019-AC02 — Manufacturer/part identity is stable.
- WO-019-AC03 — Component assignments are traceable.
- WO-019-AC04 — External/imported catalog records produce explicit provenance.

## Identity

- Work Order: `WORK-019`
- Architecture version: `v1.1`

## Allowed changes

crates/domain-electrical catalog/part data. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: geometry engine, wire algorithms, UI catalog shell. Do not edit another Work Order to make this implementation pass.

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
WORK-019 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
