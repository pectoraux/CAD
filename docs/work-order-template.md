# Work Order Template — GLM 5.3

## Identity

- Work Item:
- Title:
- Architecture version: v1.0
- Parent checkpoint:
- Dependencies:

## Authority

Read before changing code:

- `/spec/architecture.md`
- `/spec/architecture-lock.md`
- `/spec/domain-model.md`
- `/spec/commands.md`
- `/spec/requirements.md`
- `/spec/dependency-graph.md`
- `/spec/work-items.md`

These documents are authoritative. Do not redesign them.

## Objective

One bounded objective.

## Allowed changes

Explicit modules/files/subsystems.

## Required implementation

Exact behaviors and contracts.

## Acceptance criteria

Each criterion needs objective evidence.

## Required tests

Unit, property, integration, regression, visual/interoperability as applicable.

## Architecture boundaries

Explicit forbidden dependencies and forbidden mutations.

## Out of scope

List future Work Items that must not leak into this one.

## Stop conditions

If a frozen rule must change, output exactly:

`ARCHITECTURE_CHANGE_REQUIRED`

If existing repository state prevents safe implementation without architectural redesign, output exactly:

`IMPLEMENTATION_BLOCKED`

## Definition of done

- implementation complete;
- tests pass;
- typecheck/lint/build pass;
- architecture checks pass;
- no data-loss regressions;
- no out-of-scope functionality;
- evidence is concrete;
- final response follows the required evidence template.

## Final response

```text
WORK-XXX COMPLETE

Implementation summary:
Tests/evidence:
Files changed:
Architecture invariants checked:
Known limitations:
Any blockers:
```
