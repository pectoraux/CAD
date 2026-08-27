#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
python3 - <<'PY2'
import re
from pathlib import Path
text=Path('spec/work-items.md').read_text()
rows={}
for line in text.splitlines():
    m=re.match(r'^\| (W\d+) \| .*? \| ([^|]+) \| (CP\d+) \|$', line)
    if m:
        wid,deps,cp=m.groups()
        rows[wid]=[] if deps.strip() in {'—','-','none'} else [x.strip() for x in deps.split(',')]
expected={f'W{i:03d}' for i in range(1,27)}
if set(rows)!=expected:
    raise SystemExit(f'DEPENDENCY_GATE_FAIL ids={sorted(set(rows)^expected)}')
for wid,deps in rows.items():
    for d in deps:
        if d not in rows:
            raise SystemExit(f'DEPENDENCY_GATE_FAIL {wid} missing dependency {d}')
visiting=set(); visited=set()
def dfs(n):
    if n in visiting:
        raise SystemExit(f'DEPENDENCY_GATE_FAIL cycle at {n}')
    if n in visited: return
    visiting.add(n)
    for d in rows[n]: dfs(d)
    visiting.remove(n); visited.add(n)
for n in rows: dfs(n)
print('DEPENDENCY_GATE_PASS')
PY2
