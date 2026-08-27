# GLM 5.3 Work Order Handoff Protocol v1.1

## Selection

The Architect selects exactly one eligible Work Order. Eligibility means: status `READY`, all dependencies verified `VERIFIED`, and parent checkpoint open for the item.

## Agent startup

Z.ai/GLM 5.3 must:

1. read all authority documents listed in the Work Order;
2. inspect current HEAD and working tree;
3. confirm dependency commits/tests;
4. restate the Work Order objective and forbidden scope internally;
5. make no repository change until all required prerequisites are present.

## Implementation

One branch, one active implementation PR. The agent may not edit frozen specifications, future Work Orders or reviewer records.

## Completion

The agent must report files, tests, evidence identifiers, limitations and blockers. Completion claims without evidence are invalid.

## Architect checkpoint

The Architect reviews the PR, evidence and exact diff. Approval does not occur through the agent. After merge, the checkpoint is evaluated and only then is the next Work Order eligible.
