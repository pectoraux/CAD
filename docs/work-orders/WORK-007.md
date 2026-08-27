# WORK-007 — P0 Modify Commands

Status: BLOCKED-BY-W005,W006
Architecture: v1.0 frozen
Dependencies: WORK-005, WORK-006
Checkpoint: CP2

## Objective
Implement the highest-frequency CAD mutation loop.

## Commands
Delete, Move, Copy, Trim, Extend, Offset, Stretch, Rotate, Mirror, Scale, Fillet, Chamfer, Break, Join, Explode and basic entity creation.

## Acceptance criteria
- Each command has deterministic unit/property tests.
- Each command produces correct affected-ID sets.
- Undo/redo restores equivalent canonical state.
- Degenerate/invalid operations are rejected explicitly.
- Representative visual fixtures pass.
