#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

python3 - <<'PY'
import json
import subprocess
import sys

expected = {
    "aeccad-core-geometry": set(),
    "aeccad-core-document": {"aeccad-core-geometry"},
    "aeccad-core-commands": {"aeccad-core-geometry", "aeccad-core-document"},
    "aeccad-core-selection": {"aeccad-core-geometry", "aeccad-core-document"},
    "aeccad-core-snap": {"aeccad-core-geometry", "aeccad-core-selection"},
    "aeccad-core-annotation": {"aeccad-core-geometry", "aeccad-core-document"},
    "aeccad-core-layout": {"aeccad-core-document", "aeccad-core-annotation"},
    "aeccad-interop-dxf": {"aeccad-core-document"},
    "aeccad-interop-dwg": {"aeccad-core-document"},
    "aeccad-domain-electrical": {"aeccad-core-document", "aeccad-core-commands", "aeccad-core-annotation"},
    "aeccad-ai-gateway": set(),
}

raw = subprocess.check_output(
    ["cargo", "metadata", "--format-version", "1", "--no-deps"],
    text=True,
)
metadata = json.loads(raw)
workspace = {p["name"] for p in metadata["packages"] if p["source"] is None}
if workspace != set(expected):
    print("ARCH_DEP_GATE_FAIL workspace crate set mismatch", file=sys.stderr)
    print("expected:", sorted(expected), file=sys.stderr)
    print("actual:", sorted(workspace), file=sys.stderr)
    sys.exit(1)

# --no-deps does not expose dependency edges. Read each manifest directly and
# reject any workspace dependency not permitted by the frozen architecture.
for package in metadata["packages"]:
    name = package["name"]
    if name not in expected:
        continue
    manifest = package["manifest_path"]
    import pathlib
    text = pathlib.Path(manifest).read_text()
    actual = set()
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("aeccad-"):
            continue
        dep = stripped.split(" ", 1)[0].strip('=')
        if dep in expected:
            actual.add(dep)
    forbidden = actual - expected[name]
    if forbidden:
        print(f"ARCH_DEP_GATE_FAIL {name} forbidden dependencies: {sorted(forbidden)}", file=sys.stderr)
        sys.exit(1)

print("ARCHITECTURE_DEPENDENCY_GATE_PASS")
PY
