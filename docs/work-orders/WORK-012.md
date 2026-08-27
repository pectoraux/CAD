# WORK-012 — DWG Core Entities and Opaque Preservation

Status: BLOCKED-BY-W011
Architecture: v1.0 frozen
Dependencies: WORK-011
Checkpoint: CP4

## Objective
Decode the P0 DWG entity/table/object profile into the canonical model and preserve unsupported objects when safe.

## Acceptance criteria
- LINE, LWPOLYLINE/POLYLINE, ARC, CIRCLE, ELLIPSE, SPLINE, TEXT, MTEXT, HATCH, DIMENSION, LEADER/MLEADER, BLOCK/INSERT/ATTRIB/ATTDEF, layers/styles/layout structures are covered per the compatibility profile.
- Object handles/ownership are preserved separately from internal IDs.
- Unsupported objects produce diagnostics and preservation records.
- Parser fuzzing shows no panics/unsafe behavior on malformed inputs.
