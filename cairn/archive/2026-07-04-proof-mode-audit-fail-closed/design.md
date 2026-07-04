## Context

Hermeticity audit events are already typed in several paths. The missing seam is proof-mode admission: a proof consumer should not have to know every event class to decide whether evidence is eligible. A pure classifier can turn event sets and policy into accepted, downgraded, or blocked decisions.

## Decisions

### 1. Audit policy is closed by default

**Choice:** Proof mode treats every typed hermeticity audit event as blocking unless policy explicitly classifies it as informational or as an allowed downgrade for the selected proof class.

**Rationale:** New event classes should fail safe until reviewed.

### 2. Allowed downgrades are visible

**Choice:** Reports include the event class, policy basis, affected proof class, and resulting narrower claim for every allowed downgrade.

**Rationale:** Operators need to see exactly why evidence is weaker.

### 3. Practical warnings stay non-admissible

**Choice:** Practical-mode audit warnings can appear in build reports, but proof admission ignores their outputs except as diagnostic context.

**Rationale:** Continuing after a warning is useful locally, not proof of hermetic execution.

## Risks / Trade-offs

- Introducing new benign audit events will initially block proof admission until policy is updated.
- Report schemas need stable event names so downstream verifiers do not rely on prose.
