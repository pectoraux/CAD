# API Contracts v1.0

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

`get_component(component_id)`

`get_network(network_id)`

`connect_terminals(input)`

`renumber_wires(scope, strategy)`

`retag_components(scope, strategy)`

`generate_report(report_definition_id, scope)`

`validate_engineering(scope)`

## `AI Planning API`

`plan_intent(context, user_intent) -> CommandPlan`

The returned plan contains only typed command intents and explanations. It cannot contain arbitrary SQL, filesystem operations, shell commands or UI automation.

## Error contract

Errors are typed and stable:

`NotFound`, `InvalidInput`, `StaleRevision`, `GeometryInvalid`, `ConstraintViolation`, `UnsupportedObject`, `ImportDegraded`, `ExportDegraded`, `DataLossPrevented`, `PermissionDenied`, `InternalInvariantFailure`.
