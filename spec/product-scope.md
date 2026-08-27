# Product Scope and Feature Priority v1.1

## Product identity

AEC CAD OS is an AI-native professional electrical CAD system with a generic deterministic 2D CAD core. It is intended to replace the high-frequency professional AutoCAD/AutoCAD Electrical workflow while adding semantic engineering intelligence. It is not a day-one clone of every Autodesk feature.

## Feature priority

### P0 — V1 launch-critical

Generic CAD: LINE, PLINE, ARC, CIRCLE, RECTANGLE, SPLINE; selection/grips; MOVE, COPY, ERASE, TRIM, EXTEND, OFFSET, STRETCH, ROTATE, MIRROR, SCALE, FILLET, CHAMFER, BREAK, JOIN, PEDIT, EXPLODE; OSNAP, ORTHO, POLAR and coordinate input; layers/properties/linetypes/lineweights; text/mtext/dimensions/leaders/hatches; blocks/attributes; layouts/viewports/page setup/plot/PDF; undo/redo.

Electrical: projects/drawings; components; symbols; terminals; wires; wire networks; explicit connections; signals; multi-view representations; catalog parts/manufacturers; component tagging; wire numbering; cross-references; BOM and core reports; deterministic validation.

Interoperability: DXF; progressive DWG read/preserve/write; explicit degradation diagnostics; no silent loss.

AI: typed intent planning only; deterministic command execution; provenance and explanations; stale/invalid-plan rejection.

### P1 — Post-launch professional depth

Dynamic blocks, advanced block attributes, Xrefs, data extraction, sheet sets, advanced electrical terminal management, PLC, connectors, cables, panel automation, standards engine, ECLASS/manufacturer catalog ingestion, advanced reports, project-wide synchronization, broader DWG object support.

### P2 — Later platform expansion

MEP semantic objects, architecture/BIM semantics, 3D solids/surfaces/meshes, DGN semantics, full cloud collaboration, advanced API/plugin ecosystem, simulation, CAM, Revit/ArchiCAD/SolidWorks parity.

## Non-goals for V1

No 3D mechanical kernel, no full BIM model, no full Revit/ArchiCAD/SolidWorks reproduction, no AutoLISP/ObjectARX compatibility promise, no full DGN semantics, no circuit/thermal simulation, no CAM.

## Research-derived interaction principle

High-frequency direct manipulation and modification operations are the primary UX. AI augments these operations but never replaces deterministic CAD semantics.
