## Why

Mantle currently keeps coordinator logs as mutable `BTreeMap<String, Vec<RemoteCoordinatorLogChunk>>` state with cursor and text fields. That supports bounded replay, but chunks are not bound to an execution attempt/fence, do not form an immutable integrity chain, and cannot distinguish duplicate append from conflicting rewrite. Mantle also lacks an owned canonical remote-execution telemetry model and supported exporter boundary even though vendored tracing code contains optional OTLP machinery.

Rio's immutable attempt logs, trace propagation, and operational telemetry are high-value patterns. Mantle should adapt them without adopting Rio's mandatory Kubernetes, PostgreSQL, S3, or dashboard stack.

## What Changes

- Add versioned per-attempt event/log records bound to job, attempt, fence, sequence/cursor, phase, stream, payload digest, previous-record digest, and BLAKE3 record identity.
- Persist append-only immutable log segments plus bounded manifests and explicit truncation anchors instead of rewriting log vectors in coordinator state.
- Extract pure append, duplicate/conflict, chain, cursor, replay, redaction, and retention decisions.
- Define a canonical low-cardinality remote-execution telemetry event/metric model covering queue, assignment, retry, execution, transfer, admission, publication, and stale-fence rejection.
- Propagate bounded W3C trace context across supported remote bindings as diagnostic correlation only.
- Add optional Prometheus and OTLP exporter adapters behind provider-neutral typed configuration; exporter failure remains separate from build/result truth.
- Preserve immutable logs and telemetry as diagnostic evidence with explicit non-claims unless a separate policy admits a narrower receipt.

## Impact

- **Surfaces**: remote coordinator state/log storage, protocol frames, build/status reports, tracing initialization, remote farm Nickel config, operator docs, and local multi-process fixtures.
- **Dependencies**: attempt identity/fencing depends on `fence-durable-remote-attempts`; complete scheduler/transfer metrics additionally consume `prioritize-lazy-build-goals` and `complete-resumable-remote-cas-transfer` events.
- **Non-claims**: no hosted dashboard, mandatory collector, telemetry-as-authority, log-derived output trust, CI UI, or Valence semantic ownership.
- **Validation**: chain/cursor property and Kani tests, restart/truncation/tamper fixtures, redaction and cardinality negatives, exporter-outage tests, trace propagation tests, focused remote diagnostics checks, and Cairn gates.
