# WORK-023 — Typed AI Planning Gateway

Status: BLOCKED-BY-W006,W016
Architecture: v1.0 frozen
Dependencies: WORK-006, WORK-016
Checkpoint: CP6

## Objective
Allow GLM 5.3 or another provider to translate user intents into typed command plans without direct mutation authority.

## Acceptance criteria
- Provider-neutral gateway interface exists.
- Model output is validated against a typed schema.
- Invalid/ambiguous/stale plans are rejected.
- AI cannot access database/filesystem mutation APIs.
- Applied plans retain provenance and command-level explanations.
- Adversarial tests prove model text cannot bypass command validation.
