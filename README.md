# AEC CAD OS

AI-native professional CAD platform, initially targeting electrical engineering CAD, with a generic 2D CAD core and progressive DWG/DXF interoperability.

## Architectural thesis

The product is **not** a generic AutoCAD clone. It is a semantic engineering system whose primary graphical view is CAD.

- The canonical internal model is authoritative.
- DWG/DXF are interoperability formats, not the domain model.
- Electrical engineering objects and relationships are first-class.
- AI proposes deterministic commands/transactions; the CAD kernel validates and commits them.
- Unsupported imported DWG objects are preserved when possible rather than silently discarded.
- GLM 5.3 is the implementation agent; architecture is frozen and governed through bounded Work Orders.

## Implementation governance

The implementation follows the reviewed WorkflowOS pattern:

`Frozen Architecture → Requirements → Dependency Graph → Work Item → Work Order → GLM 5.3 Execution → CI/Verification → Architect Review → Merge → Checkpoint`

No implementation agent may change frozen architecture, workflow authority, scope, or acceptance criteria. Architecture changes require an explicit Architecture Change Request and a new architecture version.

## First product boundary

V1 is a desktop-first professional 2D electrical CAD application with:

- high-frequency 2D drafting and modification;
- layers, blocks, text, dimensions, hatches, layouts and plotting;
- electrical components, terminals, wires, networks, signals and cross-references;
- catalogs and part assignments;
- wire numbering, component tagging, BOM and core reports;
- progressive DWG/DXF import, preservation and export;
- deterministic command/transaction API and an AI intent layer.

3D, BIM, full AutoCAD API compatibility, full DGN support and broad simulation are explicitly later phases.

## Repo map

- `spec/architecture.md` — frozen system architecture
- `spec/architecture-lock.md` — non-negotiable invariants
- `spec/domain-model.md` — canonical entities, fields and relationships
- `spec/commands.md` — deterministic command/transaction contract
- `spec/api.md` — application and AI-facing API contracts
- `spec/interoperability.md` — DWG/DXF compatibility strategy
- `spec/requirements.md` — requirements and acceptance criteria
- `spec/dependency-graph.md` — work dependency DAG and checkpoints
- `spec/work-items.md` — implementation backlog
- `docs/work-orders/` — GLM 5.3 implementation orders
- `docs/reviews/` — architect checkpoint protocol and review records
- `scripts/` — local/static governance helpers

## Status

**Architecture v1.0 — FROZEN FOR IMPLEMENTATION**

The repository is specification-first. Implementation starts at `WORK-001` after the baseline gate passes.
