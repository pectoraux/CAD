# WORK-001 — Repository Foundation and Governance

Status: READY
Architecture: v1.1 frozen
Dependencies: none
Checkpoint: CP0

## Objective
Establish the Rust workspace, core module boundaries, deterministic test harness, spec governance and CI baseline.

## Acceptance criteria
- WO-001-AC01 — Rust workspace compiles.
- WO-001-AC02 — Module boundaries match `spec/architecture.md`.
- WO-001-AC03 — `scripts/spec-gate.sh` passes in CI.
- WO-001-AC04 — Baseline unit/integration test commands are deterministic.
- WO-001-AC05 — No CAD domain implementation is introduced beyond scaffolding/contracts.

## Out of scope
Geometry algorithms, DWG/DXF parsing, UI implementation, electrical logic.

## Stop conditions
Frozen module boundaries cannot be implemented without change -> `ARCHITECTURE_CHANGE_REQUIRED`.

## Definition of done
CI green; architecture checks green; repository skeleton documented; the frozen-spec gate and Work Order schema gate run in CI; no out-of-scope behavior.

## Identity

- Work Order: `WORK-001`
- Architecture version: `v1.1`

## Allowed changes

crates/*, app-shell, app-ui, scripts, CI scaffolding. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: any domain behavior, spec/, electrical semantics. Do not edit another Work Order to make this implementation pass.

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
WORK-001 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
