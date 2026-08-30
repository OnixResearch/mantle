# ADR 0088: Derive Rust-provider aggregate event bounds from stage bounds

## Status

Accepted (2026-08-30)

## Context

Rust-provider action authority defines `exec_events_per_stage_max`. Every
stage reconciliation rejects an event count above that limit.

The aggregate reconciliation also compared the sum of all stage events to the
same single-stage limit. V85 completed five local Rust-provider stages with
complete reconciliation:

- MRustC to Rust 1.90.0: 88,038 events;
- Rust 1.91.1: 75,843 events;
- Rust 1.92.0: 76,071 events;
- Rust 1.93.1: 75,272 events;
- Rust 1.94.0: 75,983 events.

Each stage stayed below the 262,144-event bound. Their valid aggregate was
391,207 events. The aggregate comparison rejected it as if all five stages
were one stage.

## Decision Drivers

- Preserve the existing per-stage event limit.
- Keep the aggregate bounded by the authenticated route shape.
- Do not replace the limit with an observed magic number.
- Keep overflow and missing-stage behavior fail closed.
- Keep aggregate evidence compatible with existing schemas.

## Decision

Derive the aggregate event maximum as:

```text
exec_events_per_stage_max * declared_stage_count
```

The multiplication uses checked `u32` arithmetic. Zero factors and arithmetic
overflow fail closed.

The declared stage count comes from the bounded Rust-provider route. Runtime
startup rejects empty routes, duplicate stages, and routes above the existing
stage-count limit. Runtime finish rejects missing stages before aggregate
construction. Unknown or repeated stages remain rejected at stage start.

Each stage must still pass its own event bound before it can enter the
aggregate. The aggregate count must equal the sum of all completed stage
reconciliations and the raw aggregate audit count.

No evidence schema changes. Aggregate plans already bind the action count and
every stage-plan digest. Stage plans already bind the per-stage limit.

## Alternatives Considered

### Raise the single limit to 391,207

Rejected. This copies one observation into policy and weakens every stage.

### Remove the aggregate check

Rejected. Checked derivation keeps an explicit total bound without weakening
the stage bound.

### Add a separately configured aggregate limit

Rejected. Two independent limits can drift. The route already supplies the
bounded number of stages.

## Consequences

- A valid multi-stage provider can exceed one stage's event limit in total.
- Adding a declared stage increases the aggregate maximum by exactly one
  per-stage allowance.
- A single stage cannot consume another stage's unused allowance.
- V85 remains failed evidence. A fresh promoted run must re-establish the
  aggregate evidence and continue to checkpoint publication.
