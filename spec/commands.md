# Deterministic Command and Transaction Contract v1.1

## Command envelope

Every mutation is represented as:

```text
CommandIntent {
  command_type
  command_version
  transaction_id
  actor
  document_id
  expected_revision
  input
}
```

The engine returns:

```text
CommandResult {
  transaction_id
  status: APPLIED | REJECTED
  resulting_revision
  affected_entity_ids[]
  diagnostics[]
  inverse_command | snapshot_reference
}
```

## Generic CAD commands — V1

`CreateLine`, `CreatePolyline`, `CreateArc`, `CreateCircle`, `CreateRectangle`, `CreateSpline`, `DeleteEntities`, `MoveEntities`, `CopyEntities`, `TrimEntities`, `ExtendEntities`, `OffsetEntities`, `StretchEntities`, `RotateEntities`, `MirrorEntities`, `ScaleEntities`, `FilletEntities`, `ChamferEntities`, `BreakEntities`, `JoinEntities`, `ExplodeBlockReference`, `SetEntityProperties`, `SetLayerProperties`, `CreateBlock`, `InsertBlock`, `UpdateBlockAttributes`, `CreateText`, `CreateMText`, `CreateDimension`, `CreateLeader`, `CreateMLeader`, `CreateHatch`, `CreateLayout`, `CreateViewport`, `SetPageSetup`, `PlotDrawing`, `ImportReference`, `PurgeDrawing`, `AuditDrawing`, `Undo`, `Redo`.

## Electrical commands — V1

`CreateComponent`, `PlaceSymbol`, `AssignCatalogPart`, `ConnectTerminal`, `CreateWireNetwork`, `AddWireSegment`, `MoveWireNetwork`, `NumberWires`, `TagComponents`, `CreateSignal`, `CreateCrossReference`, `CreateTerminal`, `CreateTerminalStrip`, `GenerateBOM`, `GenerateElectricalReport`, `ValidateElectricalDesign`.

## Preconditions

Commands MUST reject rather than guess when:

- a referenced ID does not exist;
- expected revision is stale and merge semantics are undefined;
- geometry is invalid for the operation;
- a required layer/style/block definition is missing;
- an electrical connection violates the active rule set;
- a catalog assignment violates the schema;
- an import/export invariant would be violated.

## AI safety boundary

AI output is never executable code. It must conform to a typed plan schema and pass deterministic validation. The model has no database/file-write capability.

## Idempotency

Every transaction has a unique ID. Replayed transactions with the same document revision and idempotency key must produce the same result or an explicit duplicate result; they must never apply twice.


## Command registry rule

Every command type is a closed, versioned contract. A Work Order may implement only command types listed here. Adding a new command requires an architecture change or a later version's command contract.

## Canonical primitive types

```text
Point2 { x: f64, y: f64 }
Vector2 { x: f64, y: f64 }
Transform2D { translation: Vector2, rotation_rad: f64, scale_x: f64, scale_y: f64 }
EntitySelection { entity_ids: EntityId[] }
ExpectedRevision { document_id: DrawingId, revision: u64 }
```

## V1 command payload rules

Each command below MUST validate the listed inputs; omitted semantic behavior is not implementation-defined.

### Creation

- `CreateLine(start, end, layer_id)` — creates one line; zero-length line is rejected.
- `CreatePolyline(vertices, closed, layer_id)` — vertices length >= 2; consecutive duplicate vertices are rejected unless the command version explicitly permits them.
- `CreateArc(center, radius, start_angle, end_angle, layer_id)` — radius > 0 and finite; normalized angular representation is deterministic.
- `CreateCircle(center, radius, layer_id)` — radius > 0 and finite.
- `CreateRectangle(min_corner, max_corner, layer_id)` — both dimensions > 0; produces a closed polyline under the current rectangle command version.
- `CreateSpline(control_points, degree, layer_id)` — minimum control-point count and degree rules are explicit in the geometry API; invalid combinations reject.

### Modification

- `DeleteEntities(selection)` — deletes only selected entities and owned dependent objects permitted by delete policy; required references cause rejection unless explicit cascade behavior is part of the command version.
- `MoveEntities(selection, displacement)` — applies one exact translation to every selected entity.
- `CopyEntities(selection, displacement, copy_count)` — original remains unchanged; copies receive new IDs; repeated placement is deterministic.
- `TrimEntities(boundaries, targets, mode)` — only intersections permitted by the geometry contract are applied; ambiguous candidate selection rejects instead of guessing.
- `ExtendEntities(boundaries, targets, mode)` — extends only to valid intersections; ambiguous/no-valid-boundary cases reject.
- `OffsetEntities(selection, distance, side)` — distance must be non-zero and finite; self-intersections or ambiguous offset topology reject with diagnostics rather than silently repair.
- `StretchEntities(selection, crossing_window, displacement)` — selection semantics are explicit: crossing-selected vertices move; fully enclosed rigid entities follow command policy.
- `RotateEntities(selection, base_point, angle_rad)` — deterministic rotation around one base point.
- `MirrorEntities(selection, axis_start, axis_end, keep_source)` — zero-length axis rejects; copied IDs are deterministic but not caller-specified.
- `ScaleEntities(selection, base_point, factor)` — factor > 0 and finite.
- `FilletEntities(first, second, radius, trim_mode)` — creates tangent connection when mathematically valid; otherwise rejects.
- `ChamferEntities(first, second, distance1, distance2, trim_mode)` — valid distances required; otherwise rejects.
- `BreakEntities(target, break_points)` — break points must lie on the target within canonical tolerance.
- `JoinEntities(selection)` — only compatible geometry is joined; order and resulting entity type are deterministic.

### Organization/documentation

`SetEntityProperties`, `SetLayerProperties`, `CreateBlock`, `InsertBlock`, `UpdateBlockAttributes`, `CreateText`, `CreateMText`, `CreateDimension`, `CreateLeader`, `CreateMLeader`, `CreateHatch`, `CreateLayout`, `CreateViewport`, `SetPageSetup`, `PlotDrawing`, `ImportReference`, `PurgeDrawing`, and `AuditDrawing` MUST expose typed payloads in the Rust contract; UI-specific JSON is forbidden from becoming the domain contract.

### Electrical

`CreateComponent`, `PlaceSymbol`, `AssignCatalogPart`, `ConnectTerminal`, `CreateWireNetwork`, `AddWireSegment`, `MoveWireNetwork`, `NumberWires`, `TagComponents`, `CreateSignal`, `CreateCrossReference`, `CreateTerminal`, `CreateTerminalStrip`, `GenerateBOM`, `GenerateElectricalReport`, and `ValidateElectricalDesign` mutate only through the electrical application service and underlying deterministic command service.

## Command result semantics

A rejected command produces no partial mutation, no revision advance, and no externally visible event. A successful command commits atomically. Validation is performed against the post-command candidate state before commit.

## Undo/redo semantics

Undo and redo operate on committed command records in document order. Undo never calls the AI planner. Redo replays the original normalized command, not a newly generated interpretation.
