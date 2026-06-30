## ADDED Requirements

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
