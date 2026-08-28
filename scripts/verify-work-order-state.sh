#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

STATE_FILE="docs/execution/work-order-state.json"
test -s "$STATE_FILE" || { echo "WORK_ORDER_STATE_GATE_FAIL missing:$STATE_FILE"; exit 1; }

python3 - <<'PY'
import json
import re
from pathlib import Path

state = json.loads(Path("docs/execution/work-order-state.json").read_text())
required = {f"W{i:03d}" for i in range(1, 27)}

if state.get("schema_version") != "1.0":
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL unsupported state schema")

active = state.get("active_work_order")
if active is not None and active not in required:
    raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL unknown active work item: {active}")

verified = set(state.get("verified_work_items", []))
if not verified <= required:
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL unknown verified work item")
if active in verified:
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL active work item is already verified")

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

for wid in verified:
    missing = deps[wid] - verified
    if missing:
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL {wid} has unverified dependencies: {sorted(missing)}")

checkpoint_state = state.get("checkpoints", {})
for wid in verified:
    cp = checkpoints[wid]
    if checkpoint_state.get(cp) not in {"OPEN", "PASSED"}:
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL verified {wid} has checkpoint {cp} not OPEN/PASSED")

if active is not None:
    missing = deps[active] - verified
    if missing:
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL {active} has unverified dependencies: {sorted(missing)}")
    cp = checkpoints[active]
    if checkpoint_state.get(cp) != "OPEN":
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL {active} checkpoint {cp} is not OPEN")

ready = []
for i in range(1, 27):
    text = Path(f"docs/work-orders/WORK-{i:03d}.md").read_text()
    if re.search(r"^Status:\s*READY\s*$", text, re.MULTILINE):
        ready.append(f"W{i:03d}")

if active is None and not verified:
    if len(ready) != 1:
        raise SystemExit(f"WORK_ORDER_STATE_GATE_FAIL expected exactly one initial READY work item, found: {ready}")

if active is None and verified and checkpoint_state.get("CP0") != "PASSED":
    raise SystemExit("WORK_ORDER_STATE_GATE_FAIL verified idle state requires CP0=PASSED")

print("WORK_ORDER_STATE_GATE_PASS")
PY

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
