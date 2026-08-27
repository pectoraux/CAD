# Canonical Domain Model v1.0

All IDs are opaque, globally unique within their type and never reused. The preferred wire form is UUIDv7 or an equivalent sortable random identifier. Handles originating from external formats are preserved separately and never reused as primary IDs.

## Identity types

```text
ProjectId
DrawingId
EntityId
ObjectId
LayerId
BlockDefinitionId
BlockReferenceId
StyleId
DimensionStyleId
LayoutId
ViewportId
ExternalRefId
ComponentId
TerminalId
WireId
WireNetworkId
ConnectionId
SignalId
CatalogPartId
ManufacturerId
ReportDefinitionId
CommandId
TransactionId
ArtifactVersionId
```

## Generic CAD entities

### `Entity`

- `id: EntityId`
- `layer_id: LayerId`
- `owner_block_id: BlockDefinitionId | null`
- `transform: Transform2D`
- `visibility: VisibilityState`
- `common_style: StyleRef`
- `source_provenance: Provenance`

Specializations for V1:

`Line`, `Polyline`, `Arc`, `Circle`, `Ellipse`, `Spline`, `Point`, `Ray`, `XLine`, `Text`, `MText`, `Hatch`, `Dimension`, `Leader`, `MLeader`, `Insert`, `Attribute`, `Solid2D`.

## Document objects

### `Drawing`

- `id`
- `project_id`
- `name`
- `units`
- `model_space_root`
- `layouts[]`
- `layers[]`
- `linetypes[]`
- `text_styles[]`
- `dimension_styles[]`
- `blocks[]`
- `external_refs[]`
- `metadata`

### `Layer`

- `id`
- `name`
- `color`
- `linetype_id`
- `lineweight`
- `transparency`
- `visible`
- `locked`
- `frozen`
- `plot_enabled`
- `description`

### `BlockDefinition`

- `id`
- `name`
- `base_point`
- `entities[]`
- `attributes[]`
- `dynamic_definition | null`

### `BlockReference`

- `id`
- `block_definition_id`
- `transform`
- `attribute_values[]`
- `explodable`

### `Layout`

- `id`
- `name`
- `paper_size`
- `orientation`
- `plot_settings`
- `viewports[]`

### `Viewport`

- `id`
- `center_model`
- `scale`
- `twist`
- `layer_overrides`
- `display_mode`

## Interoperability entities

### `OpaqueExternalObject`

- `id`
- `source_format`
- `source_version`
- `external_type`
- `external_handle`
- `owner_external_handle`
- `raw_payload`
- `proxy_graphics | null`
- `preservation_status`
- `diagnostics[]`

Invariant: opaque objects can never silently disappear during an otherwise successful round trip.

### `ExternalReference`

- `id`
- `source_path`
- `source_kind`
- `resolved_artifact_hash | null`
- `display_transform`
- `status`

## Electrical domain

### `ElectricalProject`

- `project_id`
- `standard_profile_id`
- `default_units`
- `tagging_rules`
- `wire_number_rules`
- `catalog_id`
- `report_profiles[]`

### `Component`

- `id`
- `tag`
- `family`
- `description`
- `catalog_part_id | null`
- `manufacturer_id | null`
- `parent_component_id | null`
- `rating_set`
- `location`
- `installation`
- `representations[]`
- `terminals[]`
- `source_drawing_id`
- `source_entity_id`

### `ComponentRepresentation`

- `component_id`
- `kind: Schematic | OneLine | Panel | Report`
- `symbol_definition_id | null`
- `entity_ids[]`

### `Terminal`

- `id`
- `component_id`
- `number`
- `name | null`
- `direction`
- `rating_set`
- `spare`
- `internal_external`

### `WireNetwork`

- `id`
- `wire_number`
- `wire_type`
- `gauge | null`
- `color | null`
- `voltage | null`
- `wire_ids[]`
- `terminal_ids[]`
- `component_terminal_refs[]`
- `signal_id | null`

### `Wire`

- `id`
- `network_id`
- `geometry_entity_ids[]`
- `from_connection_id`
- `to_connection_id | null`

### `Connection`

- `id`
- `from_terminal_id`
- `to_terminal_id | null`
- `wire_id`
- `status`

### `Signal`

- `id`
- `name`
- `source_endpoint`
- `destination_endpoints[]`
- `references[]`

### `CatalogPart`

- `id`
- `manufacturer_id`
- `part_number`
- `family`
- `description`
- `ratings`
- `dimensions`
- `symbol_definition_id | null`
- `footprint_definition_id | null`
- `datasheet_uri | null`
- `cost | null`
- `availability | null`

### `ReportDefinition`

- `id`
- `name`
- `query`
- `columns[]`
- `filters[]`
- `grouping[]`
- `sorting[]`
- `formatting`

## Relationship invariants

1. Every `Component` belongs to exactly one electrical project.
2. Every `Terminal` belongs to exactly one `Component`.
3. Every `Wire` belongs to exactly one `WireNetwork`.
4. A `Connection` may only connect terminals that exist and are compatible with the connection rule set.
5. A logical `Component` may have multiple graphical representations but remains one identity.
6. A `WireNetwork` may span multiple drawing views but has one logical identity.
7. Reports query the canonical engineering graph, not screen pixels.
8. Catalog assignment is metadata and does not define geometry ownership.
9. Generic entities may exist without electrical meaning.
10. Electrical semantics may reference generic entities; generic entities must not import electrical modules.
