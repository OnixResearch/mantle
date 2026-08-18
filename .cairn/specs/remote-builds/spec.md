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

### Requirement: Operator remote-build e2e rail is proof-bound and redacted

r[remote_builds.operator_e2e_rail] Mantle MUST provide a deterministic operator-facing remote-build e2e validation rail that composes route planning, framed handshake, source/input sync, remote execution, signed output admission, and bounded observability without depending on ambient network services or hidden global state. The rail MUST produce or assert machine-readable evidence for the selected route, handshake phase, upload summary, execution phase, transfer/admission phase, signer or trust basis, artifact-attestation reference, log/status bounds, redaction, and explicit non-claims.

#### Scenario: successful fixture proves remote composition only

GIVEN a concrete build request has a compatible remote route, bounded upload requirements, and explicit output trust for the builder key
WHEN the operator rail executes the remote-build fixture
THEN Mantle MUST complete route planning, framed handshake, input sync, remote execution, signed output admission, and report rendering through the same core validation seams used by supported remote-build operation
AND the resulting evidence MUST state that the rail proves fixture composition only, not production P2P deployment, release reproducibility, or general package-manager compatibility.

#### Scenario: cross-seam failures fail closed

GIVEN the rail fixture lacks output trust, emits unframed stdout, presents stale source state, exceeds upload quota or privacy policy, or requests fallback without an explicit policy
WHEN Mantle runs the corresponding negative case
THEN Mantle MUST reject the remote path before output admission or local-success reporting
AND diagnostics MUST identify the phase and stable reason code without revealing bearer tickets, private key paths, raw environment values, uploaded content, or unbounded logs.

### Requirement: Coordinator runtime schedules workers without becoming output trust

r[remote_builds.coordinator_worker_runtime] Mantle MUST provide a coordinator runtime that records worker registrations, capability facts, concurrency, queue state, live leases, output signing-key identities, and resumable job summaries, then matches concrete build requests by normalized build key, required capabilities, resource policy, upload feasibility, logical store prefix, and client output-trust preflight. The coordinator MUST NOT be treated as an output trust root.

#### Scenario: worker registration enables matching dispatch

GIVEN a worker initiates a coordinator session and registers endpoint identity, systems, feature labels, sandbox modes, network modes, concurrency, transfer capabilities, and output signing-key identities
WHEN a compatible concrete build request enters the coordinator queue
THEN the coordinator MAY assign the job to that worker
AND it MUST NOT assign jobs requiring unadvertised capabilities or mismatched logical store prefix.

#### Scenario: duplicate requests attach to one job

GIVEN two clients submit equivalent concrete requests with the same normalized build key
AND the first job is queued, running, or finished but not fully delivered
WHEN the second request is admitted
THEN Mantle MUST attach the second client to the existing job log/result
AND it MUST NOT start a duplicate build for that normalized key.

#### Scenario: restart adoption preserves phase truth

GIVEN a coordinator or worker restarts while a job is queued, running, transferring, or finished-undelivered
WHEN the worker re-registers and the client resubmits the normalized request
THEN Mantle SHOULD resume or redeliver retained state when the normalized key and trust policy still match
AND it MUST report the exact lost phase instead of silently launching a conflicting duplicate or reporting stale success.

### Requirement: Production remote realization is scheduler-owned

r[remote_builds.production_scheduler_realization] Mantle MUST route production remote builds through the lazy scheduler and scheduler-compatible build service boundary. The scheduler MUST own goal readiness, dependency waiting, duplicate-goal dedupe, route selection, `max_jobs` accounting, waiter notification, local fallback policy, terminal-state propagation, and build report emission for both local and remote realizations.

#### Scenario: ready goal realizes remotely without bypassing scheduler

GIVEN a ready derivation or action goal has a selected remote-builder route
AND all concrete inputs, upload policy, capability facts, and output-trust facts are available
WHEN Mantle dispatches the goal
THEN the remote realizer MAY execute the goal
AND the scheduler MUST mark the goal done only after verified remote outputs are admitted through the ordinary store path.

#### Scenario: duplicate remote goal attaches instead of rebuilding

GIVEN two dependents request equivalent concrete build inputs with the same normalized realization key
AND the first request is queued, running, transferring, or finished but not delivered
WHEN the second request reaches scheduler dispatch
THEN Mantle MUST attach it to the existing goal or retained result
AND it MUST NOT start a second local or remote realization for that key.

#### Scenario: invalid remote route fails before success propagation

GIVEN a remote route would require raw frontend evaluation, mismatched store prefix, unavailable source/input material, invalid fallback policy, or missing output trust
WHEN the scheduler evaluates remote dispatch
THEN Mantle MUST reject the remote route before marking the goal complete
AND it MUST preserve a phase-classified failure or explicit local fallback decision.

### Requirement: Production build farms are CI-neutral

r[remote_builds.production_ci_build_separation] Mantle MUST keep CI/jobset/pipeline orchestration separate from production remote build realization. CI systems, release tools, and other frontends MAY submit concrete Mantle build requests and consume status, cache, transfer, receipt, and report APIs, but scheduler, coordinator, cache identity, transfer admission, and output-trust decisions MUST NOT depend on CI-owned concepts such as jobsets, pipeline graphs, webhooks, branch policy, pull-request policy, checkout discovery, or frontend evaluation scheduling.

#### Scenario: external CI submits concrete build requests

GIVEN an external CI system has already chosen a revision, job, pipeline stage, or release candidate to build
AND it submits a concrete Mantle derivation/action request with declared inputs, policy, and output trust roots
WHEN Mantle plans remote realization
THEN Mantle MAY realize the concrete request through the remote build farm
AND CI labels MUST remain report metadata instead of route keys, cache identities, transfer authorities, or output-trust proof.

#### Scenario: CI scheduling fields are rejected at the build boundary

GIVEN a request asks the remote build farm to interpret jobsets, webhook payloads, branch filters, pull-request policy, checkout discovery, or frontend evaluation scheduling
WHEN Mantle validates the request for scheduler or coordinator admission
THEN Mantle MUST reject those CI-owned fields before dispatch
AND diagnostics MUST direct callers to submit concrete build inputs instead.

#### Scenario: CI success cannot prove build correctness

GIVEN an external CI system reports a job as successful
AND the corresponding remote output lacks verified PathInfo, content refs, or required attestation evidence
WHEN Mantle evaluates cache, transfer, or output admission
THEN Mantle MUST reject the output as unverified
AND it MUST NOT treat CI success as a substitute for build output trust.

### Requirement: Production output trust is cryptographic

r[remote_builds.production_cryptographic_output_trust] Mantle MUST admit production remote outputs only after cryptographic verification of returned PathInfo signatures and required attestation evidence against configured trust roots. Builder tickets, worker registration, SSH authentication, coordinator assignment, key names without matching material, or successful transport authentication MUST NOT be sufficient to import or persist remote outputs.

#### Scenario: trusted key material admits matching output

GIVEN a remote worker returns PathInfo, output content refs, and artifact-attestation evidence for the requested output
AND the PathInfo signature verifies against configured trusted public key material
AND the returned output name, store prefix, object refs, producer policy, and attestation digest match the concrete request
WHEN Mantle performs remote output admission
THEN Mantle MAY persist and export the output
AND the build report MUST record the trust basis without exposing private key paths or bearer tickets.

#### Scenario: same-name different-key output is rejected

GIVEN a remote worker returns PathInfo signed by a key with the same display name as a trusted key but different key material
WHEN Mantle validates the remote output
THEN Mantle MUST reject the output as an output-trust failure
AND it MUST NOT persist the PathInfo, export the output, or report a remote build success for that output.

#### Scenario: resource access is not output trust

GIVEN a client has a valid remote-build ticket, trusted-client credential, SSH identity, or coordinator assignment
AND the client lacks configured output signing-key or attestation-authority trust for the returned output
WHEN the remote route is planned or completed
THEN Mantle MUST reject the route or output import as an output-trust blocker
AND it MUST NOT treat resource authorization as a cache hit, substitution hit, or successful remote build result.

### Requirement: Production transfer is streaming and resumable

r[remote_builds.production_streaming_transfer] Mantle MUST transfer production remote inputs and outputs through bounded streaming artifacts keyed by declared content identities. The transfer layer MUST support CAS object, NAR, source-bundle, PathInfo, attestation, and delta artifacts; verify BLAKE3 content identities before use; preserve Nix-compatible NAR SHA-256 checks where required; enforce upload/download quotas and privacy policy; provide backpressure; and support safe resume or deterministic failure after interruption.

#### Scenario: large output streams without a single oversized frame

GIVEN a remote output is larger than the maximum control-frame payload
WHEN the client requests the output closure
THEN Mantle MUST stream the output through chunked CAS, NAR, or delta artifacts
AND output admission MUST verify every declared chunk/object identity before persistence.

#### Scenario: delta fallback remains claim-safe

GIVEN the client and worker advertise compatible delta transfer
AND the delta stream cannot be applied or verified before output admission
WHEN full transfer is available under policy
THEN Mantle MUST fall back to full transfer and verify the full output before import
AND the report MUST record the fallback reason instead of claiming a successful delta transfer.

#### Scenario: invalid transfer fails before sandbox or import

GIVEN a transfer has a digest mismatch, stale resume cursor, missing requested object, disallowed secret-descriptor upload, or quota overflow
WHEN Mantle validates the transfer
THEN Mantle MUST reject the transfer before sandbox execution or output import
AND diagnostics MUST identify the transfer phase and reason without revealing uploaded content.

### Requirement: Production coordinator runtime is restart-safe

r[remote_builds.production_coordinator_runtime] Mantle MUST provide a durable coordinator/worker runtime for production remote build farms. Workers MUST register endpoint identity, protocol version, systems, feature labels, sandbox modes, network modes, logical store prefixes, transfer capabilities, concurrency, output signing-key identities, and resumable job summaries. The coordinator MUST persist queue state, live leases, bounded log cursors, recent failures, and result-redelivery facts so restart or reconnect cannot fabricate success or duplicate work.

#### Scenario: worker registration enables deterministic assignment

GIVEN one or more workers have registered capability and trust-admission facts
AND a concrete build request declares required system, sandbox mode, network mode, feature labels, transfer policy, upload limits, logical store prefix, and trusted output keys
WHEN the coordinator plans dispatch
THEN it MAY assign the job only to a worker that satisfies every required fact
AND it MUST reject or explicitly pend the request when no matching worker is available.

#### Scenario: restart redelivers retained result

GIVEN a coordinator or worker restarts after a job has finished but before the client receives the admitted result
AND the worker re-registers a resumable summary for the same normalized build key and result identity
WHEN the client resubmits the request
THEN Mantle SHOULD redeliver the retained result through ordinary output admission
AND it MUST report the exact lost phase when retained state is unavailable.

#### Scenario: live leases prevent conflicting output ownership

GIVEN an active remote job owns a live output lease, transfer id, or result-delivery lease
WHEN a different normalized build request claims the same live output identity
THEN Mantle MUST reject the conflicting claim before dispatch
AND diagnostics MUST distinguish a lease conflict from duplicate-request attachment.

### Requirement: Accepted remote results are publishable artifacts

r[remote_builds.production_verified_publication] Mantle MUST publish production remote results only after local output admission has accepted the PathInfo, castore objects, artifact attestations, and required receipt evidence. Publisher adapters MAY target local archives, Nix-compatible binary caches, HTTP/S3 artifact stores, or future provider backends, but publication MUST remain separate from realization success and MUST NOT bypass output trust.

#### Scenario: admitted remote output is published for reuse

GIVEN a remote build output has been admitted into the local store through verified output trust
AND a publisher profile is configured for the output class
WHEN Mantle runs post-admission publication
THEN the publisher MAY export the verified artifact and metadata
AND a later request MAY resolve the artifact without remote execution after normal resolver verification.

#### Scenario: publication failure does not rewrite build truth

GIVEN a remote output has already been admitted locally
AND a configured publisher is unavailable, rejects credentials, or reports a duplicate artifact
WHEN publication runs after admission
THEN Mantle MUST report the publication failure or skip separately
AND it MUST NOT retroactively claim the build failed unless policy explicitly requires successful publication.

#### Scenario: publisher rejects unverified artifact

GIVEN a remote result has not completed local output admission
WHEN a publisher adapter is asked to publish it
THEN the publisher MUST reject the artifact
AND Mantle MUST report the rejection without treating publication as a verification substitute.

### Requirement: Remote farm configuration is typed and provider-neutral

r[remote_builds.production_operator_configuration] Mantle MUST load production remote build farm configuration through typed, reviewable, provider-neutral data. Mantle-owned config files MUST use Nickel contracts unless an external provider format is required, and runtime Rust MUST consume explicit exported data. Configuration MUST describe builder pools, capability profiles, upload and transfer limits, fallback policy, output trust roots, worker/resource policy, and publisher profiles without embedding provider-specific logic in scheduler/core modules.

#### Scenario: valid builder pool config selects capabilities

GIVEN a remote builder pool config declares endpoint identities, supported systems, sandbox/network modes, transfer capabilities, concurrency limits, upload budgets, and output trust roots
WHEN Mantle loads the config
THEN it MUST produce bounded capability facts for route planning and coordinator matching
AND provider-specific transport parameters MUST remain inside adapter configuration.

#### Scenario: unsafe config fails closed

GIVEN a remote builder config has duplicate endpoint ids, missing output trust roots, invalid upload limits, unsafe fallback defaults, or provider-specific fields in core scheduler configuration
WHEN Mantle validates the config
THEN Mantle MUST reject the config before remote dispatch
AND diagnostics MUST identify the invalid field without exposing credentials or private key paths.

### Requirement: Operator remote-build e2e rail composition proof emits bounded versioned evidence

r[remote_builds.operator_e2e_rail_composition_proof] Mantle MUST provide a bounded local multi-process composition proof of the operator remote-build e2e rail that composes route planning, framed handshake, source/input sync, remote execution, signed output admission, and bounded observability through the same core validation seams used by supported remote-build operation, without ambient network services or hidden global state, and MUST emit the result as a versioned, machine-readable evidence record with a stable schema version and stable field names for `rail_version`, `fixture_id`, `mode`, ordered composition phases, `upload_summary`, `trust_basis`, `artifact_attestation_ref`, `log_status_bounds`, `redaction`, and `non_claims`. The record MUST be byte-stable across repeated runs on the same fixture, with any inherently non-deterministic field documented and scrubbed or omitted, and MUST omit bearer tickets, private key paths, raw environment values, uploaded content, and unbounded logs, argv, or path lists.

#### Scenario: completed fixture proves composition and emits the mandated field set

GIVEN the operator remote-build e2e rail runs a bounded local multi-process fixture through route planning, framed handshake, source/input sync, remote execution, signed output admission, and bounded observability
WHEN Mantle writes the rail evidence record
THEN the record MUST carry the stable schema version and the mandated field set naming each composition phase and the core seam used
AND `non_claims` MUST state that the rail proves fixture composition only, not production P2P deployment, release reproducibility, or package-manager compatibility.

#### Scenario: cross-seam failures fail closed with stable reason codes

GIVEN the rail fixture lacks output trust, emits unframed stdout, presents stale source state, exceeds upload quota or privacy policy, or requests fallback without an explicit policy
WHEN Mantle runs the corresponding negative case
THEN Mantle MUST reject the remote path before output admission or local-success reporting
AND diagnostics MUST identify the phase and a stable reason code without revealing bearer tickets, private key paths, raw environment values, uploaded content, or unbounded logs.

#### Scenario: repeated runs are byte-stable

GIVEN the rail is run repeatedly against the same bounded fixture
WHEN Mantle renders the evidence record
THEN the record MUST be byte-stable modulo documented non-deterministic fields
AND inherently non-deterministic fields MUST be scrubbed or omitted rather than embedded verbatim.

#### Scenario: malformed or oversized evidence is not promoted

GIVEN the rail would emit a bearer ticket, private key path, raw environment value, uploaded content, unbounded log, unbounded argv list, or unbounded path list, or the evidence record is malformed or oversized
WHEN Mantle renders or consumes the record
THEN Mantle MUST omit or redact the disallowed field and diagnose the malformed or oversized record
AND it MUST NOT promote the record to a successful rail claim.

### Requirement: Remote assignments use durable attempt fencing [r[remote_builds.durable_attempt_fencing]]

Mantle MUST distinguish normalized realization keys, durable coordinator jobs, and per-assignment execution attempts. Every current assignment MUST carry a unique attempt id and a monotonically advancing fence generation, and every state-changing worker report MUST match the coordinator's current job, attempt, and fence before it can mutate durable state or reach output admission.

#### Scenario: Reassignment advances the fence

- GIVEN a live remote job is reassigned after timeout, worker loss, policy-directed retry, or operator cancellation
- WHEN the coordinator publishes the replacement assignment
- THEN it MUST durably record a new attempt id and a fence generation greater than the superseded assignment
- AND the superseded assignment MUST no longer be authorized to mutate current job state.

#### Scenario: Stale completion cannot win a race

- GIVEN an older worker reports completion after a replacement attempt became current
- WHEN the coordinator validates the older report's job, attempt, and fence
- THEN Mantle MUST reject the report as stale before transfer acceptance or output admission
- AND it MUST NOT replace, merge, or relabel the current attempt's state with the stale result.

#### Scenario: Restart preserves the current owner

- GIVEN a coordinator restarts with a queued, running, transferring, or finished-undelivered attempt
- WHEN durable state is reloaded
- THEN Mantle MUST recover the current attempt id, assignment nonce, fence generation, phase, and retained-result disposition
- AND ambiguous or legacy live state without a safe current nonce and fence MUST fail closed instead of inventing successful ownership.

#### Scenario: Coordinator state reset cannot recreate a stale owner

- GIVEN an older worker retains credentials from a prior coordinator state incarnation
- WHEN a replacement coordinator admits the same normalized request on the same worker and initial fence
- THEN the replacement job and attempt identities MUST use a fresh shell-supplied assignment nonce
- AND the older credentials MUST be rejected before any current-state mutation.

### Requirement: Attempt reports are idempotent and conflict detecting [r[remote_builds.idempotent_attempt_reporting]]

Mantle MUST apply attempt reports through stable event ids and canonical payload digests. Repeating the same event id with the same digest MUST be an idempotent no-op, while reusing an event id with different content or applying an event from a stale fence MUST fail before durable state changes.

#### Scenario: Duplicate delivery is harmless

- GIVEN a current attempt report was durably applied
- WHEN transport redelivers the same event id with the same canonical payload digest
- THEN Mantle MUST return an already-applied disposition
- AND job, log, transfer, lease, and output state MUST remain unchanged
- AND a finished-undelivered output response MAY be cryptographically revalidated to reconstruct its admission report after restart without reapplying state.

#### Scenario: Conflicting duplicate fails closed

- GIVEN a current or retained event id already binds one canonical payload digest
- WHEN a peer submits the same event id with a different payload digest
- THEN Mantle MUST reject the report with a stable conflict reason
- AND it MUST NOT apply either a merged event or the conflicting payload.

### Requirement: Remote attempt decisions have a pure deterministic core [r[remote_builds.pure_attempt_decisions]]

Mantle MUST implement remote authorization, retry, transition, idempotency, and fence decisions as pure deterministic functions over explicit bounded input facts. Clock reads, persistence, transport, cancellation, process control, cryptographic admission, and rendering MUST remain in imperative shell code.

#### Scenario: Equivalent facts produce equivalent decisions

- GIVEN two decision evaluations contain equivalent current attempt state, authorization facts, retry policy, failure class, budget, supplied time facts, and report identity
- WHEN the pure core evaluates them
- THEN both evaluations MUST return the same decision, next state, and stable reason code
- AND discovery order, wall-clock reads, environment state, filesystem state, and transport timing MUST NOT affect the result.

#### Scenario: Shell failure cannot bypass a core rejection

- GIVEN the pure core classifies a report as stale, unauthorized, conflicting, over budget, or terminal
- WHEN the coordinator shell handles that decision
- THEN the shell MUST NOT persist the rejected mutation or pass its output to admission
- AND a logging, cancellation, or exporter failure MUST NOT turn the rejection into success.

### Requirement: Remote transfer resume is scoped to the current attempt [r[remote_builds.attempt_scoped_transfer_resume]]

Mantle MUST bind each remote input/output transfer session and checkpoint to the durable job, current attempt id, current fence generation, canonical transfer manifest, and transfer policy. Reassignment MUST invalidate the old session's authority while allowing a new current session to reuse independently verified content through ordinary missing-object negotiation.

#### Scenario: Current attempt resumes safely

- GIVEN a current fenced attempt has a valid interrupted transfer checkpoint and verified receiver objects
- WHEN the same attempt reconnects under matching manifest and policy
- THEN Mantle MAY resume by requesting the deterministically remaining content
- AND completion MUST still pass signed PathInfo, requested identity, store-prefix, object, attestation, and claim-strength admission.

#### Scenario: Superseded attempt cannot resume

- GIVEN a transfer checkpoint names an attempt or fence superseded by reassignment
- WHEN the worker reconnects or submits another chunk, acknowledgement, or completion
- THEN Mantle MUST reject that mutation before changing current transfer or result state
- AND verified castore objects from the older session MAY be reused only after the current receiver reprobes them by content identity.

#### Scenario: Fallback remains claim safe

- GIVEN delta or resumable streaming cannot continue safely and policy allows full-NAR fallback
- WHEN Mantle uses the fallback
- THEN it MUST verify the full transfer and ordinary output-admission evidence before success
- AND the report MUST identify the actual transfer mode, bytes, and fallback reason instead of claiming streaming or delta completion.

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

### Requirement: Failed remote sandbox capture is explicit and bounded
r[remote_builds.failure_debug_capture]

Mantle MUST default remote failure bundles to metadata-only evidence. Failed-sandbox artifact capture MUST require explicit typed policy defining allowed relative paths or artifact classes, sensitivity policy, regular-file handling, maximum files/bytes/depth, retention, and cleanup behavior. Capture selection MUST be decided before filesystem reads, and rejected capture MUST NOT delay unboundedly or rewrite the remote build result.

#### Scenario: Allowed artifact is captured before cleanup

- GIVEN a remote attempt fails and policy explicitly allows a bounded regular-file artifact under the sandbox root
- WHEN Mantle applies the accepted capture plan before cleanup
- THEN it MAY ingest the artifact as a content-addressed debug object and bind its ref in the bundle
- AND sandbox cleanup or quarantine MUST continue according to the recorded policy outcome.

#### Scenario: Unsafe artifact is rejected

- GIVEN a requested artifact is absolute, traverses above the sandbox, follows an escaping symlink, is a socket/device/FIFO, exceeds file/count/byte/depth limits, or matches sensitive policy
- WHEN capture planning or application reaches it
- THEN Mantle MUST reject the artifact before publication
- AND it MUST NOT include host content, secrets, or an unverified ref in the debug bundle.

#### Scenario: Capture failure does not rewrite build truth

- GIVEN the remote build has already failed and debug capture encounters an I/O, quota, scrub, ingestion, or cleanup error
- WHEN Mantle reports the attempt
- THEN the original failure phase and output-admission state MUST remain unchanged
- AND debug-capture degradation or quarantine MUST be reported as a separate diagnostic fact.

#### Scenario: Retention preserves active replay lease only

- GIVEN debug bundles and captured objects are subject to retention while one bundle has an active inspect or replay lease
- WHEN retention runs
- THEN Mantle MUST preserve the active leased bundle and apply only the accepted bounded deletion plan to eligible roots
- AND expired metadata MUST NOT authorize deletion of ordinary build outputs or unrelated CAS objects.

### Requirement: Remote stateful workspaces use bounded fenced leases
r[remote_builds.stateful_workspace_leases]

Mantle MUST bind each mutable remote workspace lease to a worker identity, authority class, action-compatibility digest, toolchain refs, stable guest mount path, current job, attempt, fence generation, quota policy, and retention class. A stale attempt, different worker, different authority, incompatible action/toolchain, concurrent owner, or unknown cleanup state MUST NOT read or mutate the workspace.

#### Scenario: Current compatible attempt reuses workspace

- GIVEN a worker holds a compatible bounded workspace and the current job/attempt/fence acquires its exclusive lease
- WHEN the remote sandbox starts in mutable-session mode
- THEN Mantle MAY mount the workspace at the declared stable guest path
- AND status/build evidence MUST identify warm-state use and its narrower claim class without exposing host paths or workspace contents.

#### Scenario: Stale or foreign owner is rejected

- GIVEN a workspace lease belongs to a superseded attempt, different worker, different authority class, incompatible action/toolchain, or another active owner
- WHEN a remote request asks to mount, renew, snapshot, scrub, or delete it
- THEN Mantle MUST reject the mutation before sandbox start or filesystem change
- AND current lease and workspace state MUST remain unchanged.

#### Scenario: Failed cleanup quarantines state

- GIVEN a remote build ends and required scrub, bounded scan, snapshot, or cleanup cannot complete
- WHEN Mantle transitions the workspace lease
- THEN it MUST quarantine the workspace and block subsequent reuse
- AND it MUST report cleanup failure separately from execution/output truth while withholding any claim that requires successful cleanup.

#### Scenario: Retention preserves active leases

- GIVEN workspace retention or worker garbage collection runs
- WHEN active, idle, expired, and quarantined workspaces are classified under policy
- THEN Mantle MUST preserve current active leases and apply only the accepted bounded eviction plan
- AND stale metadata or storage pressure MUST NOT authorize deletion of a current workspace.

### Requirement: Remote worker resources use fenced leases
r[remote_builds.fenced_worker_resource_leases]

Mantle MUST reserve quantified worker capacities and named scarce tokens through durable leases bound to the normalized job, current attempt, fence generation, worker identity, requirement digest, and reservation digest before remote assignment. Lease authorization MUST control scheduling capacity only and MUST NOT establish tool identity, output trust, attestation validity, or license compliance.

#### Scenario: Current assignment owns one reservation

- GIVEN a compatible worker has sufficient unreserved capacity
- WHEN the coordinator commits an assignment
- THEN it MUST durably commit exactly one current job/attempt/fence-bound reservation before instructing the worker to execute
- AND concurrent assignments MUST observe the committed remaining capacity.

#### Scenario: Stale attempt cannot mutate capacity

- GIVEN reassignment advanced the current attempt or fence
- WHEN the superseded worker tries to renew, release, resize, or complete its old resource lease
- THEN Mantle MUST reject the mutation with a stale-fence diagnostic
- AND current reservation and job state MUST remain unchanged.

#### Scenario: Restart recovers conservatively

- GIVEN the coordinator or worker restarts with durable active resource leases
- WHEN registrations and job state are reconciled
- THEN Mantle MUST preserve or release each reservation through an explicit current-state transition
- AND it MUST NOT silently double-allocate capacity or report a leaked reservation as successful execution.

#### Scenario: Named token does not confer output trust

- GIVEN a worker obtains a named scarce token such as a licensed-tool seat
- WHEN the build completes
- THEN the token MUST count only as resource authorization evidence
- AND returned outputs MUST still pass ordinary action identity, PathInfo, signature, attestation, and producer-policy admission.

### Requirement: remote_builds.ticket_randomness

r[remote_builds.ticket_randomness]

Mantle SHALL create each remote ticket from fresh operating-system cryptographic randomness obtained by the imperative shell. A pure core SHALL receive that randomness as an explicit input and SHALL reject input that does not meet the named entropy contract.

#### Scenario: Ticket receives valid random input

- **GIVEN** fresh random bytes that meet `TICKET_ENTROPY_BYTES`
- **WHEN** the shell invokes the ticket core
- **THEN** the core SHALL produce one opaque bearer token and public ticket metadata
- **AND** the shell SHALL reveal the bearer token once

#### Scenario: Random input is invalid

- **GIVEN** missing, short, or structurally invalid random input
- **WHEN** the ticket core validates issuance
- **THEN** issuance SHALL fail
- **AND** no ticket state SHALL be written

### Requirement: remote_builds.ticket_verifier_state

r[remote_builds.ticket_verifier_state]

Mantle SHALL persist only a keyed BLAKE3 verifier and public policy metadata for a remote ticket. It SHALL keep the verifier key outside remote state and SHALL NOT persist recoverable bearer material.

#### Scenario: Ticket state is written

- **GIVEN** a newly issued ticket and an active verifier key
- **WHEN** Mantle commits remote state
- **THEN** state SHALL contain the public identifier, key identifier, verifier, and policy metadata
- **AND** state SHALL NOT contain token bytes, an encoded token, the verifier key, or another recoverable bearer representation

#### Scenario: State is copied without the verifier key

- **GIVEN** an attacker who obtains remote state but not the SecretSpec-owned verifier key
- **WHEN** the attacker inspects ticket records
- **THEN** the records SHALL NOT directly disclose a usable bearer token

#### Scenario: Verifier key is retired

- **GIVEN** active tickets bound to a retiring verifier-key identifier
- **WHEN** the operator commits key retirement
- **THEN** Mantle SHALL invalidate those tickets
- **AND** it SHALL require replacement issuance under the new key
- **AND** it SHALL NOT load an undeclared old key or silently extend overlap

### Requirement: remote_builds.ticket_one_time_delivery

r[remote_builds.ticket_one_time_delivery]

Mantle SHALL deliver newly issued bearer material once through an explicit caller-owned secret file descriptor. Normal stdout and stderr SHALL remain free of bearer material, and interactive terminal reveal SHALL require a separate explicit operator action.

#### Scenario: Caller provides a secret sink

- **GIVEN** successful issuance and a valid caller-owned secret file descriptor
- **WHEN** Mantle completes the state commit
- **THEN** it SHALL write the bearer token once to that descriptor
- **AND** it SHALL close its copy of the descriptor
- **AND** it SHALL NOT repeat the value through another output channel

#### Scenario: Secret delivery fails

- **GIVEN** a closed, invalid, or failing secret sink
- **WHEN** Mantle attempts one-time delivery
- **THEN** it SHALL return a typed redacted failure
- **AND** normal stdout, stderr, logs, diagnostics, and evidence SHALL NOT contain the token
- **AND** recovery SHALL require explicit revocation or replacement issuance rather than value replay

### Requirement: remote_builds.ticket_constant_time_verification

r[remote_builds.ticket_constant_time_verification]

Mantle SHALL decode presented ticket material strictly, recompute the keyed verifier, and compare verifier bytes with a constant-time primitive before it applies ticket policy.

#### Scenario: Presented ticket is valid

- **GIVEN** a well-formed token, the matching verifier key, and an active ticket policy
- **WHEN** Mantle authenticates the request
- **THEN** verifier comparison SHALL succeed
- **AND** Mantle SHALL apply expiry, revocation, scope, and use-count policy

#### Scenario: Presented ticket does not match

- **GIVEN** a well-formed but incorrect token
- **WHEN** Mantle authenticates the request
- **THEN** constant-time comparison SHALL reject it
- **AND** diagnostics SHALL NOT distinguish which verifier bytes differed

### Requirement: remote_builds.ticket_nominal_secret_boundary

r[remote_builds.ticket_nominal_secret_boundary]

Mantle SHALL convert structural ticket input into distinct checked ticket, bearer-token, verifier, key-identity, TTL, validity-window, use-limit, remaining-use, build-time-limit, and upload-limit types before authentication or policy evaluation.

#### Scenario: Valid structural credential is admitted

- **GIVEN** a bounded ticket request contains a valid public ID, bearer token, explicit time, and policy limits
- **WHEN** credential admission runs
- **THEN** Mantle SHALL construct distinct checked values before verifier comparison or policy evaluation
- **AND** the pure credential core SHALL retain those roles until a redacted diagnostic or explicit secret sink requires projection

#### Scenario: Malformed credential cannot bypass admission

- **GIVEN** a credential has an empty or oversized ID, malformed bearer token, overflowing TTL, invalid validity window, zero use limit, or excessive resource limit
- **WHEN** direct protocol admission or deserialization runs
- **THEN** Mantle SHALL reject the credential before authentication
- **AND** derived deserialization SHALL NOT bypass checked construction
- **AND** no ticket state SHALL be written or redeemed

#### Scenario: Secret-bearing type reaches a diagnostic

- **GIVEN** an issued or presented bearer token enters an error, debug, report, or serialization path
- **WHEN** Mantle renders that path
- **THEN** it SHALL omit the token, its length, prefix, suffix, digest, and other value-derived data
- **AND** only the explicit one-time secret sink MAY receive the issued bearer value

#### Scenario: Resource limits cannot be exchanged

- **GIVEN** build-time and upload limits use numeric primitive representations
- **WHEN** source passes an upload limit to an API that requires a build-time limit
- **THEN** the source SHALL fail compilation or require an explicit checked conversion

### Requirement: remote_builds.secretspec_service_keys

r[remote_builds.secretspec_service_keys]

Mantle SHALL resolve service-secret values through the pinned SecretSpec Rust SDK in an imperative shell under an explicit profile and `mantle-remote` scope. Provider paths that can launch processes SHALL run inside an owned bounded worker.

#### Scenario: Production key resolves

- **GIVEN** a metadata-only declaration and an allowed systemd credential provider
- **WHEN** Mantle starts the remote service
- **THEN** the shell SHALL resolve the key before accepting requests
- **AND** pure cores SHALL receive only the required opaque key input

#### Scenario: Provider resolution fails

- **GIVEN** a missing declaration, wrong scope, missing provider credential, timeout, cancellation, or oversized provider output
- **WHEN** Mantle starts or rotates a service key
- **THEN** the operation SHALL fail closed
- **AND** no remote authority SHALL be enabled with a fallback value

### Requirement: remote_builds.legacy_ticket_invalidation

r[remote_builds.legacy_ticket_invalidation]

Mantle SHALL require an explicit migration that invalidates all legacy deterministic or plaintext ticket records before remote service startup continues.

#### Scenario: Operator migrates legacy state

- **GIVEN** state that contains legacy ticket secrets
- **WHEN** the operator confirms invalidating migration
- **THEN** Mantle SHALL atomically remove legacy secret fields
- **AND** Mantle SHALL mark the old identifiers invalid
- **AND** the report SHALL require replacement issuance

#### Scenario: Legacy state remains

- **GIVEN** one or more live legacy ticket records
- **WHEN** the remote service starts without completed migration evidence
- **THEN** startup SHALL fail closed
- **AND** the service SHALL NOT authenticate a legacy token

### Requirement: remote_builds.private_atomic_state

r[remote_builds.private_atomic_state]

Mantle SHALL read and replace credential state through a private, no-follow, regular-file boundary with same-directory atomic replacement and strict schema limits.

#### Scenario: Valid state replacement completes

- **GIVEN** a private regular state file and valid next state
- **WHEN** Mantle commits the update
- **THEN** it SHALL write and flush a private temporary file
- **AND** it SHALL atomically replace the target
- **AND** readers SHALL observe either the prior complete state or the next complete state

#### Scenario: State path is unsafe

- **GIVEN** a link, non-regular target, unsafe permissions, oversized input, malformed content, or unknown schema version
- **WHEN** Mantle opens credential state
- **THEN** it SHALL fail closed before authentication or mutation

### Requirement: remote_builds.no_secret_evidence

r[remote_builds.no_secret_evidence]

Mantle SHALL NOT place ticket material, service keys, provider credentials, verifiers, or hashes derived from those values in logs, diagnostics, reports, receipts, snapshots, or lifecycle evidence.

#### Scenario: Credential operation emits evidence

- **GIVEN** issuance, verification, migration, rotation, or failure
- **WHEN** Mantle emits durable evidence
- **THEN** evidence MAY contain public identifiers, schema versions, key identifiers, policy outcomes, and redacted categories
- **AND** evidence SHALL omit every secret and secret-derived value
