# Requirements / Work Order / Evidence Traceability v1.1

Every requirement has one or more acceptance criteria, one primary Work Item, and one explicit evidence class. No requirement may be satisfied only by an agent claim.

| Requirement | Primary Work Item(s) | Evidence required |
|---|---|---|
| CAD-001 | W002,W005,W006,W007 | unit + property + integration |
| CAD-002 | W004,W005 | property + interaction benchmark |
| CAD-003 | W003,W008,W015 | integration + round-trip |
| CAD-004 | W008,W009 | visual + PDF/plot regression |
| CAD-005 | W006,W007 | deterministic replay suite |
| ELEC-001 | W016,W017,W018 | graph invariants + integration |
| ELEC-002 | W017 | cross-view consistency suite |
| ELEC-003 | W019 | catalog persistence + assignment tests |
| ELEC-004 | W020,W021 | deterministic project replay |
| ELEC-005 | W022 | validation fixture suite |
| INTEROP-001 | W010,W011,W012 | corpus import report |
| INTEROP-002 | W012,W013,W014 | data-loss diagnostics + negative fixtures |
| INTEROP-003 | W013,W014 | round-trip + visual + reopen |
| AI-001 | W023 | schema/adversarial tests |
| AI-002 | W006,W023 | architectural static checks + runtime denial tests |
| AI-003 | W023 | provenance/explanation tests |
| PERF-001 | W004,W005,W024 | reference benchmark |
| GOV-001 | all | frozen-spec and PR governance checks |

## Evidence rule

Evidence must identify the exact command, fixture, test, benchmark, artifact hash or CI run used to support the criterion. “Works locally” or “reviewed” is not sufficient evidence.
