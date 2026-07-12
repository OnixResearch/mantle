# ADR 0013: Harden remote execution without replacing Mantle foundations

## Status

Proposed

## Context

Mantle has a lazy goal scheduler, provider-neutral realization seams, a native castore/PathInfo trust pipeline, typed remote-farm configuration, coordinator job persistence, bounded log replay, and remote protocol fixtures. The archived production-farm change nevertheless overstates one important surface: its task list marks resumable streaming transfer complete while its durable evidence says that implementation was not started, and current production-shaped paths still carry whole inline payload vectors.

Rio Build provides useful production prior art: pure authorization/retry/log/evidence kernels, pull assignments with fencing, receiver-missing CAS negotiation, critical-path/resource-aware dispatch, immutable per-attempt logs, and trace/metric propagation. Rio's full deployment architecture also owns a separate CAS and assumes Kubernetes, PostgreSQL, S3-compatible storage, and CI-adjacent services that Mantle should not adopt.

## Decision Drivers

- Preserve Mantle's authoritative castore, PathInfo, attestation, and output-admission path.
- Preserve ADR 0001's lazy graph, streaming evaluation, and dynamic goals.
- Keep scheduler and configuration provider-neutral.
- Keep resource authorization separate from output trust.
- Keep CI/jobset/pipeline semantics outside the build backend.
- Make safety-critical decisions pure, deterministic, bounded, and independently testable.
- Prefer current evidence over checked task boxes or capability labels.

## Decision

Harden remote execution through four native Cairn changes:

1. **`fence-durable-remote-attempts`** separates realization, job, and attempt identity; advances a monotonic fence on reassignment; makes reports idempotent; and extracts pure authorization/retry/transition kernels.
2. **`complete-resumable-remote-cas-transfer`** reuses Mantle castore and delta identities, adds receiver-driven chunk demand, credits, checkpoints, safe resume, and verified content-presence cutoff, and repairs the archived completion contradiction with superseding evidence.
3. **`prioritize-lazy-build-goals`** replaces FIFO ready ordering with a pure deterministic priority tuple over known-graph critical-path pressure, resource fit, locality, starvation class, and stable identity while retaining the lazy scheduler.
4. **`persist-remote-attempt-observability`** stores fenced immutable BLAKE3-chained log segments, defines a canonical bounded telemetry model, propagates diagnostic trace context, and keeps Prometheus/OTLP as optional shell adapters.

Across all four changes, Mantle uses functional cores over immutable DTOs and thin imperative shells for clocks, files, transport, process control, cryptography, and exporters. BLAKE3 identifies Mantle-owned manifests, chunks, checkpoints, records, snapshots, and receipts; interoperability-required NAR hashes remain unchanged.

The rollout guarantees one accepted current fenced attempt, not exactly-once physical execution. Transfer completion remains separate from output admission. Telemetry and logs remain diagnostic unless another explicit policy admits a narrower claim.

## Alternatives Considered

### Adopt Rio Build wholesale

Rejected because it would duplicate Mantle's CAS, scheduler, trust, evidence, and configuration ownership while introducing mandatory deployment dependencies that do not fit provider-neutral Onix boundaries.

### Add resumable transfer only

Rejected because retries without fenced attempts can admit stale checkpoints/results, while mutable logs cannot reliably explain which attempt produced an event.

### Replace lazy goals with Rio's eager global DAG

Rejected because Mantle intentionally overlaps evaluation and building and supports dynamic goals. Only ready-goal ordering should change.

### Let Prometheus or OTLP define runtime semantics

Rejected because exporter schemas and availability would become hidden control dependencies. Mantle owns canonical events; exporters remain adapters.

## Consequences

- Remote hardening lands in dependency order: attempt fencing first; transfer and scheduler priority may then proceed; complete observability consumes all three event surfaces.
- Existing coordinator state and logs need conservative migration or explicit legacy rejection.
- The current inline payload path remains a fixture/bootstrap compatibility mode until interruption/resume evidence proves the production replacement.
- Scheduler reports describe a known-graph estimate, not globally optimal makespan.
- Immutable log chains detect mutation but are not signatures or trusted timestamps.
- ChaosControl may consume exported fault fixtures and receipts, while Mantle retains runtime semantics and Cairn retains lifecycle readiness ownership.
