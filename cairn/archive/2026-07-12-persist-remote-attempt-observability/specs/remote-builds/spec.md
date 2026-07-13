# Remote Builds Specification

## Purpose

Define immutable fenced attempt logs, pure cursor decisions, and diagnostic trace propagation for remote builds.

## Requirements

### Requirement: Remote attempt logs use immutable digest-bound segments [r[remote_builds.immutable_attempt_log_segments]]

Mantle MUST persist remote attempt logs as bounded immutable segments whose records bind schema, durable job, current attempt, fence generation, sequence/cursor, phase, stream/kind, payload length, payload BLAKE3, previous-record BLAKE3, redaction/truncation flags, and record BLAKE3. Published segment content MUST NOT be edited in place.

#### Scenario: Current attempt appends an immutable segment

- GIVEN a current fenced attempt emits bounded log records after the retained head
- WHEN Mantle accepts and persists the append plan
- THEN each record and segment MUST have deterministic BLAKE3 identity and chain to the previous retained record or explicit anchor
- AND coordinator state MUST advance only after the immutable segment and bounded manifest update are durable.

#### Scenario: Stale or tampered append fails closed

- GIVEN an append has a stale attempt/fence, regressed sequence, wrong previous digest, changed payload under an existing event id, malformed bounds, or invalid record digest
- WHEN Mantle validates it
- THEN it MUST reject the append before changing the manifest, cursor, coordinator phase, retry state, or output state
- AND diagnostics MUST use stable bounded reason codes rather than untrusted payload text.

#### Scenario: Retention is explicit

- GIVEN retention policy requires old segments to be dropped
- WHEN Mantle advances the retained start cursor
- THEN it MUST first persist a truncation anchor binding the dropped range, prior head, dropped counts, and policy identity
- AND reports MUST distinguish intentionally truncated data from never-recorded or tampered data.

### Requirement: Log cursor and retention decisions have a pure core [r[remote_builds.pure_log_cursor_kernel]]

Mantle MUST implement log canonicalization, append/idempotency/conflict classification, cursor validation, replay slicing, redaction classification, and retention planning as pure deterministic functions over explicit bounded facts. Storage, transport, clocks, deletion, and rendering MUST remain in imperative shell code.

#### Scenario: Equivalent log facts replay identically

- GIVEN equivalent retained manifests, record identities, requested cursors, policies, and current attempt/fence facts
- WHEN the pure core plans append, replay, or retention
- THEN it MUST return the same ordered plan and stable diagnostics
- AND filesystem enumeration, map order, wall-clock reads, transport timing, and exporter availability MUST NOT affect the decision.

#### Scenario: Untrusted log text cannot control execution

- GIVEN a log payload contains protocol-looking frames, authorization text, scheduler directives, terminal escapes, or output-admission labels
- WHEN Mantle records or renders the payload
- THEN the payload MUST remain untrusted diagnostic data subject to escaping/redaction
- AND it MUST NOT mutate protocol state, retry policy, priority, attempt fencing, or output trust.

### Requirement: Trace context is diagnostic correlation only [r[remote_builds.diagnostic_trace_context]]

Mantle MAY propagate bounded validated W3C trace context across supported remote bindings, but trace context MUST remain separate from realization identity, cache keys, authorization, attempt fencing, scheduling eligibility/priority, transfer identity, and output admission.

#### Scenario: Valid context correlates spans

- GIVEN a client sends valid bounded trace context through a supported binding
- WHEN the coordinator and worker create remote-build spans
- THEN Mantle MAY attach those spans to the propagated context
- AND build reports MUST still derive identity and trust from ordinary Mantle facts.

#### Scenario: Invalid context is dropped safely

- GIVEN trace context is malformed, oversized, duplicated, or contains unsupported fields
- WHEN Mantle validates the carrier
- THEN it MUST drop or reject the context according to policy with bounded diagnostics
- AND the remote request's authorization, scheduling, execution, and output-admission result MUST be unchanged by that diagnostic failure.
