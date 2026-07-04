## Context

Remote-build support is currently split between accepted primitives and a sidecar execution path. Existing work covers bounded protocol frames, stdio/SSH transport semantics, ticket authorization, concrete request validation, source-bundle-backed input upload, signed output admission, route-plan diagnostics, and pure coordinator scheduling functions. The production gap is composition: the lazy scheduler must own remote placement, the coordinator must persist live state, transfer must scale beyond inline frames, and output trust must verify cryptographic key material before any remote result becomes local store state.

Hydra and Hercules CI both center distributed work around a durable scheduler. Mantle should do the same without copying their CI/jobset model wholesale. The coordinator is a build resource scheduler, not a CI orchestrator or trust root; the client still admits outputs through Mantle store and attestation policy.

## Decisions

### 1. Scheduler integration is the production boundary

**Choice:** Remote realization becomes a scheduler-compatible build service / realizer selected by route policy for ready goals. The CLI may still expose direct stdio tools for diagnostics and bootstrap, but production builds go through the same lazy goal state machine as local builds and substitutions.

**Rationale:** This preserves goal dedupe, waiter notification, `-j` limits, dependency interleaving, local fallback policy, and existing build reports. A separate remote loop would recreate a competing scheduler and drift from local build semantics.

### 2. Remote workers execute concrete Mantle inputs only

**Choice:** The client or frontend lowers build intent into concrete Mantle derivations/actions before dispatch. Builders reject raw Nickel, Onix module semantics, package-manager cache discovery, or undeclared sibling checkout reads.

**Rationale:** Mantle's advantage over generic CI is that remote execution is auditable and frontend-neutral. The remote worker should not become a second evaluator with hidden imports or ambient source access.

### 3. CI orchestration stays outside the build farm

**Choice:** Mantle exposes remote build, status, cache, transfer, and receipt surfaces for external CI systems to call, but it does not own jobsets, pipelines, webhook triggers, branch/PR policy, checkout discovery, or frontend evaluation scheduling inside the build farm.

**Rationale:** Coupling CI to building would blur Mantle's strongest boundary. CI decides what should be built and when; Mantle decides how a concrete build request is realized, verified, cached, and reported.

### 4. Output trust uses real cryptographic verification

**Choice:** Remote output admission verifies PathInfo signatures against configured public key material, validates store prefix and output identity, checks object refs and artifact-attestation digests, and records the trust basis. Builder tickets and coordinator assignment authorize resource use only.

**Rationale:** This matches Mantle's existing binary-cache trust boundary and avoids the common farm mistake where “this machine was allowed to build” becomes “this output is trusted.”

### 5. Transfer is streaming, digest-addressed, and resumable

**Choice:** Production transfer uses chunked CAS/NAR/delta streams keyed by BLAKE3 content identities, with NAR SHA-256 retained only where Nix-compatible formats require it. Inline JSON frames remain fixture/bootstrap convenience only.

**Rationale:** Hydra and Hercules rely heavily on binary-cache/store transfer. Mantle needs equivalent throughput while preserving source-bundle and delta semantics. Large outputs must not require one giant frame or full-memory buffering.

### 6. The coordinator persists queue, leases, logs, and resume summaries

**Choice:** A coordinator runtime records worker registrations, queued/running/recent jobs, live output leases, bounded log chunks, and resumable job summaries in durable state. Workers may dial out to the coordinator, allowing firewall-friendly deployments similar to Hercules agents.

**Rationale:** Restart survival and worker-initiated registration are the practical difference between a remote command runner and a build farm. The coordinator still never admits outputs by itself.

### 7. Accepted results publish after local admission

**Choice:** Once a remote result is accepted through local store and attestation admission, publisher adapters may export it to configured caches/artifact stores. Publisher errors are diagnostics and do not retroactively turn an admitted build into a failure unless policy explicitly requires publication.

**Rationale:** A build farm should amortize work. The next client should prefer cache reuse over recontacting the worker, but publication is not itself proof of correctness.

### 8. Configuration is typed and provider-neutral

**Choice:** New operator configuration uses typed Nickel contracts where Mantle owns the config format, and runtime Rust consumes explicit exported data. Concrete transports such as SSH-stdio, future P2P, S3, or REAPI stay adapter-owned.

**Rationale:** This keeps Onix/Mantle configuration reviewable while preventing provider details from leaking into core scheduling logic.

## Risks / Trade-offs

- Scheduler integration may expose old sidecar assumptions about per-root execution; tests must cover multi-root dependencies and duplicate goals.
- Streaming transfer adds complexity around partial failure, resume cursors, and garbage collection leases.
- Verifying key material can break existing operator shortcuts that trusted only key names; the migration should fail closed with clear diagnostics.
- Persistent coordinator state must be compacted and bounded so log replay and recent-job history do not become unbounded control data.
- External CI adapters will still need their own mapping from commits, jobs, or pipelines to concrete Mantle requests; that work is intentionally outside this farm boundary.
- Publication after admission can hide performance regressions if reports do not distinguish remote execution, local cache hits, trusted substitution, and post-build publication.
