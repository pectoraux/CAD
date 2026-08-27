#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
test -s spec/frozen-spec.sha256 || { echo 'FROZEN_SPEC_GATE_FAIL missing manifest'; exit 1; }
sha256sum -c spec/frozen-spec.sha256

git diff --quiet -- $(cat scripts/frozen-spec-list.txt) || {
  echo 'FROZEN_SPEC_GATE_FAIL frozen specification changed'
  exit 1
}

echo 'FROZEN_SPEC_GATE_PASS'
