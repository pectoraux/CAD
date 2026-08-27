# Architect Review Protocol v1.0

The architect/reviewer is independent from the implementation agent.

## Authority order

1. Frozen architecture
2. Requirements
3. Work Item
4. Work Order
5. Implementation
6. Agent claims

Claims never override evidence.

## Review inputs

- exact merge base;
- changed files;
- Work Order;
- CI results;
- tests/evidence;
- architecture checks;
- relevant corpus/render diffs;
- performance evidence where applicable;
- prior checkpoint verdicts.

## Review sequence

1. **Scope** — did the PR implement only the Work Order?
2. **Architecture** — are module boundaries and invariants intact?
3. **Correctness** — do acceptance criteria have objective evidence?
4. **Regression** — do previous checkpoint guarantees still hold?
5. **Data integrity** — any silent loss, corruption or non-determinism?
6. **Security/safety** — can invalid AI/input mutate authoritative state?
7. **Performance** — does the change violate hot-path budgets?
8. **Maintainability** — does the implementation preserve explicit contracts?

## Verdicts

`APPROVED`

`CHANGES_REQUESTED`

`ARCHITECTURE_CHANGE_REQUIRED`

`IMPLEMENTATION_BLOCKED`

## Mandatory CAD review checks

For geometry changes:

- exact/robust predicates covered;
- degenerates handled;
- undo deterministic;
- rendering and hit-testing agree;
- no coordinate-system ambiguity.

For interoperability changes:

- fixture corpus evidence;
- round-trip evidence;
- unsupported-object diagnostics;
- no silent loss;
- reopen/audit evidence when available.

For AI changes:

- typed output only;
- deterministic command execution;
- no direct filesystem/database mutation;
- malformed/hallucinated commands rejected;
- provenance/explanation retained.
