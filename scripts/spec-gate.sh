#!/usr/bin/env bash
set -euo pipefail

required=(
  spec/architecture.md
  spec/architecture-lock.md
  spec/domain-model.md
  spec/commands.md
  spec/api.md
  spec/interoperability.md
  spec/requirements.md
  spec/dependency-graph.md
  spec/work-items.md
  docs/work-order-template.md
  docs/ARCHITECT-MASTER-PROMPT.md
  docs/reviews/ARCHITECT-REVIEW-PROTOCOL.md
  docs/reviews/CHECKPOINT-PROTOCOL.md
  docs/ARCHITECT-REVIEWER-PROMPT.md
  docs/reviews/REVIEW-PACKET-TEMPLATE.md
  docs/reviews/ARCHITECTURE-CHANGE-REQUEST-TEMPLATE.md
  docs/WORK-ORDER-HANDOFF-PROTOCOL.md
  spec/traceability.md
  spec/product-scope.md
  spec/VERSION-HISTORY.md
)
for f in "${required[@]}"; do
  test -s "$f" || { echo "SPEC_GATE_FAIL missing:$f"; exit 1; }
done

grep -q 'Status: FROZEN' spec/architecture-lock.md || { echo 'SPEC_GATE_FAIL architecture not frozen'; exit 1; }
grep -q 'Rust stable' spec/architecture-lock.md || { echo 'SPEC_GATE_FAIL Rust lock missing'; exit 1; }
grep -q 'GLM 5.3' docs/ARCHITECT-MASTER-PROMPT.md || { echo 'SPEC_GATE_FAIL GLM contract missing'; exit 1; }

bash ./scripts/verify-work-orders.sh
bash ./scripts/verify-dependencies.sh
bash ./scripts/verify-architecture-dependencies.sh
bash ./scripts/verify-work-order-state.sh

echo 'SPEC_GATE_PASS'
