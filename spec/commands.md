# Deterministic Command and Transaction Contract v1.0

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
