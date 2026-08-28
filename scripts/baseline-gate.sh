#!/usr/bin/env bash
set -euo pipefail
bash ./scripts/spec-gate.sh
bash ./scripts/frozen-spec-gate.sh
bash ./scripts/verify-work-orders.sh
bash ./scripts/verify-dependencies.sh
bash ./scripts/verify-architecture-dependencies.sh
bash ./scripts/verify-work-order-state.sh

test -f Cargo.toml
test -f docs/BASELINE.md

cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo 'BASELINE_GATE_PASS'
