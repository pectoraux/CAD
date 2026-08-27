# Dependency Graph v1.1

The Work Items form a directed acyclic graph. A Work Item is eligible only when every listed dependency is VERIFIED and its parent checkpoint permits execution.

## Canonical dependency table

| Work Item | Dependencies | Checkpoint |
|---|---|---|
| W001 | — | CP0 |
| W002 | W001 | CP1 |
| W003 | W002 | CP1 |
| W004 | W003 | CP2 |
| W005 | W004 | CP2 |
| W006 | W003 | CP2 |
| W007 | W005,W006 | CP2 |
| W008 | W003,W006 | CP3 |
| W009 | W008 | CP3 |
| W010 | W003 | CP3 |
| W011 | W003 | CP4 |
| W012 | W011 | CP4 |
| W015 | W007,W010,W012 | CP4 |
| W013 | W012,W015 | CP4 |
| W014 | W010,W013 | CP4 |
| W016 | W003,W006 | CP5 |
| W017 | W016,W008 | CP5 |
| W018 | W017,W007 | CP5 |
| W019 | W017 | CP5 |
| W020 | W018,W019 | CP5 |
| W021 | W018,W019,W020 | CP5 |
| W022 | W018,W019 | CP5 |
| W023 | W006,W020,W022 | CP6 |
| W024 | W007,W009 | CP3 |
| W025 | W020,W021,W022,W023,W024 | CP5 |
| W026 | W014,W015,W021,W022,W023,W025 | CP7 |

The semantic order around interoperability is `W012 -> W015 -> W013 -> W014`; this ensures writer certification includes production block/Xref semantics and the round-trip harness tests the certified writer profile.

## Checkpoints

- CP0: repository/spec baseline green before implementation.
- CP1: deterministic geometry and canonical document model validated.
- CP2: selection, snapping, commands and undo/redo validated.
- CP3: annotation/layout/plot and desktop host validated.
- CP4: DWG/DXF read/preserve/write profile and block/Xref semantics validated.
- CP5: electrical graph, catalog, automation, reporting, validation and UX validated.
- CP6: AI typed planning and adversarial execution boundary validated.
- CP7: full V1 certification.

Checkpoints are hard barriers. Downstream work is not eligible merely because source code exists; dependencies and prior checkpoint evidence must be VERIFIED.
