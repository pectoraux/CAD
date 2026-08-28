#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

STATE_FILE="docs/execution/work-order-state.json"
test -s "$STATE_FILE" || { echo "WORK_ORDER_STATE_GATE_FAIL missing:$STATE_FILE"; exit 1; }

python3 - <<'PY'
import json
import re
import sys
from pathlib import Path

state = json.loads(Path("docs/execution/work-order-state.json").read_text())
required = {f"W{i:03d}" for i in range(1, 27)}

active = state.get("active_work_order")
if active is not None and active not in required:
    raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL unknown active work item: {active}")

verified = set(state.get("verified_work_items", []))
if not verified <= required:
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL unknown verified work item")
if active in verified:
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL active work item is already verified")

if state.get("schema_version") != "1.0":
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL unsupported state schema")

spec = Path("spec/work-items.md").read_text()
deps = {}
checkpoints = {}
for line in spec.splitlines():
    m = re.match(r"^\| (W\d+) \| .*? \| ([^|]+) \| (CP\d+) \|$", line)
    if m:
        wid, dep_text, cp = m.groups()
        deps[wid] = set() if dep_text.strip() in {"—", "-", "none"} else {x.strip() for x in dep_text.split(",")}
        checkpoints[wid] = cp
if set(deps) != required:
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL work-item set mismatch")

checkpoint_state = state.get("checkpoints", {})
for wid, required_deps in deps.items():
    if not required_deps <= verified:
        if wid == "W001" and required_deps:
            raise SystemExit("WORK_ORDER_STATE_GATE_FAIL W001 dependency model invalid")

if active is not None:
    if not deps[active] <= verified:
        missing = sorted(deps[active] - verified)
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL {active} has unverified dependencies: {missing}")
    cp = checkpoints[active]
    if checkpoint_state.get(cp) != "OPEN":
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL {active} checkpoint {cp} is not OPEN")

# During the pre-implementation baseline exactly one item is statically READY.
# Once execution begins, mutable state is authoritative and the frozen Work
# Order files are not edited.
ready = []
for i in range(1, 27):
    p = Path(f"docs/work-orders/WORK-{i:03d}.md")
    text = p.read_text()
    if re.search(r"^Status:\s*READY\s*$", text, re.MULTILINE):
        ready.append(f"W{i:03d}")
if not active and len(ready) > 1:
    raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL multiple statically READY work items: {ready}")

print("WORK_ORDER_STATE_GATE_PASS")
PY

# On pull requests the caller supplies WORK_ORDER_ID. This binds a PR to the
# architect-controlled active Work Order and prevents unrelated implementation.
if [[ -n "${WORK_ORDER_ID:-}" ]]; then
  active=$(python3 - <<'PY'
import json
from pathlib import Path
print(json.loads(Path("docs/execution/work-order-state.json").read_text()).get("active_work_order") or "")
PY
)
  [[ "$active" == "$WORK_ORDER_ID" ]] || {
    echo "WORK_ORDER_STATE_GATE_FAIL PR work item $WORK_ORDER_ID is not the active work item ($active)"
    exit 1
  }
fi
