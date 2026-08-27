# WORK-026 — V1 Certification and Release Gate

Status: BLOCKED-BY-W014,W015,W021,W022,W023,W025
Architecture: v1.1 frozen
Dependencies: WORK-014, WORK-015, WORK-021, WORK-022, WORK-023, WORK-025
Checkpoint: CP7

## Objective
Certify the V1 product against the frozen requirements and checkpoints.

## Acceptance criteria

- WO-026-AC01 — All prior checkpoint gates are green.
- WO-026-AC02 — Every V1 requirement has objective evidence mapped in `spec/traceability.md`.
- WO-026-AC03 — No unresolved data-loss, non-determinism, authority-bypass, architecture-boundary or command-authority defect remains.
- WO-026-AC04 — The certified DWG/DXF profile and unsupported-feature matrix are published.
- WO-026-AC05 — Representative electrical workflows pass end-to-end without bypassing the deterministic command/validation core.

## Required evidence
- full unit/integration/regression suite;
- geometry property/fuzz suite;
- visual regression corpus;
- DWG/DXF interoperability corpus;
- representative electrical projects;
- command determinism suite;
- AI adversarial suite;
- performance benchmark;
- documented unsupported-feature matrix;
- architect approval.

## Stop conditions
Any unresolved data-loss, non-determinism, architecture-boundary, or command-authority defect blocks release.

## Identity

- Work Order: `WORK-026`
- Architecture version: `v1.1`

## Allowed changes

verification/certification tooling, test fixtures and release docs. Changes outside these areas require Architect approval before implementation.

## Required implementation

Implement only the behavior stated in this Work Order together with the referenced frozen contracts. Use existing public interfaces and preserve ownership boundaries. Do not invent unspecified semantics.

## Forbidden changes

Do not modify frozen specification authority or introduce: feature implementation, architecture/spec changes. Do not edit another Work Order to make this implementation pass.

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
WORK-026 COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
