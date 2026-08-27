# WORK-004 — Spatial Index and Selection

Status: BLOCKED-BY-W003
Architecture: v1.0 frozen
Dependencies: WORK-003
Checkpoint: CP2

## Objective
Implement scalable hit-testing, window selection and selection-set semantics.

## Acceptance criteria
- Point/window selection is deterministic.
- Spatial index updates are transactionally synchronized with document changes.
- Hit-test results are stable under identical state/input.
- Reference benchmark demonstrates no O(N) pointer-move scan for ordinary view interactions.
