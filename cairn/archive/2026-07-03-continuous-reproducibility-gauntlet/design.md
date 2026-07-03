## Context

Mantle needs an evidence history, not only one successful release. Continuous gauntlet reports should aggregate the repeatability matrix, witnesses, Nix comparison, hermeticity adversarial tests, cache attacks, and bootstrap pressure without promoting stale or partial evidence.

## Decisions

### 1. Aggregation is evidence-indexing, not rerun logic

**Choice:** The continuous report reads signed/hashed track reports and summarizes their current status. Track-specific runners remain separate.

**Rationale:** This keeps the aggregate core deterministic and lets expensive tracks run on different schedules or hosts.

### 2. Staleness is computed from bound digests

**Choice:** A report is current only when source digest, policy digest, universe digest, toolchain digest, witness set, and track schema versions match the current claim boundary.

**Rationale:** Reusing old evidence after source or policy changes would undermine proof-before-claim rules.

### 3. Flakes and blockers are visible outcomes

**Choice:** The aggregate report tracks pass, fail, blocked, unsupported, stale, and flaky states with first/last seen run ids and next actions.

**Rationale:** Long-running evidence should guide work rather than collapse everything into a single red/green status.

## Risks / Trade-offs

- Aggregates can become noisy; keep a concise current-claim summary plus detailed per-track records.
- CI cost must be controlled with incremental profiles and scheduled heavy runs.
