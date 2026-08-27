# WORK-016 — Electrical Project Domain

Status: BLOCKED-BY-W003,W006
Architecture: v1.0 frozen
Dependencies: WORK-003, WORK-006
Checkpoint: CP5

## Objective
Introduce the electrical project aggregate without contaminating the generic CAD core.

## Acceptance criteria
- Electrical project has explicit standard/profile configuration.
- Project-to-drawing membership is persisted.
- Generic CAD can exist without electrical semantics.
- Electrical domain can reference CAD entities through stable IDs.
