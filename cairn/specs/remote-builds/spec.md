# Remote Builds Specification

## Purpose

Defines the `remote-builds` capability.

## Requirements

### Requirement: Mantle uses a versioned P2P remote-build protocol [r[remote_builds.p2p_protocol]]

Mantle MUST use a versioned, authenticated, bounded remote-build protocol for P2P builder sessions. The protocol MUST negotiate version and capabilities before build requests, validate endpoint identity, enforce an explicit message state machine, bound every variable-length message or list, and fail closed on mismatches or unexpected state.

#### Scenario: Compatible protocol reaches authorization [r[remote_builds.p2p_protocol.scenario.compatible]]

- GIVEN a client connects to a builder with a supported remote-build ALPN, matching protocol version, matching endpoint identity, and compatible capability set
- WHEN the session sends the `Hello` message
- THEN the builder MUST proceed to the authorization state
- AND the accepted capability set MUST be recorded in the session state.

#### Scenario: Version mismatch fails closed [r[remote_builds.p2p_protocol.scenario.version-mismatch]]

- GIVEN a client connects with an unsupported protocol version or ALPN
- WHEN the builder validates the session greeting
- THEN the builder MUST reject the session before accepting credentials or build requests
- AND diagnostics MUST identify the unsupported protocol version or ALPN.

#### Scenario: Message and list limits are enforced [r[remote_builds.p2p_protocol.scenario.limits]]

- GIVEN a peer sends an oversized message, too many chunks, too many paths or object refs, too many bytes, or a message that is invalid for the current protocol state
- WHEN Mantle decodes the remote-build stream
- THEN Mantle MUST fail the session with deterministic diagnostics
- AND it MUST NOT enqueue, start, or import a build from that invalid stream.

### Requirement: Mantle keeps remote-build transport bindings pluggable and stdio-safe [r[remote_builds.transport_bindings]]

Mantle MUST keep remote-build protocol semantics independent of the underlying byte transport. Loopback, stdio child, SSH-stdio, and P2P bindings MAY carry the same versioned frames, and every binding MUST preserve authentication, endpoint identity validation, message limits, concrete-build validation, and output-trust checks.

#### Scenario: Same protocol runs over multiple bindings [r[remote_builds.transport_bindings.scenario.same-protocol]]

- GIVEN two transport bindings provide ordered authenticated frames for the same remote-build protocol version
- WHEN a client performs Hello, authorization, missing-input sync, build execution, and output transfer over either binding
- THEN Mantle MUST drive the same protocol state transitions and validation decisions
- AND transport-specific code MUST NOT bypass build request, input, or output trust checks.

#### Scenario: Stdio mode reserves stdout for protocol frames [r[remote_builds.transport_bindings.scenario.stdio-stdout]]

- GIVEN a remote builder is launched in stdio protocol mode
- WHEN the child emits logs, progress diagnostics, tracing, human-readable errors, or any unframed byte
- THEN Mantle MUST keep those bytes off stdout or fail the session as protocol corruption
- AND it MUST NOT enqueue a build or import outputs from a corrupted stdout stream.

#### Scenario: Initial handshake completes before expensive work [r[remote_builds.transport_bindings.scenario.cheap-handshake]]

- GIVEN a remote builder has expensive store scans, host probes, or missing-input walks available after connection
- WHEN the client opens a remote-build session
- THEN Mantle MUST complete Hello, authorization, and capability reporting before starting those expensive operations
- AND remote platform or capability facts MUST come from Mantle capability data rather than a kernel-name shortcut.

### Requirement: Mantle supports coordinator-driven worker scheduling [r[remote_builds.coordinator_scheduling]]

Mantle MUST support an optional coordinator scheduling role for remote builders without making that coordinator an output-trust root. Workers MUST register endpoint identity, systems, feature and capability labels, sandbox and network modes, concurrency, output signing-key identities, and resumable job summaries before receiving work. The coordinator MUST match requests by concrete build identity, required system, required capabilities, trust preflight, upload limits, resource limits, and logical store prefix.

#### Scenario: Worker-initiated registration advertises capabilities [r[remote_builds.coordinator_scheduling.scenario.worker-registration]]

- GIVEN a worker starts behind a network boundary where inbound client connections may be unavailable
- WHEN it initiates a session with the coordinator and sends a valid worker registration
- THEN the coordinator MUST record the worker endpoint identity, supported systems, feature labels, sandbox and network modes, concurrency, and output signing-key identities
- AND it MUST NOT assign jobs requiring capabilities the worker did not advertise.

#### Scenario: Capability mismatch is not dispatched [r[remote_builds.coordinator_scheduling.scenario.capability-mismatch]]

- GIVEN a concrete build request requires a system, feature label, sandbox mode, network mode, or resource limit that no registered worker satisfies
- WHEN the coordinator evaluates the request for dispatch
- THEN Mantle MUST reject the dispatch or keep it pending only under an explicit wait policy
- AND diagnostics MUST identify the missing capability without falling back to an incompatible worker.

#### Scenario: Identical concrete requests attach to one in-flight job [r[remote_builds.coordinator_scheduling.scenario.dedupe-identical]]

- GIVEN two clients submit the same normalized concrete build key, excluding per-attempt transport ids, temp paths, and log cursors
- WHEN a matching job is already queued, running, or finished-but-undelivered
- THEN Mantle MUST attach the later client to the existing job log and result
- AND it MUST NOT start a duplicate build for that same normalized key.

#### Scenario: Conflicting live output claims are rejected [r[remote_builds.coordinator_scheduling.scenario.reject-conflict]]

- GIVEN an in-flight job owns a live output lease or transfer id
- WHEN a different normalized build request claims the same live output lease or transfer id
- THEN Mantle MUST reject the conflicting request before dispatch
- AND diagnostics MUST distinguish a conflict from ordinary identical-request dedupe.

#### Scenario: Re-registered workers can redeliver resumable results [r[remote_builds.coordinator_scheduling.scenario.resume-redelivery]]

- GIVEN a coordinator restarts or a worker reloads while a remote build is running or finished but not fully delivered
- WHEN the worker re-registers with resumable job keys and the client resubmits the same normalized build key
- THEN Mantle SHOULD reattach the client to the retained job or redeliver the verified result when the worker still owns it
- AND it MUST report a phase-classified loss instead of silently launching a conflicting duplicate when the job cannot be resumed.

### Requirement: Mantle supports bounded tickets and trusted clients for builder access [r[remote_builds.access_tickets]]

Mantle MUST support both configured trusted clients and copy-paste bearer tickets for remote builder resource access. Ticket records MUST include a secret-derived identifier, optional display name, creation time, expiration time, optional uses remaining, maximum build time, maximum upload bytes, optional bound client endpoint id, and revoked state. Ticket possession MUST authorize only builder resource access and MUST NOT authorize output import trust.

#### Scenario: One-use ticket redeems after a valid request is queued [r[remote_builds.access_tickets.scenario.redeem-after-validation]]

- GIVEN a ticket has one use remaining and the client sends a well-formed concrete build request
- WHEN the builder validates authorization and request shape and accepts the job into the queue
- THEN Mantle MUST decrement the ticket use count exactly once
- AND subsequent use of the same ticket MUST fail if no uses remain.

#### Scenario: Malformed request does not consume a one-use ticket [r[remote_builds.access_tickets.scenario.malformed-not-consumed]]

- GIVEN a ticket has one use remaining and the client authenticates with that ticket
- WHEN the client sends a malformed build request that fails shape validation before queue admission
- THEN Mantle MUST reject the request
- AND the ticket use count MUST remain unchanged.

#### Scenario: Expired revoked or bound tickets fail [r[remote_builds.access_tickets.scenario.reject-invalid]]

- GIVEN a ticket is expired, revoked, exhausted, or bound to a different client endpoint id
- WHEN a client attempts to authorize a remote-build session with that ticket
- THEN Mantle MUST reject authorization before queue admission
- AND diagnostics MUST identify the ticket policy reason without revealing the bearer secret.

#### Scenario: Ticket list hides bearer secrets [r[remote_builds.access_tickets.scenario.secret-safe-list]]

- GIVEN an operator lists tickets or server status
- WHEN Mantle renders the ticket or status view
- THEN Mantle MUST show non-secret ticket metadata and secret-derived identifiers only
- AND it MUST require an explicit reveal command to print a bearer ticket string.

### Requirement: Mantle separates builder access from output trust [r[remote_builds.output_trust_separation]]

Mantle MUST treat remote builder resource authorization and remote output import trust as separate decisions. A ticket, trusted client entry, or successful protocol authentication MUST NOT be sufficient to import output PathInfo, artifacts, or attestations. Output import MUST require configured trust in the builder signing key, accepted attestation policy, or another explicit Mantle trust root.

#### Scenario: Ticketed build with untrusted output key fails before import [r[remote_builds.output_trust_separation.scenario.ticket-not-output-trust]]

- GIVEN a client has a valid ticket for a builder but has not configured trust for that builder's output signing key or attestation authority
- WHEN the client prepares or completes the remote build session
- THEN Mantle MUST reject output import or fail the preflight before dispatch
- AND diagnostics MUST state that builder access is not output trust.

#### Scenario: Trusted output key permits verified import [r[remote_builds.output_trust_separation.scenario.trusted-key]]

- GIVEN the builder signs produced PathInfo and artifact attestations with a key trusted by the client
- WHEN the client verifies the remote outputs against the requested build identity and store prefix
- THEN Mantle MAY import the outputs without rebuilding locally
- AND the build report MUST identify the verified builder key or trust root.

#### Scenario: Key or policy mismatch rejects outputs [r[remote_builds.output_trust_separation.scenario.reject-mismatch]]

- GIVEN remote outputs are signed by an unknown key, signed by a same-name-but-different key, missing required attestations, or governed by a mismatched producer policy
- WHEN the client evaluates output import
- THEN Mantle MUST reject the outputs
- AND it MUST NOT report a successful cache hit or remote build result for those outputs.

### Requirement: Remote builders execute concrete Mantle build inputs [r[remote_builds.local_eval_concrete_execution]]

Mantle remote builders MUST execute only concrete Mantle build inputs that have already been evaluated or lowered by the client or an external frontend. Remote builders MUST NOT evaluate arbitrary Nickel source, interpret frontend module-layer semantics, or accept raw frontend configuration as a build request.

#### Scenario: Concrete derivation graph is accepted [r[remote_builds.local_eval_concrete_execution.scenario.concrete]]

- GIVEN the client locally evaluates a Mantle expression or receives frontend-neutral build inputs from an external frontend
- WHEN it sends a concrete derivation graph or action-spec graph with requested outputs and provenance to the builder
- THEN the builder MUST validate the concrete request shape
- AND it MAY enqueue the build after authorization and input checks pass.

#### Scenario: Raw Nickel evaluation request is rejected [r[remote_builds.local_eval_concrete_execution.scenario.no-remote-eval]]

- GIVEN a remote-build request contains raw Nickel source, import paths, frontend module settings, Onix roles, provider topology, or a request for the builder to evaluate module semantics
- WHEN the builder validates the request
- THEN Mantle MUST reject the request before queue admission
- AND it MUST NOT perform remote evaluation or frontend-specific interpretation.

#### Scenario: Build identity is bound to the request [r[remote_builds.local_eval_concrete_execution.scenario.identity-bound]]

- GIVEN a concrete request declares derivation or action identities, requested output names, store prefix, hermeticity mode, and input refs
- WHEN the builder completes the build
- THEN the returned outputs MUST be checked against those declared identities
- AND stale or extra outputs MUST NOT satisfy the request.

### Requirement: Mantle synchronizes missing remote inputs through native CAS manifests [r[remote_builds.missing_input_sync]]

Mantle MUST synchronize remote build inputs through Mantle-native manifests and content-addressed objects. The client MUST declare derivations or action specs, PathInfo refs, source-input refs, blob/directory/object refs, closure metadata, and configured logical store prefix. The builder MUST request only missing inputs, verify uploaded content before sandbox start, and fail closed on digest, signature, prefix, or closure mismatches.

#### Scenario: Builder requests only missing inputs [r[remote_builds.missing_input_sync.scenario.missing-only]]

- GIVEN the client declares a concrete input manifest and the builder already has some referenced inputs locally
- WHEN the builder computes the missing input set
- THEN the builder MUST request only refs it lacks
- AND the client MUST upload only the requested missing refs.

#### Scenario: Uploaded input digest mismatch fails [r[remote_builds.missing_input_sync.scenario.digest-mismatch]]

- GIVEN the builder requests a missing CAS object or PathInfo record
- WHEN the uploaded bytes, object digest, PathInfo signature, source-input manifest, or store prefix does not match the declared input manifest
- THEN Mantle MUST reject the upload before sandbox execution
- AND the remote build MUST NOT start from the invalid input set.

#### Scenario: Upload quota is enforced [r[remote_builds.missing_input_sync.scenario.upload-limit]]

- GIVEN the authorized client or ticket has a maximum upload byte limit
- WHEN missing input upload exceeds that limit
- THEN Mantle MUST stop the upload and fail the session with deterministic diagnostics
- AND it MUST NOT enqueue or continue the build after the limit violation.

### Requirement: Mantle imports remote outputs through signed delta-capable substitution [r[remote_builds.signed_delta_output_import]]

Mantle MUST return remote build outputs through signed PathInfo, artifact attestations, and substitution import paths. The client MUST verify output signatures, requested identity, content/object digests, store prefix, and attestation policy before acceptance. When compatible delta capabilities are advertised, the client SHOULD prefer delta transfer and MUST fall back to full transfer when delta is unavailable or fails safely.

#### Scenario: Delta-capable output import succeeds [r[remote_builds.signed_delta_output_import.scenario.delta]]

- GIVEN the builder and client advertise compatible delta transfer capabilities and the builder has produced signed output metadata
- WHEN the client requests the missing output closure
- THEN Mantle SHOULD transfer reusable chunks or blobs through the delta protocol
- AND the build report MUST record delta mode, transferred bytes, reused bytes, and verified output identity.

#### Scenario: Delta failure falls back to full transfer [r[remote_builds.signed_delta_output_import.scenario.full-fallback]]

- GIVEN a delta transfer attempt fails before verified output admission but full transfer is available
- WHEN the client falls back to full output transfer
- THEN Mantle MUST verify the full transferred output before import
- AND the build report MUST record full mode with a fallback reason rather than claiming a successful delta hit.

#### Scenario: Tampered output is rejected [r[remote_builds.signed_delta_output_import.scenario.reject-tamper]]

- GIVEN a remote output has a mismatched object digest, stale PathInfo, missing signature, mismatched store prefix, mismatched requested output, or invalid artifact attestation
- WHEN the client validates remote output import
- THEN Mantle MUST reject the output
- AND it MUST NOT persist or export the tampered output as a successful build result.

### Requirement: Mantle reports remote builder queue status with enforced limits [r[remote_builds.queue_status_limits]]

Mantle remote builders MUST expose redacted status snapshots and enforce configured resource limits. Status MUST include builder endpoint id, configured concurrency, queued jobs, active jobs, recent jobs, phases, durations, and short errors. Status MUST NOT reveal bearer ticket secrets, uploaded content, secret environment values, or untrusted log payload as control data.

#### Scenario: Status shows queue and active phases [r[remote_builds.queue_status_limits.scenario.status]]

- GIVEN a remote builder has queued, active, and recent jobs
- WHEN an operator requests status
- THEN Mantle MUST report queue positions, active phases, durations, configured concurrency, and recent results
- AND secret-bearing fields MUST be redacted.

#### Scenario: Build time limit is enforced [r[remote_builds.queue_status_limits.scenario.build-time-limit]]

- GIVEN an authorized client or ticket has a maximum build time
- WHEN the remote build exceeds that duration
- THEN Mantle MUST stop or fail the build according to the executor policy
- AND the recent status and client diagnostic MUST identify a build-time limit violation.

#### Scenario: Concurrency limit bounds execution [r[remote_builds.queue_status_limits.scenario.concurrency]]

- GIVEN the builder is configured for a maximum number of concurrent builds
- WHEN more authorized requests arrive than available build slots
- THEN Mantle MUST keep excess requests queued until a slot is available or the request fails
- AND it MUST NOT start more builds than the configured concurrency limit.

#### Scenario: Live logs are bounded and replayable [r[remote_builds.queue_status_limits.scenario.log-replay]]

- GIVEN a remote build streams logs while clients may disconnect and reconnect
- WHEN Mantle records and replays log chunks for that job
- THEN Mantle MUST enforce configured log-byte, silent-time, and replay-cursor limits
- AND it MUST drop, truncate, or fail slow subscribers according to policy instead of buffering unbounded logs.

### Requirement: Mantle manages remote session lifecycle with phase-classified failures [r[remote_builds.session_lifecycle]]

Mantle MUST assign each remote-build connection attempt a stable session identity, classify failures by phase, and protect live remote-build artifacts with session-scoped leases or roots. Transport failures SHOULD be retried according to client policy, while authentication, builder configuration, capability, and output-trust failures MUST surface as terminal diagnostics until operator inputs change.

#### Scenario: Failure phase controls retry and diagnostics [r[remote_builds.session_lifecycle.scenario.failure-phase]]

- GIVEN a remote session fails during transport setup, authentication, request validation, build execution, or output import
- WHEN Mantle reports the failure to a client or operator
- THEN diagnostics MUST identify the phase and retry class
- AND a resource-access success MUST NOT hide a later build or output-trust failure.

#### Scenario: Stable session identity prevents reconnect spin [r[remote_builds.session_lifecycle.scenario.stable-session]]

- GIVEN a transport dies and the client creates new protocol client values during reconnect attempts
- WHEN Mantle decides whether the active session changed
- THEN it MUST compare stable session or spawn identities rather than fresh proxy/client object identity
- AND it MUST apply bounded reconnect or backoff behavior instead of spinning without yielding.

#### Scenario: Live artifacts are leased while sessions need them [r[remote_builds.session_lifecycle.scenario.session-leases]]

- GIVEN a remote build session uploads inputs, realizes helper artifacts, or produces outputs that are still needed by an active job or transfer
- WHEN local or remote garbage collection runs
- THEN Mantle MUST keep those artifacts reachable through a session-scoped lease or root
- AND it SHOULD release or replace that lease when the session or queued job no longer needs the artifacts.

#### Scenario: Restart adoption preserves phase truth [r[remote_builds.session_lifecycle.scenario.restart-adoption]]

- GIVEN a coordinator, direct builder, or worker process restarts while a client is waiting for remote build logs or outputs
- WHEN the client reconnects and the remote side advertises retained session, job, log, or result state
- THEN Mantle SHOULD resume from the retained state when the normalized build key and trust policy still match
- AND it MUST report the exact lost phase when retained state is unavailable rather than reporting stale success.

### Requirement: Remote outputs use a single trust-admission pipeline

r[remote_builds.output_trust_admission_pipeline] Mantle MUST admit remote build outputs only through a single output-trust pipeline that checks requested output identity, logical store prefix, PathInfo signatures, signer key material, object refs, artifact attestations, producer policy, revocation/expiration state, and requested claim strength. Builder tickets, trusted-client entries, coordinator assignment, or transport authentication MUST authorize only resource access and MUST NOT by themselves admit outputs.

#### Scenario: trusted output evidence is imported

GIVEN a remote builder returns PathInfo and artifact-attestation evidence signed by key material trusted by the client
AND the returned output identity, object refs, store prefix, producer policy, and requested output names match the concrete request
WHEN Mantle performs output admission
THEN Mantle MAY import and export the output
AND the build report MUST identify the verified trust basis without exposing private key paths or bearer ticket secrets.

#### Scenario: resource access without output trust is blocked

GIVEN a client has a valid remote-build ticket or trusted-client authorization
AND the client lacks configured trust for the builder signing key, attestation authority, or required receipt policy
WHEN Mantle plans or completes a remote build
THEN Mantle MUST reject the remote route or output import as an output-trust blocker
AND it MUST NOT treat resource authorization as a cache hit, substitution hit, or successful remote build result.

#### Scenario: same-name different-key and stale output fail closed

GIVEN a returned output is signed by an unknown key, a same-name-but-different key, a revoked or expired key, or has stale object refs, wrong store prefix, missing attestation, or mismatched requested output identity
WHEN Mantle validates remote output admission
THEN Mantle MUST reject the output with deterministic diagnostics
AND it MUST NOT persist or export the tampered or untrusted output as successful.

### Requirement: Stdio and SSH-stdio are hardened remote-build bindings

r[remote_builds.stdio_ssh_hardened_bindings] Mantle MUST support stdio and SSH-stdio as operator-facing remote-build bindings that carry the same versioned remote-build frame protocol and enforce the same authentication, endpoint identity, message limits, concrete-request validation, input-sync checks, and output-trust admission as other remote transports. Stdout MUST be reserved for protocol frames, while logs and diagnostics MUST be bounded and separated from control data.

#### Scenario: SSH-stdio preserves protocol semantics

GIVEN a remote builder is launched over SSH-stdio with a configured endpoint identity and ticket
WHEN the client performs hello, authorization, input sync, build execution, and output transfer
THEN Mantle MUST drive the same state transitions as the stdio binding
AND accepted outputs MUST pass the same signed output-admission checks before import.

#### Scenario: human stdout corrupts the protocol

GIVEN a stdio or SSH-stdio builder writes unframed human text, logs, tracing, or errors to stdout
WHEN the client decodes the remote-build stream
THEN Mantle MUST fail the session as terminal protocol corruption
AND it MUST NOT enqueue a build or import outputs from that corrupted stream.

#### Scenario: handshake is cheap and bounded

GIVEN a remote builder has expensive store scans, input walks, or sandbox setup available after connection
WHEN a stdio or SSH-stdio session starts
THEN Mantle MUST complete hello, authorization, capability reporting, and request shape validation before expensive work starts
AND timeout, oversized-frame, child-exit, and invalid-sequence failures MUST include phase-classified diagnostics.

### Requirement: Remote build dispatch integrates with the lazy scheduler

r[remote_builds.scheduler_build_service_dispatch] Mantle MUST expose remote build execution through a scheduler-compatible dispatch boundary so ready derivation or action goals can be realized remotely without bypassing lazy goal deduplication, dependency interleaving, route policy, or output admission. The remote dispatcher MUST accept only concrete evaluated build inputs and MUST return outputs through signed PathInfo, artifact-attestation, and store import validation before a goal is marked done.

#### Scenario: scheduler dispatches a ready goal remotely

GIVEN a ready goal has a selected remote-builder route
AND the remote build service receives concrete build inputs, an upload manifest, and trusted output-admission facts
WHEN the scheduler dispatches the goal
THEN Mantle MAY run the remote build through the remote service
AND the goal MUST be marked done only after verified remote outputs are imported through the ordinary store path.

#### Scenario: identical ready goals dedupe across remote dispatch

GIVEN two requested roots normalize to the same concrete build key
AND the first request has already queued, started, or finished a remote dispatch that still owns the result
WHEN the scheduler receives the second request
THEN Mantle MUST attach the second request to the existing goal/result
AND it MUST NOT start a duplicate remote build for the same normalized key.

#### Scenario: invalid remote request cannot bypass scheduler policy

GIVEN a remote route would require raw Nickel evaluation, frontend module interpretation, mismatched store prefix, invalid fallback policy, or untrusted output admission
WHEN the scheduler evaluates dispatch
THEN Mantle MUST reject the remote route before marking the goal complete
AND it MUST preserve a phase-classified failure instead of fabricating a local or remote success.

### Requirement: Remote input sync consumes verified source-bundle state

r[remote_builds.source_bundle_input_sync] Mantle MUST synchronize remote build inputs using declared CAS, PathInfo, and source-input refs, including verified source-bundle or imported-source-state records. The builder MUST request only missing refs, the client MUST upload only requested refs whose bytes match the declared identity, and upload planning MUST enforce privacy and quota policy before any source, store, proof, or secret-descriptor bytes move.

#### Scenario: imported source state satisfies a missing remote input

GIVEN a remote build request declares a source-input ref
AND the client has a verified imported source-bundle record for that source identity even though the logical source path is absent from the physical store
WHEN the builder requests the missing source ref
THEN the client MAY materialize and upload the source from imported source state
AND the builder MUST verify the uploaded digest, source identity, readiness class, and store-prefix binding before sandbox execution.

#### Scenario: stale or unsupported source input is rejected

GIVEN the client has source material with a stale digest, unsupported source kind, wrong store prefix, missing readiness class, or mismatched source identity
WHEN remote input sync evaluates the upload
THEN Mantle MUST reject the upload before sandbox execution
AND the remote build MUST NOT repair the input by fetching from the network or reading ambient package-manager caches.

#### Scenario: upload privacy policy runs before transfer

GIVEN a remote-builder candidate requires missing input upload
WHEN Mantle plans or starts the remote route
THEN the upload plan MUST summarize bounded source, store, proof, and secret-descriptor classes, object counts, and byte counts
AND a disallowed class or quota overflow MUST reject the route before a remote session receives those bytes.

### Requirement: Remote build status and reports are bounded diagnostic evidence

r[remote_builds.operator_observability] Mantle MUST expose remote-build status and build reports as bounded, redacted diagnostic evidence. Reports MUST identify route decisions, rejected-route reasons, worker capabilities, queue phases, active and recent jobs, upload classes/counts/bytes, transfer mode, transferred and reused bytes, fallback reasons, signer/trust basis, artifact-attestation references, log cursors, retry class, and non-claims when those facts are available. Reports MUST NOT reveal bearer tickets, private key paths, raw environment values, uploaded content, unbounded argv/path lists, or untrusted log payload as control data.

#### Scenario: status summarizes queue and workers safely

GIVEN a remote coordinator or builder has registered workers, tickets, queued jobs, active jobs, recent failures, and retained log cursors
WHEN an operator requests remote status
THEN Mantle MUST render bounded human and JSON snapshots of those facts
AND secret-bearing fields MUST be redacted or omitted.

#### Scenario: build report explains accepted remote output

GIVEN a remote build output is admitted through verified output trust
WHEN Mantle renders the build report
THEN the report MUST include selected route, endpoint id, upload summary, transfer mode, transferred bytes, reused bytes, fallback reason when present, signer or trust-basis identity, artifact-attestation path, and output identity
AND it MUST distinguish route eligibility, remote execution, output transfer, and output import claims.

#### Scenario: logs and diagnostics cannot become unbounded control data

GIVEN a remote build streams logs, reconnects clients, emits oversized diagnostics, or includes untrusted control-looking bytes in logs
WHEN Mantle records, replays, or reports those diagnostics
THEN Mantle MUST enforce configured byte, chunk, cursor, and redaction limits
AND slow subscribers or oversized logs MUST be truncated or failed according to policy instead of buffering unbounded data.
