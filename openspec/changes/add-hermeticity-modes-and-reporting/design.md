# Design: Add hermeticity modes and reporting

## Context

Crunch already has both practical and proof-oriented use cases, but the code
base does not expose that split as a first-class decision. Operators get one
execution path with a mix of hard errors, warnings, and quiet recovery.

That is bad API shape. Later hardening work needs a stable selector and a
shared reporting model before individual fallback paths can be tightened.

## Goals / Non-Goals

**Goals:**

- add one explicit hermeticity selector to build-entry commands
- make pipeline and self-build code carry that selector unchanged
- define a typed audit-event model for degraded execution facts
- expose those facts in human and JSON build reporting

**Non-Goals:**

- implement every strict-mode blocker in this change
- normalize env vars or umask in this change
- redesign attestation formats

## Decisions

### 1. One small shared mode enum

**Choice:** add a shared `HermeticityMode` enum with at least `Practical` and
`Strict` variants.

**Rationale:** later changes need one stable selector, not a pile of ad hoc
booleans.

### 2. Audit events are typed, not free-form strings

**Choice:** degraded execution facts are recorded as typed audit events with a
stable machine-readable kind and a human-readable detail string.

**Rationale:** JSON build reports and future tests need a stable shape.

### 3. Reporting lands before enforcement

**Choice:** mode plumbing and reporting land before stricter enforcement.

**Rationale:** that lets later changes upgrade specific events from
report-only to hard-fail without inventing a second reporting path.

## Risks / Trade-offs

**More report surface**
Build reports get extra fields. That is acceptable because operators need this
truth.

**Strict mode starts small**
Initial strict-mode behavior will only become fully meaningful once later
changes mark more events as blockers. That is fine because this step is the
plumbing layer.
