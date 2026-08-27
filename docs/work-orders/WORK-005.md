# WORK-005 — Snap and Precision Engine

Status: BLOCKED-BY-W004
Architecture: v1.0 frozen
Dependencies: WORK-004
Checkpoint: CP2

## Objective
Implement deterministic snapping and precision input for professional 2D drafting.

## Scope
Endpoint, midpoint, center, intersection, nearest, perpendicular, tangent, grid, ortho and polar tracking.

## Acceptance criteria
- Candidate generation is deterministic.
- Snap priority and tie-breaking are documented and tested.
- Cursor-path performance meets benchmark.
- Snapping never mutates the document.
