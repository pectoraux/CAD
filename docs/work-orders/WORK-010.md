# WORK-010 — DXF Codec

Status: BLOCKED-BY-W003
Architecture: v1.0 frozen
Dependencies: WORK-003
Checkpoint: CP3

## Objective
Implement an independent DXF reader/writer mapped through the canonical model.

## Acceptance criteria
- P0 entity/table inventory supported.
- Unsupported constructs produce diagnostics.
- Canonical -> DXF -> canonical round trips preserve tested semantics.
- Format code is isolated in `interop-dxf`.
