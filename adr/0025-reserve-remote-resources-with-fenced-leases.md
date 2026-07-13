# ADR 0025: Reserve remote resources with fenced leases and verified locality

## Status

Accepted (2026-07-12)

## Context

Mantle already has deterministic lazy ready-goal ordering, hard route gates, durable remote attempt fencing, receiver-driven transfer missing sets, and clone-persist coordinator mutations. Its resource and locality inputs are still ordinal classes or a boolean gate. They cannot prove aggregate capacity, token ownership, or the receiver facts behind a locality preference.

Inferring capacity from worker concurrency would fabricate CPU, memory, scratch, accelerator, or named-token availability. Trusting worker locality advertisements would allow stale or adversarial claims to become zero-transfer evidence. Reusing the ready-goal comparator by inventing known-graph pressure for worker candidates would also corrupt scheduler evidence.

## Decision Drivers

- Preserve Mantle's lazy goal scheduler and deterministic starvation protection.
- Keep hard capability, trust, upload, network, prefix, and resource admission ahead of preference.
- Keep clocks, probes, persistence, locks, and rendering outside the pure core.
- Commit one durable resource reservation before returning a remote assignment.
- Bind mutations to current job, attempt, fence, and worker identity.
- Derive locality only from canonical transfer scope and receiver-probed presence.
- Never infer capacity, provider order, graph pressure, or transfer savings.
- Keep scheduling quantities out of action identity while retaining semantic execution differences.

## Decision

Mantle adds a pure provider-neutral resource/locality module under `crunch-build::distributed`. It canonicalizes bounded CPU, memory-byte, scratch-byte, accelerator, and named-token vectors; rejects zero, duplicate, invalid, or oversized quantities; uses checked arithmetic for aggregate reservations; and produces deterministic admission, fit, mutation, and recovery plans.

Resource requirements separate scheduling quantities from explicit semantic accelerator classes. Accelerator class names must match between the two views. Counts and dynamic availability do not enter the normalized action key; semantic accelerator classes do.

The coordinator persists active resource leases in the same durable state as jobs. A lease binds worker endpoint and generation, job, attempt, fence, requirement digest, reservation digest, and exact reserved vector. Dispatch clones state, plans against all active leases, installs the current attempt and lease, persists the candidate, and only then returns the assignment. Reassignment atomically removes the superseded lease and installs the replacement. Terminal report, cancellation, timeout, and worker-loss transitions release only the current matching lease. Stale or unknown fences cannot renew, resize, release, or complete a current reservation.

Legacy workers and requests may omit quantified resource fields. Their resource class is `unknown`; a quantified request cannot dispatch to an unquantified worker. Mantle never converts concurrency into capacity.

Locality summaries are created by an imperative receiver probe that supplies a canonical transfer manifest and receiver facts to the pure core. The core re-runs missing-set planning and binds the result to worker identity/generation, manifest digest, and policy digest. Worker generation is also the freshness fence for destructive receiver-cache transitions: workers must advance it before restart, eviction, or any other change that can invalidate a positive presence fact. Additive cache changes may conservatively underclaim until the next probe. Full locality and zero transfer require a current verified complete set. Unverified hints remain unknown and cannot contribute reused bytes.

Worker placement evaluates every hard route gate first, then compares admitted resource fit and verified locality/transfer classes according to configured scheduling precedence, followed by stable worker identity. It does not create ready-goal graph facts. Goal starvation and known-graph ordering remain exclusively owned by the existing lazy scheduler.

Reports expose bounded counts, bytes, digests, classes, lease scopes, and stable reason codes. They do not expose resource credentials, license material, object lists, provider secrets, or unbounded paths. Named tokens authorize scheduling capacity only and never establish tool identity, legal compliance, output trust, attestation validity, or release eligibility.

## Alternatives Considered

### Infer resource capacity from concurrency

Rejected because one build slot has no honest conversion to CPU, memory, scratch, accelerator, or token capacity.

### Keep reservations only in process memory

Rejected because concurrent/restarted coordinators could overcommit or lose ownership.

### Trust worker-provided locality classes or saved-byte claims

Rejected because advertisements can be stale or adversarial and do not prove receiver presence.

### Add resource fields directly to the action hash

Rejected for scheduling-only quantities because dynamic availability and requested parallel capacity should not churn cache identity. Explicit semantic accelerator classes remain hashed.

### Reuse the ready-goal comparator with synthetic graph pressure

Rejected because synthetic pressure would be fabricated evidence. Worker placement consumes only worker-placement facts; the existing scheduler remains authoritative for goals and starvation.

## Consequences

- Quantified requests require explicit operator or worker inventory; missing capacity is a hard quantified-resource blocker rather than a guessed default.
- Coordinator state gains bounded lease and verified-locality records and stricter restart validation.
- Capacity shrink during registration is rejected while active leases exceed the new total.
- Locality probes add bounded metadata work and must be refreshed when worker generation, manifest, or policy changes.
- Placement evidence proves only deterministic ordering over current admitted facts. It does not prove global makespan optimality, future availability, provider autoscaling, execution success, output trust, license compliance, or release reproducibility.
