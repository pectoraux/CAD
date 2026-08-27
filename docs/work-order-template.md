# Work Order Template — GLM 5.3

## Identity

- Work Order: `WORK-XXX`
- Work Item: `WI-XXX`
- Architecture version: `v1.1`
- Parent checkpoint:
- Dependencies:
- Status: `BLOCKED | READY | IMPLEMENTING | PR_OPEN | VERIFYING | CHANGES_REQUESTED | APPROVED`

## Authority

Read, in order:

1. `spec/architecture-lock.md`
2. `spec/architecture.md`
3. `spec/domain-model.md`
4. `spec/commands.md`
5. `spec/api.md`
6. `spec/requirements.md`
7. `spec/traceability.md`
8. `spec/interoperability.md`
9. `spec/dependency-graph.md`
10. `spec/work-items.md`
11. this Work Order

Lower-ranked material cannot override higher-ranked material.

## Objective

One bounded objective.

## Allowed changes

Exact crates/modules/subsystems that may change.

## Required implementation

Exact entities, fields, APIs, command semantics, invariants, persistence and behaviors. No unspecified behavior is implied.

## Forbidden changes

Explicit modules/files/authorities that must remain untouched.

## Acceptance criteria

Every criterion has objective evidence.

## Required tests/evidence

Specify unit, property, integration, regression, visual, corpus, fuzz, performance and security evidence as applicable.

## Scope boundary

List dependent/future Work Items that must not be implemented.

## Stop conditions

If semantics are missing or contradictory, output:

`ARCHITECTURE_CHANGE_REQUIRED`

If repository state prevents safe implementation without redesign, output:

`IMPLEMENTATION_BLOCKED`

Do not guess.

## Definition of done

- required implementation complete;
- only allowed files changed;
- all acceptance criteria evidenced;
- typecheck/lint/build pass where applicable;
- architecture checks pass;
- regression suite passes;
- no silent data loss;
- no authority bypass;
- final response includes exact evidence identifiers;
- implementation PR is opened for Architect Review.

## Final response

```text
WORK-XXX COMPLETE

Implementation summary:
Files changed:
Acceptance evidence:
Architecture invariants checked:
Tests/CI:
Known limitations:
Out-of-scope work intentionally not implemented:
Any blockers:
```
