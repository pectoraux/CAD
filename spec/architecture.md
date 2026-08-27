# AEC CAD OS Architecture v1.0

## 1. System shape

```text
                    AEC CAD OS
                        |
        +---------------+----------------+
        |               |                |
      Desktop        Domain          Interop
        UI             Core             Core
        |               |                |
      Tauri       Electrical Model   DWG / DXF
        |               |                |
   React/TS       Rules / Catalog    Import/Export
        |               |                |
        +---------------+----------------+
                        |
                  Canonical Model
                        |
              Deterministic Commands
                        |
                    AI Gateway
                        |
                    GLM 5.3
```

## 2. Module boundaries

### `core-geometry`
Primitives, transforms, predicates, intersections, measurements, bounding boxes and tessellation inputs. No UI, persistence, DWG or electrical dependencies.

### `core-document`
Canonical document graph, entities, IDs, layers, blocks, styles, layouts, references and metadata. Depends on geometry.

### `core-commands`
Deterministic mutation transactions, preconditions, effects, undo/redo records and command registry. Depends on geometry/document.

### `core-selection`
Hit testing, spatial index, selection sets and grip state. Depends on geometry/document.

### `core-snap`
Endpoint, midpoint, center, intersection, perpendicular, tangent, nearest, grid, ortho and polar candidate generation. Depends on geometry/selection.

### `core-annotation`
Text, dimensions, leaders, hatch and annotation styles. Depends on document/geometry.

### `core-layout`
Layouts, viewports, page setup, plot configuration and output staging. Depends on document/annotation.

### `interop-dxf`
DXF parsing/writing and mapping to/from canonical model.

### `interop-dwg`
Independent DWG decoder/encoder and preservation layer. This subsystem has no authority over the canonical model.

### `domain-electrical`
Components, terminals, wires, networks, circuits, signals, relationships, catalogs, standards and reports. Depends on document/commands/annotation; generic CAD does not depend on it.

### `app-shell`
Tauri lifecycle, native windowing, file system integration, IPC.

### `app-ui`
React/TypeScript interaction surfaces: canvas orchestration, properties, layers, catalog, project navigation, reports, command palette and AI panel.

### `ai-gateway`
Provider-neutral intent/planning interface. The gateway accepts context and returns a typed plan. It cannot directly mutate state.

### `project-services`
Project metadata, cloud synchronization, identity, permissions and artifact/version metadata. This is later than the first local CAD core.

## 3. Data flow

```text
External File -> Parser -> Canonical Model -> Commands -> Canonical Model -> Writer -> External File
                                          |
                                          +-> Renderer
                                          +-> Electrical semantics
                                          +-> Reports
                                          +-> AI context
```

The canonical model is never reconstructed by scraping rendered graphics.

## 4. Execution model

All document changes execute as:

`CommandIntent -> precondition evaluation -> transaction -> validation -> commit -> event -> render`

AI follows:

`User Intent -> AI Plan -> Command Intents -> deterministic validation -> commit`

## 5. Undo/redo

Undo is command-based. A committed mutation records enough deterministic information to produce an inverse operation or restore a known prior immutable snapshot segment. UI components may never implement their own mutation history.

## 6. Performance model

- Spatial indexing is mandatory before large-drawing interactive editing.
- Renderer consumes immutable render snapshots or equivalent read-only projections.
- UI must never perform O(N) entity scans during pointer-move paths for ordinary operations.
- Geometry calculations on the hot path must avoid unnecessary heap allocations.

## 7. Storage model

The canonical runtime model is in-memory and transactionally mutated. Persistent project/document storage is versioned and schema-controlled. External file import creates an import record and diagnostics. Save/export creates an explicit artifact version.
