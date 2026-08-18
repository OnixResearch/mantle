## Context

Direct client-to-builder dispatch does not scale across worker pools or network boundaries where workers must dial out. The existing remote-builds spec names coordinator scheduling; this change turns that model into a concrete runtime milestone with durable state and restart semantics.

## Decisions

### 1. Workers register before work assignment

**Choice:** Workers initiate coordinator sessions and advertise endpoint identity, systems, feature labels, sandbox modes, network modes, concurrency, output signing-key identities, transfer capabilities, and resumable job summaries.

**Rationale:** The coordinator can match only facts the worker explicitly declares.

### 2. Normalized build keys dedupe jobs

**Choice:** The coordinator keys queued/running/finished-undelivered jobs by normalized concrete build identity, excluding attempt-specific transport ids, temp paths, and log cursors.

**Rationale:** Duplicate clients should attach to one job; conflicting output claims should fail before dispatch.

### 3. Coordinator schedules resources but not trust

**Choice:** The coordinator may enforce resource and queue policy, but clients still verify returned outputs through their own trust roots.

**Rationale:** Compromising the scheduler must not make untrusted outputs importable.

## Risks / Trade-offs

- Durable queue state must avoid retaining secrets while preserving enough data to resume or report phase loss.
- Worker re-registration can race with client resubmission; normalized keys and leases need precise conflict handling.
