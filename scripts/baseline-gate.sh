#!/usr/bin/env bash
set -euo pipefail
./scripts/spec-gate.sh
./scripts/frozen-spec-gate.sh
./scripts/verify-work-orders.sh
test -f Cargo.toml
test -f docs/BASELINE.md
echo 'BASELINE_GATE_PASS'
