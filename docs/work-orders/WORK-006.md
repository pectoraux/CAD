# WORK-006 — Command and Transaction Engine

Status: BLOCKED-BY-W003
Architecture: v1.0 frozen
Dependencies: WORK-003
Checkpoint: CP2

## Objective
Implement the only authoritative document mutation path.

## Acceptance criteria
- Typed command envelope and result contract exist.
- Preconditions are enforced before mutation.
- Transactions are idempotent.
- Undo/redo representation is deterministic.
- Replay of identical valid transactions does not double-apply.
- UI and AI are unable to bypass command execution.
