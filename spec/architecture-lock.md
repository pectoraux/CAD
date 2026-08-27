# Architecture Lock v1.0

**Status: FROZEN**

This document is authoritative. Any change to a frozen rule requires an Architecture Change Request and a new architecture version.

## Non-negotiable principles

1. **Canonical model authority** — the internal AEC CAD document model is the application source of truth. DWG/DXF/PDF are external representations.
2. **Deterministic CAD core** — geometry, snapping, modification, document semantics, layout and validation are deterministic and do not depend on an LLM.
3. **AI is a planner, never the authority** — GLM or another model may propose intents/plans, but only validated commands/transactions can mutate the document.
4. **Explicit command transactions** — every mutation has a deterministic command identity, input contract, preconditions, effects, and inverse/undo representation where applicable.
5. **Unknown-object preservation** — unsupported imported external objects must be preserved as opaque payload/proxy objects where the source format permits safe preservation.
6. **DWG compatibility is progressive** — reader, preservation, writer and round-trip compatibility are separate measurable capabilities.
7. **No silent data loss** — import/export must produce explicit diagnostics for unsupported or degraded content.
8. **Domain separation** — generic CAD primitives do not depend on electrical domain modules; electrical semantics depend on generic CAD but not vice versa.
9. **No vendor lock-in** — no required proprietary CAD SDK is part of the architecture. ODA is not a dependency.
10. **Provider-independent AI boundary** — model/provider details remain behind an AI gateway. GLM 5.3 is the current implementation provider/agent, not a domain dependency.
11. **Desktop-first** — native desktop is the authoritative professional client. Web/WASM is a later client of the same core.
12. **Rust core** — geometry/document/interoperability/command core is Rust-first. TypeScript/React is application UI; Tauri is the desktop shell.
13. **No direct cross-module internals** — modules communicate through explicit public contracts.
14. **Evidence over claims** — no Work Item is complete without objective automated or manually recorded evidence tied to acceptance criteria.
15. **Scope integrity** — a Work Order must not implement later work. Architectural uncertainty triggers escalation rather than speculative design drift.

## Technology lock

- Core: Rust stable
- Desktop shell: Tauri
- UI: React + TypeScript
- Storage: PostgreSQL for cloud/project metadata; local project cache is file/database based but must not replace the canonical cloud system of record once cloud sync exists
- Serialization: explicit versioned canonical representation
- Testing: Rust unit/property tests; integration tests; visual regression; interoperability corpus tests; E2E for critical workflows

## Architecture change gate

An implementation agent must stop and report `ARCHITECTURE_CHANGE_REQUIRED` when it requires:

- a new authoritative model;
- a second CAD mutation engine;
- a new file/interoperability authority;
- direct AI mutation of the document;
- replacing Rust core with a different core runtime;
- removing unknown-object preservation;
- weakening round-trip/data-loss guarantees;
- moving electrical semantics into the generic CAD core;
- introducing a proprietary CAD SDK as a hard dependency.
