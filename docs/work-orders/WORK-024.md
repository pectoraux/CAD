# WORK-024 — Tauri Desktop Shell and Canvas Host

Status: BLOCKED-BY-W007,W009
Architecture: v1.0 frozen
Dependencies: WORK-007, WORK-009
Checkpoint: CP3

## Objective
Create the professional desktop application shell and integrate the deterministic core.

## Acceptance criteria
- Tauri host launches reliably.
- React UI communicates through explicit IPC/service contracts.
- No UI component writes document state outside command service.
- Open/edit/save flows are instrumented and tested.
