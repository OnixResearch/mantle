## Context

Remote execution spreads diagnostic facts across route plans, coordinator state, worker status, build reports, transfer reports, output-admission diagnostics, and log streams. The active immutable-observability work owns append-only log persistence and telemetry; it intentionally does not define a portable failure-debug bundle or replay operation.

A useful bundle must survive worker cleanup and be inspectable without trusting the failed worker. It must also avoid turning arbitrary sandbox contents and logs into an unbounded exfiltration channel.

## Decisions

### 1. Make the bundle a manifest over immutable refs

**Choice:** `mantle-remote-failure-debug-bundle-v1` binds bundle schema/ref, action ref/spec ref, route decision ref, worker capability summary ref, job/attempt/fence, declared input-manifest ref, sandbox/network policy refs, immutable log head/range refs, transfer/admission report refs, workspace-mode summary, failure phase/reason class, optional captured-artifact manifest ref, policy digest, and non-claims. Mantle-owned identity uses domain-separated BLAKE3.

**Rationale:** Referencing admitted immutable artifacts keeps the bundle bounded and avoids copying large payloads into control metadata.

### 2. Default to metadata-only capture

**Choice:** Every eligible failure may produce a metadata-only bundle. Capturing failed-sandbox files requires explicit policy naming allowed relative paths or artifact classes, maximum files/bytes/depth, sensitivity class, retention, and whether capture failure blocks only debug evidence.

**Rationale:** Whole-sandbox capture is unsafe and can dwarf the build output.

### 3. Capture before cleanup through an explicit plan

**Choice:** The pure core decides the capture allowlist and limits from normalized failure and policy facts. The shell applies that plan after process termination and before sandbox/workspace cleanup, ingests accepted files as content-addressed debug objects, then proceeds with cleanup or quarantine.

**Rationale:** Timing is necessary operationally, but selection and claim semantics remain deterministic and testable.

### 4. Treat all failed-process material as untrusted

**Choice:** Logs, paths, filenames, diagnostics, and captured bytes cannot drive protocol or replay control. Rendering escapes control characters; bundle construction redacts or rejects secret descriptors, credentials, private-key paths, raw environment values, host absolute paths, sockets/devices, escaping links, and unsupported file kinds.

**Rationale:** A failed builder can deliberately emit malicious diagnostic material.

### 5. Replay creates a new execution

**Choice:** `inspect` validates and summarizes a bundle without executing. `replay-plan` resolves immutable declared refs and emits a new bounded execution plan. `replay` requires explicit operator execution, receives a new job/attempt/fence and route decision, and passes through ordinary policy, sandbox, transfer, and output admission.

**Rationale:** Prior attempt authority is stale and debug evidence is not a capability token.

### 6. Record replay comparison without promising reproduction

**Choice:** Replay evidence compares failure phase/reason class, exit status class, log/captured-artifact refs when policy permits, and admitted outputs. Matching or diverging facts are reported; neither outcome rewrites the original build result.

**Rationale:** Remote environments and nondeterministic failures may differ even with the same declared action.

### 7. Keep observability ownership separate

**Choice:** This change consumes immutable log and telemetry refs from `persist-remote-attempt-observability`; it does not create another log store, exporter model, or trace protocol. If immutable logs are unavailable, bundle creation reports a bounded missing-log fact rather than embedding mutable vectors as equivalent evidence.

**Rationale:** A debug package composes evidence; it should not duplicate its producers.

### 8. Root and expire debug artifacts explicitly

**Choice:** Bundle manifests and optional captured objects use policy-bound retention roots with byte/count limits. Active inspection/replay leases prevent deletion; expiration removes only accepted roots and reports cleanup failures.

**Rationale:** Debug artifacts are useful but can contain sensitive or large data and must not become permanent ambient store roots.

## Functional Core / Imperative Shell

- **Core**: eligibility, manifest canonicalization, capture allowlist/limits, sensitivity/redaction decisions, inspect summary, replay-plan construction, comparison classification, retention plans, and stable diagnostics.
- **Shell**: process/sandbox observation, file reads, CAS ingestion, log-ref resolution, atomic manifest writes, policy loading, replay execution, clocks, roots/leases, cleanup, and CLI rendering.

## Risks / Trade-offs

- Metadata-only bundles may not contain enough tool-specific detail; explicit capture expands evidence at a deliberate privacy/storage cost.
- Redaction cannot make arbitrary binary artifacts safe, so capture policy is allowlist-based and defaults off.
- Replay may diverge because the original failure depended on undeclared state; divergence is diagnostic evidence, not a replay-system failure by itself.
- Dependency on immutable log work means early implementation may honestly report missing immutable-log refs.
