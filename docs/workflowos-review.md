# WorkflowOS Workflow Review and Adaptation

## What we are adopting

WorkflowOS's frozen architecture establishes a strong governance pattern: architecture is frozen; work is decomposed into dependent Work Items; execution agents do not own workflow state; evidence is required for completion; architectural changes require an explicit change request; Work Orders contain explicit scope, acceptance criteria, stop conditions and a definition of done. (source: https://github.com/pectoraux/WorkflowOS/blob/main/spec/architecture-lock.md)

Its actual Z.ai Work Order pattern reinforces this: the agent is instructed to read the frozen specification before code changes, not redesign the architecture, operate within explicit module boundaries, provide concrete test/evidence output, and escalate architectural blockers instead of inventing alternate designs. (source: https://github.com/pectoraux/WorkflowOS/blob/main/docs/work-items/WORK-004-zai-prompt.md)

## What we keep

- frozen architecture versions;
- dependency-ordered Work Items;
- bounded Work Orders;
- explicit acceptance criteria;
- evidence over claims;
- CI as evidence, not authority;
- independent architect review;
- explicit architecture-change escalation;
- no direct LLM control of authoritative state.

## What we add for CAD

WorkflowOS is optimized for software systems. CAD needs additional domain-specific gates:

1. **Visual evidence** — rendered geometry can fail while unit tests pass.
2. **Interoperability evidence** — file open/save correctness needs real corpus testing.
3. **Geometry robustness** — degenerates and floating-point edge cases require property/fuzz testing.
4. **Performance gates** — pointer-move, selection and snapping are interactive hot paths.
5. **Data-loss gates** — import/export must report unsupported content explicitly.
6. **Checkpoint certification** — each architectural layer must have a runnable acceptance harness before dependent layers proceed.

## Recommended operating loop

```text
Architect freezes spec
        |
Create dependency-ordered Work Items
        |
Generate one bounded GLM 5.3 Work Order
        |
GLM implements on isolated branch
        |
CI + domain verification
        |
Architect reviews scope + evidence
        |
  +----+----+
  |         |
APPROVED  CHANGES_REQUESTED
  |         |
merge     new implementation cycle
  |
checkpoint
  |
next eligible Work Item
```

## Important adaptation

We should **not** copy WorkflowOS's software-project domain model into the CAD product. We are copying its implementation governance pattern. The CAD repo remains a product repo with its own canonical architecture.
