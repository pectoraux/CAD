# Execution Governance State

`docs/execution/work-order-state.json` is mutable execution state. It is **not** architectural authority and does not replace the frozen specification.

Frozen Work Orders retain their initial planning status and may not be edited during implementation. The Architect changes lifecycle state here when a Work Order becomes active or verified.

Rules:

- exactly one Work Order may be active at a time;
- an active Work Order must have all dependencies verified;
- a Work Order may become verified only after all dependencies are verified;
- the Work Order checkpoint must be OPEN while it is active;
- implementation PR titles must contain the active Work Order ID, such as `WORK-001`;
- agents must not modify this state file as part of an implementation Work Order;
- Architect/reviewer administration is responsible for activation, verification, and checkpoint advancement.

The repository governance checks validate this state on every CI run and on implementation pull requests.
