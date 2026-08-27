# DWG/DXF Interoperability Strategy v1.1

## Objective

Deliver progressive professional compatibility without making a proprietary SDK a hard dependency.

## Architecture

```text
External File
   |
Decoder
   |
External Intermediate Representation
   |
Canonical Mapping
   |
Canonical Model
   |
Canonical Mapping
   |
External Intermediate Representation
   |
Encoder
   |
External File
```

## DWG compatibility levels

### Level 1 — Read

Decode file header/version metadata and high-frequency entity/object structures.

### Level 2 — Preserve

Store unsupported entities/objects as opaque payloads with ownership/handle information where safe.

### Level 3 — Render

Render supported proxy graphics without claiming semantic support.

### Level 4 — Edit

Edit canonical supported entities while preserving unrelated opaque content.

### Level 5 — Write

Write certified compatibility profiles.

### Level 6 — Round trip

Prove `external -> ours -> external -> ours` semantic and visual stability over the certification corpus.

## Initial compatibility profile

Read/write priority:

- LINE
- LWPOLYLINE / POLYLINE
- ARC
- CIRCLE
- ELLIPSE
- SPLINE
- POINT
- XLINE / RAY
- TEXT / MTEXT
- HATCH
- DIMENSION
- LEADER / MLEADER
- INSERT / BLOCK / ATTRIB / ATTDEF
- layers
- linetypes
- text styles
- dimension styles
- block records
- layouts
- viewports
- plot settings
- external references metadata

## Explicitly later

3D solids/surfaces/meshes, complex proxy semantics, full Autodesk application-defined object behavior, full AutoLISP/ObjectARX compatibility and DGN semantics.

## Data-loss policy

No export is “successful” if unsupported data was silently discarded. Export returns a diagnostics report with:

- preserved opaque objects;
- unsupported-but-rendered objects;
- unsupported-and-not-rendered objects;
- degraded semantics;
- entities omitted by user-selected export policy.

## Corpus strategy

The repository must maintain a legally usable corpus organized by discipline and complexity. Every interoperability change adds or updates fixtures. Each certified profile requires parser tests, render regression, round-trip tests and reopen tests.
