## Context

Remote job status already reports phases, recent failures, transfer facts, trust basis, output refs, and bounded cursors. Coordinator persistence currently embeds mutable log vectors in the main state snapshot. The root package uses `tracing`; vendored `snix-tracing` has optional OTLP support, but Mantle has no provider-neutral remote event contract or supported Prometheus/OTLP operator surface.

## Decisions

### 1. Bind logs to fenced attempts

**Choice:** Every log/event record names schema, job id, attempt id, fence generation, sequence/cursor, phase, stream/kind, payload length, payload BLAKE3, previous-record BLAKE3, redaction/truncation flags, and record BLAKE3.

**Rationale:** Job-only cursors cannot distinguish superseded writers. Attempt/fence binding lets the same stale-report kernel protect logs.

### 2. Persist immutable segments, not mutable vectors

**Choice:** The shell writes bounded immutable segment objects named by BLAKE3 and atomically advances a small per-attempt manifest. Coordinator state stores only bounded head/retention summaries. A segment is never edited after publication.

**Rationale:** Immutable segment identity makes restart, replay, duplicate delivery, and tamper detection explicit without requiring PostgreSQL or S3.

### 3. Make truncation explicit and verifiable

**Choice:** Retention creates a new truncation-anchor record that binds the dropped sequence range, prior head digest, dropped byte/chunk counts, policy identity, and new retained start cursor. Old segments may be deleted only after that anchor and manifest update are durable.

**Rationale:** Silent vector eviction breaks the integrity story. An anchor preserves bounded evidence that data was intentionally dropped without retaining unbounded payload.

### 4. Keep log decisions pure

**Choice:** Pure functions validate append order, fence/current-attempt facts, previous digest, event id, duplicate/conflict behavior, cursor requests, replay slices, retention plans, redaction classification, and stable diagnostics. Filesystem and transport code only applies accepted plans.

**Rationale:** Log correctness should be testable without storage, sockets, clocks, or collectors.

### 5. Treat log payload as untrusted diagnostic bytes

**Choice:** Log text never drives protocol state, authorization, retry, output admission, or scheduling. Rendering escapes control data, applies size/redaction policy, and keeps protocol stdout separate.

**Rationale:** A builder controls its output and must not smuggle control messages through logs.

### 6. Define one canonical telemetry event model

**Choice:** Mantle emits normalized events for route decision, queue admission, assignment, attempt transition, retry/fence rejection, execution result, transfer progress/completion, output admission, and publication. Events use stable reason/phase classes and bounded attributes. Exporters consume this model; they do not define it.

**Rationale:** Prometheus and OTLP should be adapters over Mantle semantics, not new runtime owners.

### 7. Bound metric cardinality

**Choice:** Metrics use declared low-cardinality labels such as phase, result class, transfer mode, retry class, route class, and worker capability class. Raw job/attempt ids, store paths, output names, trace ids, tickets, key material, and arbitrary error text stay in logs/traces or bounded exemplars, not metric labels.

**Rationale:** Unbounded labels can turn observability into a denial-of-service vector.

### 8. Propagate trace context as correlation only

**Choice:** Supported transports carry validated bounded W3C trace context separately from build identity. Invalid context is dropped and diagnosed; trace context never authorizes a client, selects a cache key, fences an attempt, or admits output.

**Rationale:** Trace correlation is useful across client/coordinator/worker but is not authority.

### 9. Isolate exporter availability from build truth

**Choice:** Prometheus serving and OTLP export are optional shell adapters configured through typed provider-neutral data. Export failure increments bounded diagnostics and may block an explicitly requested observability-evidence claim, but it does not turn an admitted build into failure or a failed build into success.

**Rationale:** A collector outage must not become hidden scheduler or trust state.

## Functional Core / Imperative Shell

- **Core**: record canonicalization, digest-chain validation, append/idempotency decision, cursor/replay plan, truncation plan, redaction class, telemetry normalization, metric-label admission, trace-context shape validation, and stable non-claims.
- **Shell**: segment file/CAS writes, atomic manifest updates, retention deletion, transport extraction/injection, tracing spans, metric exposition, OTLP network export, clock reads, and operator rendering.

## Risks / Trade-offs

- Hash chaining detects mutation but is not a signature or trusted timestamp; reports must not overclaim.
- Retention anchors preserve integrity metadata, not dropped payload content.
- Optional exporter dependencies can increase build size, so features and Nix package profiles must remain explicit.
- High event volume requires bounded batching and backpressure; telemetry must drop or degrade according to policy rather than block coordinator safety paths indefinitely.
