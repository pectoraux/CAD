# WORK-013 — DWG Writer Compatibility Profile A

Status: BLOCKED-BY-W012
Architecture: v1.0 frozen
Dependencies: WORK-012
Checkpoint: CP4

## Objective
Implement a certified first DWG writer profile only for the subset proven by the corpus.

## Acceptance criteria
- No version is advertised without fixtures.
- Exported files reopen in at least one independent DWG-capable implementation during verification where legally/practically available.
- Audit/reopen tests pass for certified fixtures.
- Diagnostics prevent silent loss.
