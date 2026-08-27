# Requirements v1.0

## Product requirements

### CAD-001 — Professional 2D drafting
The system shall support the P0 geometry and modify workflows defined in the command contract.

### CAD-002 — Precision drafting
The system shall provide deterministic snaps, ortho, polar tracking and coordinate input.

### CAD-003 — Document organization
The system shall provide layers, blocks, styles, properties, layouts and viewports.

### CAD-004 — Production documentation
The system shall provide text, dimensions, leaders, hatches and plotting/PDF output.

### CAD-005 — Command determinism
Identical command inputs against identical document revisions produce identical results.

### ELEC-001 — Semantic components
Components, terminals, wires, networks and relationships shall be represented as domain objects.

### ELEC-002 — Engineering synchronization
A logical component shall support multiple graphical representations linked to one identity.

### ELEC-003 — Catalog
Catalog parts and manufacturer data shall be separable from geometry and assignable to components.

### ELEC-004 — Automation
Wire numbering, component tagging, cross-reference generation and BOM/report generation shall be deterministic.

### ELEC-005 — Validation
Engineering validation shall report errors/warnings with object-level traceability.

### INTEROP-001 — DWG/DXF read
High-frequency production drawings shall import into the canonical model or an explicit opaque-preservation representation.

### INTEROP-002 — No silent loss
Unsupported content must generate diagnostics and cannot silently disappear.

### INTEROP-003 — Round trip
Certified compatibility profiles shall meet defined semantic/visual regression thresholds.

### AI-001 — Intent planning
AI shall translate natural-language requests into typed command plans.

### AI-002 — Deterministic execution
AI cannot directly mutate the document.

### AI-003 — Explainability
Every AI-generated plan shall expose the commands it proposes and validation outcomes.

### PERF-001 — Interactive response
Pointer-driven selection/snapping/modification shall remain within defined latency budgets on the reference benchmark corpus.

### GOV-001 — Architecture lock
Implementation agents shall not alter frozen architecture without escalation.

## Acceptance criteria families

Every Work Item must reference one or more of: geometry correctness, command determinism, visual regression, interoperability corpus, electrical graph integrity, security/permission correctness, or performance evidence.
