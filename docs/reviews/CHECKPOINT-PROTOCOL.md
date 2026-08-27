# Architectural Checkpoint Protocol

A checkpoint is a hard barrier, not a progress update.

## CP0 — Specification integrity

Required:

- repository clean baseline;
- architecture lock present;
- dependency DAG valid;
- Work Items all linked to requirements;
- Work Orders generated for next eligible items;
- governance checks green.

## CP1 — Canonical model integrity

Required:

- geometry/document model tests;
- ID/reference invariants;
- serialization round trips;
- no UI ownership of domain state.

## CP2 — Deterministic drafting loop

Required:

- selection/snaps;
- P0 editing commands;
- undo/redo;
- command determinism tests;
- interaction latency benchmark.

## CP3 — Production drawing

Required:

- annotation;
- blocks;
- layouts/viewports;
- plot/PDF;
- representative drawing regression.

## CP4 — Interoperability

Required:

- DWG/DXF corpus;
- read/preserve/write certification profile;
- round-trip tests;
- visual comparisons;
- explicit diagnostics for degraded objects.

## CP5 — Electrical engineering integrity

Required:

- component graph;
- terminals/wires/networks;
- catalog;
- tag/number automation;
- reports/BOM;
- validation engine;
- cross-view representation integrity.

## CP6 — AI safety and utility

Required:

- typed plan schema;
- adversarial plan rejection;
- deterministic execution;
- no direct mutation authority;
- task-level quality benchmark.

## CP7 — V1 release

All prior gates green, end-to-end electrical workflows certified, supported interoperability profile documented, known limitations published.
