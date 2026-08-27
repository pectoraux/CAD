# WORK-015 — Blocks, Attributes and Xrefs

Status: BLOCKED-BY-W007,W010
Architecture: v1.0 frozen
Dependencies: WORK-007, WORK-010
Checkpoint: CP4

## Objective
Make reusable content and external references production-grade.

## Acceptance criteria
- Block definitions/references remain identity-linked.
- Attribute values survive round-trip.
- Xref references have explicit status and transforms.
- Explode does not silently destroy attribute semantics without explicit command behavior.
