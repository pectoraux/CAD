# WORK-014 — Interoperability Corpus and Round-trip Harness

Status: BLOCKED-BY-W010,W013
Architecture: v1.0 frozen
Dependencies: WORK-010, WORK-013
Checkpoint: CP4

## Objective
Create the reusable fixture, parser, renderer and round-trip verification harness.

## Acceptance criteria
- Corpus categorized by architecture/electrical/mechanical/general and legacy/modern complexity.
- Parser, semantic and visual regression reports are machine-readable.
- Round-trip failures identify exact fixture and degradation class.
- Unsupported content is counted and visible.
