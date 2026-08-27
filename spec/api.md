# API Contracts v1.1

The first implementation may expose these through internal Rust traits/commands before a public network API exists.

## `DocumentService`

`open_document(source) -> OpenResult`

`get_document_snapshot(document_id, revision) -> Snapshot`

`apply_command(command_intent) -> CommandResult`

`validate_document(document_id, scope) -> ValidationResult`

`export_document(document_id, target_format, compatibility_profile) -> ExportResult`

## `SelectionService`

`hit_test(view, point, tolerance) -> SelectionCandidates`

`select_by_window(view, rectangle, mode) -> SelectionSet`

## `SnapService`

`snap_candidates(view, cursor, context) -> SnapCandidate[]`

Candidates are ranked deterministically by snap priority, distance, entity order and stable ID.

## `ElectricalService`

`get_component(component_id)` — side-effect free.

`get_network(network_id)` — side-effect free.

`connect_terminals(input)` — returns a validated `CommandIntent`/`CommandResult`; it does not mutate independently of `CommandService`.

`renumber_wires(scope, strategy)` — returns a preview or typed command plan; commit occurs only through `CommandService`.

`retag_components(scope, strategy)` — returns a preview or typed command plan; commit occurs only through `CommandService`.

`generate_report(report_definition_id, scope)` — side-effect free over the canonical graph.

`validate_engineering(scope)` — side-effect free; diagnostics only.

## `CommandService`

`apply(command_intent) -> CommandResult` is the sole mutation entry point for the CAD document. No UI, importer, report generator or AI adapter may mutate canonical state through another path.

## `ValidationService`

`validate_candidate(document_revision, command_intent) -> ValidationResult` is side-effect free.

## `RevisionService`

`get_revision(document_id, revision) -> ImmutableSnapshot`
`list_revision_history(document_id) -> Revision[]`

## `AI Planning API`

`plan_intent(context, user_intent) -> CommandPlan`

The returned plan contains only typed command intents and explanations. It cannot contain arbitrary SQL, filesystem operations, shell commands or UI automation.

## Error contract

Errors are typed and stable:

`NotFound`, `InvalidInput`, `StaleRevision`, `GeometryInvalid`, `ConstraintViolation`, `UnsupportedObject`, `ImportDegraded`, `ExportDegraded`, `DataLossPrevented`, `PermissionDenied`, `InternalInvariantFailure`.


## Typed `CommandPlan`

```text
CommandPlan {
  schema_version: string,
  planner_provider: string,
  planner_model: string,
  source_intent: string,
  target_document_id: DrawingId,
  expected_revision: u64,
  commands: PlannedCommand[],
  assumptions: Assumption[],
  explanation: string
}

PlannedCommand {
  command_type: ClosedCommandType,
  command_version: string,
  input: TypedCommandInput,
  preconditions: DeclaredPrecondition[],
  explanation: string
}
```

The validator rejects unknown command types, unknown fields, invalid enum values, stale revisions, unresolved entity references, unsupported assumptions and plans whose declared preconditions no longer hold. No free-form tool call is accepted as a command.

## API authority rules

There is exactly one authoritative mutation path: `CommandService.apply`. ElectricalService mutation-shaped helpers are planners/delegators and may never write canonical state directly.

- APIs return immutable snapshots/results rather than mutable live references.
- Cross-module APIs expose public DTOs/value objects; `internal/` types are never shared.
- Error codes are stable within a major contract version.
- Any side effect is named in the API contract and occurs only in the owner module.
