# WORK-002 — Geometry Primitives and Predicates

Status: BLOCKED-BY-W001
Architecture: v1.0 frozen
Dependencies: WORK-001
Checkpoint: CP1

## Objective
Implement the deterministic 2D geometry foundation required by the canonical CAD model.

## Scope
Point, vector, transform, line, segment, arc, circle, ellipse, polyline, spline representation; distance, projection, containment, intersection, bounding box and transform operations.

## Acceptance criteria
- Geometry types have stable semantics and serialization tests.
- Predicates are deterministic.
- Degenerate inputs are explicitly handled.
- Property tests cover symmetry/invariance properties where applicable.
- No document/electrical/UI/file-format dependencies enter geometry.

## Out of scope
Interactive snapping, selection, rendering, DWG/DXF, electrical semantics.
