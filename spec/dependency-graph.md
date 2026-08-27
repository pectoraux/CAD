# Dependency Graph v1.0

```text
W001 Foundation
  |
W002 Geometry model
  |
W003 Document model
  |
W004 Spatial index + selection
  |
W005 Snap engine
  |
W006 Command/transaction engine
  |
W007 Core modify commands
  |
W008 Annotation + styles
  |
W009 Layout + plot
  |
W010 DXF codec
  |
W011 DWG read foundation
  |
W012 DWG preservation + core entities
  |
W013 DWG write profile A
  |
W014 Round-trip harness
  |
W015 Block/Xref semantics
  |
W016 Electrical project model
  |
W017 Electrical components/terminals
  |
W018 Wires/networks/connections
  |
W019 Catalog
  |
W020 Tagging/numbering/cross-reference
  |
W021 Reports/BOM
  |
W022 Electrical validation
  |
W023 AI command planning
  |
W024 Desktop UX shell
  |
W025 Electrical UX
  |
W026 End-to-end certification
```

## Checkpoints

- **CP0**: repository/spec baseline green before implementation.
- **CP1**: canonical geometry/document contracts frozen and executable.
- **CP2**: deterministic editing loop demonstrated.
- **CP3**: production-document workflow demonstrated.
- **CP4**: DWG read/preserve/write profile A passes corpus gates.
- **CP5**: electrical graph integrity and automation demonstrated.
- **CP6**: AI planner passes adversarial deterministic execution tests.
- **CP7**: full V1 certification.

Parallelism is allowed only for work items whose dependencies and module boundaries are satisfied. Checkpoints are mandatory architectural review barriers.
