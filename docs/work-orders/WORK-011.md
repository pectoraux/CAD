# WORK-011 — DWG Header and Section Reader

Status: BLOCKED-BY-W003
Architecture: v1.0 frozen
Dependencies: WORK-003
Checkpoint: CP4

## Objective
Begin independent DWG reader implementation: version detection, header, section directory/records and foundational binary primitives.

## Acceptance criteria
- Supported versions are explicitly enumerated.
- Parser rejects malformed/truncated inputs safely.
- Golden fixtures cover headers and section metadata.
- No claims of entity-level compatibility are made yet.
