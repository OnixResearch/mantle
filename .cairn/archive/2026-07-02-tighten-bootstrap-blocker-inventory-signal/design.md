## Context

The current report-only command produced `target/bootstrap-blocker-inventory/current.{json,md}` with a valid summary, including 37 findings, 396 evidence-backed suppressions, and 0 promotion claims, but the wrapper still returned a failing status. That makes a successful inventory refresh look like a tool failure.

The actionable list is also diluted by metadata files such as checked placeholder inventory receipts. Those records are useful evidence, but they should not compete with live source blockers in the primary repair queue.

## Decisions

### 1. Keep exit semantics mode-dependent

**Choice:** Report-only mode exits successfully when report generation and serialization succeed. Enforcement mode remains fail-closed when clean-baseline requirements are violated.

**Rationale:** Operators need a non-gating inventory command for triage. CI or completion-claim gates need the stricter mode.

### 2. Classify metadata separately, do not delete it

**Choice:** The scanner keeps evidence-backed metadata visible in a dedicated section while excluding it from the primary unsuppressed actionable-finding count when a durable suppression reason exists.

**Rationale:** The evidence remains auditable without obscuring the next source repair target.

### 3. Preserve promotion-claim hard failures

**Choice:** Any detected bootstrap promotion claim remains a first-class failure in enforcement mode and remains prominent in report-only output.

**Rationale:** The purpose of the gate is to prevent trust-boundary overclaims. Cleaner dashboards must not weaken that guard.

## Risks / Trade-offs

- Reclassifying metadata could hide a real blocker if suppressions are too broad. Mitigation: tests must include a negative fixture where an unsuppressed source blocker with similar text remains counted.
- Exit-code changes can affect scripts that currently treat report-only failure as a blocker. Mitigation: document the mode split in the wrapper usage text and keep enforcement unchanged.
