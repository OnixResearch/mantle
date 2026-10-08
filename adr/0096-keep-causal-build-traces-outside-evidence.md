# ADR 0096: Keep causal build traces outside build evidence

## Status

Proposed (2026-10-01). Runtime traces are diagnostic observations, not a proof of complete scheduler history or successful output admission.

## Context

Current build logs interleave concurrent derivations; aggregate reports retain terminal outcomes; release receipts bind reproducible stored facts; remote trace contexts identify attempts but not local scheduler causes. Reconstructing why a leaf failed is manual and error-prone. Putting a transient action stream into a receipt would make receipt bytes depend on interleaving and caching.

## Decision Drivers

- Retain existing scheduling and receipt bytes with tracing enabled or disabled.
- Identify actual root requirements, dependency edges, dispatches, cache admissions, and cleanup without guessing from log text; type retry and cancellation only when those transitions really occur.
- Fail closed on unknown, missing, dangling, cyclic, oversized, or sensitive cause data.
- Permit deterministic review of concurrent interleaving without claiming full causal completeness.

## Decision

An opt-in `mantle-build-trace-v1` artifact lives beside ordinary diagnostic files. Its records use sequential action IDs scoped to one invocation, a redacted BLAKE3 goal identity, a closed action kind and cause vocabulary, and an explicit `caused_by` action ID. The unique external-trigger seed self-references; every other record must name an earlier present action with a valid typed edge. Shared dependencies name the first observed triggering root; the trace does not assert that no other roots depended on them. A separate `#![no_std]` core with `alloc` validates canonical order, identity edges, redaction, and byte/record/action bounds and walks chains. The imperative scheduler emits events; its observation must never control scheduling or output admission. The artifact is excluded from report, transcript, receipts, attestations, and release inputs. Evidence validators reject it as an unsupported schema.

A same-goal action keeps its parent's goal identity. A propagated
cross-goal failure names its immediately failed dependency and is admitted
only when that exact dependent/dependency pair appeared in an earlier
dependency-ready record. Merely observing another failed goal first cannot
create a dependency between independent parallel roots.

Ordinary local `mantle build --causal-trace` has no scheduler retry or
cancellation transition. Watch-mode cancellation is a separate, untraced
flow. Their closed-vocabulary kinds are validated for observed producers;
tracing never invents such actions to fill a diagnostic history.

## Alternatives Considered

- Parse logs: rejected because free-form text is interleaved and drops cause identities.
- Extend the aggregate report: rejected because the report is a terminal snapshot, not an ordered diagnostic stream.
- Event-source the scheduler: rejected because it would change scheduling authority for an observation-only request.

## Consequences

A failed or overbound trace is an explicit diagnostic failure, not an invented chain. Output artifacts and receipts remain independent of tracing. The trace describes recorded scheduler observations for this invocation only: it cannot prove completeness, worker honesty, execution success, or release eligibility.
