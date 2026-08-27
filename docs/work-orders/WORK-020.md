# WORK-020 — Tagging, Wire Numbering and Cross-reference Automation

Status: BLOCKED-BY-W018,W019
Architecture: v1.0 frozen
Dependencies: WORK-018, WORK-019
Checkpoint: CP5

## Objective
Implement deterministic project-wide automation for tags, wire numbers and cross-references.

## Acceptance criteria
- Same project state + same rules => same results.
- Existing manual/fixed values are treated according to explicit strategy, never overwritten silently.
- Cross-document relationships are traceable.
- Preview/dry-run is supported before project-wide mutation.
