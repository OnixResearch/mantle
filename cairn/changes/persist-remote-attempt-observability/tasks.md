## Implementation

- [ ] [serial] [depends:fence-durable-remote-attempts] r[remote_builds.immutable_attempt_log_segments] Inventory current log append/replay/retention/state paths and define migration treatment for mutable legacy job logs.
- [ ] [depends:log-inventory] r[remote_builds.immutable_attempt_log_segments] Add bounded versioned log/event, segment-manifest, and truncation-anchor DTOs bound to job, attempt, fence, sequence, payload/previous/record BLAKE3 identities, and named policy.
- [ ] [depends:log-dtos] r[remote_builds.pure_log_cursor_kernel] Implement pure canonicalization, append/idempotency/conflict, chain, cursor/replay, redaction, and retention-plan kernels.
- [ ] [depends:pure-log-kernel] r[remote_builds.immutable_attempt_log_segments] Replace mutable coordinator log vectors with shell-owned immutable segment writes and atomic bounded manifests; retain only bounded head/retention summaries in coordinator state.
- [ ] [depends:immutable-log-shell] r[remote_builds.immutable_attempt_log_segments] Add explicit truncation anchors and delete expired segments only after the anchor/manifest transition is durable.
- [ ] [depends:fence-durable-remote-attempts] r[operator_diagnostics.remote_execution_telemetry] Define canonical bounded telemetry events and low-cardinality metric descriptors for route, queue, assignment, retry/fencing, execution, transfer, admission, and publication.
- [ ] [depends:prioritize-lazy-build-goals] r[operator_diagnostics.remote_execution_telemetry] Instrument scheduler priority/dispatch events without exposing raw goal/store/provider identifiers as metric labels.
- [ ] [depends:complete-resumable-remote-cas-transfer] r[operator_diagnostics.remote_execution_telemetry] Instrument transfer demand, credit, resume, cutoff, fallback, and admission events using actual mode and byte accounting.
- [ ] [depends:telemetry-model] r[remote_builds.diagnostic_trace_context] Add bounded W3C trace-context injection/extraction to supported remote bindings while keeping trace data outside identity, authorization, fencing, scheduling, and trust decisions.
- [ ] [depends:telemetry-model] r[operator_diagnostics.telemetry_exporter_isolation] Add optional Prometheus and OTLP shell adapters plus typed Nickel configuration, bounded queues/batches, redacted diagnostics, and explicit disabled defaults.
- [ ] [depends:telemetry-exporters] r[operator_diagnostics.remote_execution_telemetry] Extend remote status/build reports with immutable-log head/cursor/truncation facts and telemetry/exporter health summaries plus diagnostic non-claims.

## Verification

- [ ] [depends:pure-log-kernel] r[remote_builds.pure_log_cursor_kernel] Add table/property/Kani tests for canonical record identity, sequence monotonicity, chain continuity, duplicate idempotency, conflict detection, cursor bounds, truncation planning, and overflow-safe accounting.
- [ ] [depends:immutable-log-shell] r[remote_builds.immutable_attempt_log_segments] Positive: append across restart, replay from a retained cursor, apply retention, and verify immutable segment/head/truncation-anchor identities.
- [ ] [depends:immutable-log-shell] r[remote_builds.immutable_attempt_log_segments] Negative: tamper with a segment, previous digest, sequence, current fence, event id, cursor, or truncation anchor and prove fail-closed replay without coordinator control-state mutation.
- [ ] [depends:telemetry-model] r[operator_diagnostics.remote_execution_telemetry] Negative: submit raw paths, job ids, output names, bearer-like text, arbitrary errors, and unbounded labels and prove metric-label admission rejects or redacts them deterministically.
- [ ] [depends:diagnostic-trace-context] r[remote_builds.diagnostic_trace_context] Positive and negative: propagate valid context end to end, drop malformed/oversized context, and prove trace values never change authorization, priority, cache identity, fence, or output admission.
- [ ] [depends:telemetry-exporters] r[operator_diagnostics.telemetry_exporter_isolation] Positive and negative: scrape/export bounded fixtures, then stop or backpressure collectors and prove builds retain their original result while exporter health reports degradation.
- [ ] [depends:observability-verification] r[operator_diagnostics.remote_execution_telemetry] Run focused remote/log/tracing tests, local multi-process restart and collector-outage fixtures, Cairn validate, and proposal/design/tasks gates; preserve logs/telemetry as diagnostic evidence only.
