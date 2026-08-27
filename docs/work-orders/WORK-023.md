# WORK-023 — Typed AI Planning Gateway

Status: BLOCKED-BY-W006,W016
Architecture: v1.1 frozen
Dependencies: WORK-006, WORK-016
Checkpoint: CP6

## Objective
Allow GLM 5.3 or another provider to translate user intents into typed command plans without direct mutation authority.

## Acceptance criteria
- WO-023-AC01 — Provider-neutral gateway interface exists.
- WO-023-AC02 — Model output is validated against a typed schema.
- WO-023-AC03 — Invalid/ambiguous/stale plans are rejected.
- WO-023-AC04 — AI cannot access database/filesystem mutation APIs.
- WO-023-AC05 — Applied plans retain provenance and command-level explanations.
- WO-023-AC06 — Adversarial tests prove model text cannot bypass command validation.

## Identity

- Work Order: `WORK-023`
- Architecture version: `v1.1`

## Allowed changes

crates/ai-gateway and adversarial tests. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: command engine changes, direct mutation, provider-specific domain coupling. Do not edit another Work Order to make this implementation pass.

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
WORK-023 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
