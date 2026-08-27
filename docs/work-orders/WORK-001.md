# WORK-001 — Repository Foundation and Governance

Status: READY
Architecture: v1.0 frozen
Dependencies: none
Checkpoint: CP0

## Objective
Establish the Rust workspace, core module boundaries, deterministic test harness, spec governance and CI baseline.

## Acceptance criteria
- W001-AC01 Rust workspace compiles.
- W001-AC02 Module boundaries match `spec/architecture.md`.
- W001-AC03 `scripts/spec-gate.sh` passes in CI.
- W001-AC04 Baseline unit/integration test commands are deterministic.
- W001-AC05 No CAD domain implementation is introduced beyond scaffolding/contracts.

## Out of scope
Geometry algorithms, DWG/DXF parsing, UI implementation, electrical logic.

## Stop conditions
Frozen module boundaries cannot be implemented without change -> `ARCHITECTURE_CHANGE_REQUIRED`.

## Definition of done
CI green; architecture checks green; repository skeleton documented; no out-of-scope behavior.
