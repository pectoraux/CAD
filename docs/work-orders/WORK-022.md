# WORK-022 — Electrical Rules and Validation

Status: BLOCKED-BY-W018,W019
Architecture: v1.0 frozen
Dependencies: WORK-018, WORK-019
Checkpoint: CP5

## Objective
Implement deterministic design checks and diagnostics.

## Minimum rules

- dangling terminal;
- disconnected wire endpoint;
- duplicate/invalid tag;
- duplicate wire number where prohibited;
- missing catalog assignment where required;
- invalid component connection;
- missing required representation;
- report inconsistency.

## Acceptance criteria
Every diagnostic includes severity, stable code, affected IDs and human-readable explanation.
