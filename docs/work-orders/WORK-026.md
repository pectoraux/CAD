# WORK-026 — V1 Certification and Release Gate

Status: BLOCKED-BY-W014,W015,W021,W022,W023,W025
Architecture: v1.0 frozen
Dependencies: WORK-014, WORK-015, WORK-021, WORK-022, WORK-023, WORK-025
Checkpoint: CP7

## Objective
Certify the V1 product against the frozen requirements and checkpoints.

## Required evidence
- full unit/integration/regression suite;
- geometry property/fuzz suite;
- visual regression corpus;
- DWG/DXF interoperability corpus;
- representative electrical projects;
- command determinism suite;
- AI adversarial suite;
- performance benchmark;
- documented unsupported-feature matrix;
- architect approval.

## Stop conditions
Any unresolved data-loss, non-determinism, architecture-boundary, or command-authority defect blocks release.
