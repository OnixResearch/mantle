## Context

`crunch-build::scheduling` owns deterministic ranking over provider-neutral classes and correctly evaluates hard eligibility before preference. `crunch-build::distributed` can normalize remote route facts, but its resource gate is a boolean and the scheduler receives ordinal classes. The remote-builder Nickel profile currently declares system, sandbox/network modes, feature labels, concurrency, upload bytes, build time, retry policy, and transfer policy. It does not model quantified capacity or scarce token ownership.

This change supplies reviewed facts to the existing comparator. It does not introduce a second scheduler or an eager global DAG.

## Decisions

### 1. Model bounded provider-neutral resource vectors

**Choice:** An action or derivation may declare scheduling requirements for CPU units, memory bytes, scratch bytes, accelerator requirements, and bounded named token quantities. A worker advertises corresponding capacities plus current fenced reservations. All names, counts, and arithmetic have explicit limits and checked operations.

**Rationale:** A boolean resource match cannot prevent aggregate overcommit or explain scarce-resource rejection.

### 2. Separate semantic capabilities from scheduling quantities

**Choice:** Platform, toolchain, sandbox, network, accelerator semantics, and other facts that can change execution meaning remain in action identity. Scheduling-only quantities such as requested parallel capacity or a license-seat reservation do not change action identity unless explicitly declared as semantic action fields. Reports identify the classification.

**Rationale:** Dynamic worker availability must not churn cache identity, while semantic execution differences must not be hidden as scheduler metadata.

### 3. Reserve atomically with fenced leases

**Choice:** The pure core plans reservation from an immutable inventory snapshot and returns an accept/reject plan plus resulting capacities. The coordinator shell commits one job/attempt/fence-bound lease before assignment. Release, expiry, worker loss, and restart adoption use explicit transitions; stale attempts cannot renew or release a current lease.

**Rationale:** Capacity checks without atomic reservation race under concurrent dispatch.

### 4. Treat named tokens as resource authorization only

**Choice:** Named token pools model scarce capacity such as licensed-tool seats. Records contain opaque resource-class names and quantities, never license credentials. Holding a token authorizes scheduling only and does not establish tool identity, output trust, or license compliance.

**Rationale:** Scheduling authority and build-result trust are separate boundaries.

### 5. Derive locality from receiver-verified presence

**Choice:** Worker locality facts are produced from bounded probes of declared input object refs and the production transfer missing-set planner. Summaries bind manifest/policy identity, verified present and missing object counts/bytes, and freshness generation. Unverified worker hints may guide probing but cannot become `FullyPresent` or zero-transfer evidence.

**Rationale:** Self-reported locality can be stale or adversarial.

### 6. Normalize facts into the existing comparator

**Choice:** Concrete resource and locality planning yields existing hard eligibility plus resource-fit, content-locality, and transfer-cost classes. The existing configured lexicographic scheduler and starvation rules remain authoritative.

**Rationale:** The missing feature is fact derivation and reservation, not another priority algorithm.

### 7. Keep placement deterministic and bounded

**Choice:** Equivalent eligible workers are ordered by explicit resource fit, verified transfer class, stable worker identity, and existing policy. Dynamic inventory affects only later scheduling snapshots. Reports do not claim global makespan or future availability.

**Rationale:** Provider response timing and wall-clock races must not decide placement.

## Functional Core / Imperative Shell

- **Core**: requirement/inventory validation, checked resource subtraction/addition, reservation/release/recovery plans, semantic-versus-scheduling classification, verified-locality summary normalization, eligibility and preference classes, and stable diagnostics.
- **Shell**: worker probes, coordinator state persistence, lease commit/release, clocks/timeouts, transfer manifest probes, provider adapters, policy loading, and report rendering.

## Risks / Trade-offs

- Coarse resource declarations are easier to operate but can underutilize workers; exact values remain operator policy rather than hard-coded defaults.
- Durable reservations need conservative recovery to avoid both overcommit and leaked capacity.
- Locality probes cost metadata I/O; bounded summaries and generation-based freshness avoid unbounded scans.
- Named tokens can represent license capacity but cannot prove legal entitlement or license-server health.
