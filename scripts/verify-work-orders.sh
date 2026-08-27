#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
for f in docs/work-orders/WORK-*.md; do
  for h in "## Identity" "## Objective" "## Allowed changes" "## Required implementation" "## Forbidden changes" "## Acceptance criteria" "## Required tests/evidence" "## Scope boundary" "## Stop conditions" "## Definition of done" "## Final response"; do
    grep -qF "$h" "$f" || { echo "WORK_ORDER_GATE_FAIL $f missing:$h"; exit 1; }
  done
done
count=$(find docs/work-orders -name 'WORK-*.md' | wc -l)
test "$count" -eq 26 || { echo "WORK_ORDER_GATE_FAIL expected 26 work orders, found $count"; exit 1; }
echo WORK_ORDER_GATE_PASS
