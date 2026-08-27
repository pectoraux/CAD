#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
manifest="spec/frozen-spec.sha256"
test -s "$manifest" || { echo 'FROZEN_SPEC_GATE_FAIL missing manifest'; exit 1; }
# The manifest itself is protected by a bootstrap hash. Update only through an Architect-approved freeze.
EXPECTED_FROZEN_LIST_SHA256="7144bc8cd934d4afe0c138977f5d9cf8cc2594ad48997809f2109e7c1ef8188d"
actual_list_hash=$(sha256sum scripts/frozen-spec-list.txt | awk '{print $1}')
if [[ "$actual_list_hash" != "$EXPECTED_FROZEN_LIST_SHA256" ]]; then
  echo 'FROZEN_SPEC_GATE_FAIL frozen file list altered'; exit 1
fi
EXPECTED_MANIFEST_SHA256="07b4e9e7c8a68ad3f03492f1ca451164e6b2ba13c2cc82972c4b8bf21a52b8ec"
actual_manifest_hash=$(sha256sum "$manifest" | awk '{print $1}')
if [[ "$actual_manifest_hash" != "$EXPECTED_MANIFEST_SHA256" ]]; then
  echo 'FROZEN_SPEC_GATE_FAIL manifest altered'; exit 1
fi
sha256sum -c "$manifest"
if [[ -n "${BASE_SHA:-}" ]]; then
  git diff --quiet "$BASE_SHA" HEAD -- $(cat scripts/frozen-spec-list.txt) || {
    echo 'FROZEN_SPEC_GATE_FAIL frozen files changed in commit range'; exit 1;
  }
fi
# Never allow a working-tree modification to hide behind committed history.
git diff --quiet -- $(cat scripts/frozen-spec-list.txt) || {
  echo 'FROZEN_SPEC_GATE_FAIL frozen specification changed in working tree'; exit 1
}
echo FROZEN_SPEC_GATE_PASS
