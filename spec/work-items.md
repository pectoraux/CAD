# Work Items v1.1

Each Work Item maps to exactly one primary architectural capability and has a bounded Work Order.

| ID | Objective | Dependencies | Checkpoint |
|---|---|---|---|
| W001 | repo foundation, Rust workspace, governance gates | none | CP0 |
| W002 | geometry primitives and predicates | W001 | CP1 |
| W003 | canonical document model | W002 | CP1 |
| W004 | spatial index and selection | W003 | CP2 |
| W005 | snapping and precision input | W004 | CP2 |
| W006 | transaction/command engine | W003 | CP2 |
| W007 | P0 modify commands | W005,W006 | CP2 |
| W008 | annotation/styles | W003,W006 | CP3 |
| W009 | layouts/viewports/plot | W008 | CP3 |
| W010 | DXF read/write | W003 | CP3 |
| W011 | DWG header/section reader | W003 | CP4 |
| W012 | DWG entity/object decoder + opaque preservation | W011 | CP4 |
| W013 | DWG writer compatibility profile A | W012,W015 | CP4 |
| W014 | interoperability corpus and round-trip harness | W010,W013 | CP4 |
| W015 | blocks/attributes/xrefs production semantics | W007,W010,W012 | CP4 |
| W016 | electrical project domain | W003,W006 | CP5 |
| W017 | components/representations/terminals | W016,W008 | CP5 |
| W018 | wires/networks/connections/signals | W017,W007 | CP5 |
| W019 | manufacturer/catalog/parts | W017 | CP5 |
| W020 | tagging/numbering/cross-references | W018,W019 | CP5 |
| W021 | BOM/report engine | W018,W019,W020 | CP5 |
| W022 | electrical rules/validation | W018,W019 | CP5 |
| W023 | typed AI planning gateway | W006,W020,W022 | CP6 |
| W024 | Tauri desktop shell + canvas host | W007,W009 | CP3 |
| W025 | electrical UX | W020,W021,W022,W023,W024 | CP5 |
| W026 | V1 certification and release gate | W014,W015,W021,W022,W023,W025 | CP7 |

## Work Item contract

Every item must specify: objective, dependencies, owned modules, inputs/outputs, acceptance criteria, tests/evidence, out of scope, stop conditions, and definition of done.
