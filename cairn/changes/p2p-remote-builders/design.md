# Design: P2P remote builders

## Architecture

Remote building is a build-tool feature, not a frontend feature. The client evaluates Nickel or accepts already-lowered build inputs locally, then sends a concrete remote-build request that the builder can validate without understanding frontend module semantics. The builder runs Mantle's normal build executor against admitted inputs and returns output metadata and transfer capability data.

Keep the functional core pure: request validation, ticket policy checks, protocol state transitions, input-missing set calculation, output trust decisions, failure classification, session-lease planning, and status redaction should be deterministic functions over in-memory data. The imperative shell owns transport endpoints, filesystem/state reads, clocks, store locks, actual blob transfer, process execution, and stdout/stderr.

Client-facing code should cross one remote boundary. Store-input negotiation, build execution, logs, output transfer, and status are messages on the same `RemoteBuilderSession`; they should not grow separate remote providers per domain.

## Protocol shape

Use a versioned ALPN such as `mantle-remote-build/1`. The control stream should be a small explicit state machine:

1. `Hello { version, endpoint_id, capabilities }`
2. `AuthTicket` or `AuthTrustedClient`
3. `AuthOk { builder_signing_keys, accepted_capabilities }`
4. `BuildRequest { request_id, action_or_derivation_graph, requested_outputs, hermeticity_mode, output_mode }`
5. `InputManifest` chunks from client
6. `MissingInputs` chunks from builder
7. `InputUploadReady` and bounded data streams for missing CAS objects/source inputs
8. `BuildQueued`, `BuildStarted`, `BuildLog`, `BuildFinished`
9. `OutputClosure`, `OutputTransferReady`, `OutputTransferDone`, `Done` or `Error`

Every variable-length list must have explicit chunk, item, and byte limits. Version mismatch, endpoint identity mismatch, unexpected message order, unsupported capabilities, and oversized messages fail closed.

`ConcreteBuildRequest` now carries bounded executable payload identity: either an action spec (`action_id`, serialized concrete action JSON) or a derivation spec (`drv_path`, serialized derivation JSON), plus explicit expected output names and logical paths. Raw frontend/Nickel evaluation remains rejected at the remote boundary. This model is intentionally still a transport payload contract, not proof that the fixture server executes the payload yet.

## Frame and first-binding decision

The first concrete frame format is `u32be-length-prefixed-json`: a four-byte big-endian payload length followed by one JSON `RemoteFrame` payload. The decoder rejects incomplete headers, payload lengths above `MAX_REMOTE_FRAME_BYTES`, JSON errors, and trailing or missing bytes. This makes stdio pollution deterministic: ordinary human stdout starts with bytes that cannot satisfy the frame contract and fails before queue admission.

Preserves is a plausible later codec because it has a syntax-neutral data model, Rust support, schema tooling, and canonical binary syntax. Do not switch this change to Preserves while the request/executor semantics are still moving: the current validation and CLI fixture depend on readable framed JSON evidence. Revisit Preserves when the protocol fields stabilize and the work is mostly codec/schema generation rather than behavioral semantics.

The first implementation binding is in-process loopback over the same `RemoteFrame` state machine. Stdio and SSH-stdio use the same length-prefixed frame contract when the shell grows process spawning. Production P2P remains a later binding and must not add a second protocol core.

The stdio binding shell validates child output with the same frame decoder: stdout is decoded only as a sequence of length-prefixed frames, stderr is retained as bounded diagnostics, and child exit failure or unframed stdout is classified before queue admission. This keeps human logs from becoming control data while preserving stderr for operator debugging.

A deterministic `serve_stdio_remote_once` seam now exercises the same exchange over generic `Read`/`Write`: read bounded client frame stream, validate Hello/auth/build/input/upload order, plan missing inputs, redeem the ticket only after valid upload, and write framed builder responses. `mantle remote serve --binding stdio-once` exposes this as an explicit operator/test fixture that reads one framed stdin exchange, persists ticket redemption, and writes only framed responses to stdout. The default `remote serve` remains metadata-only; a production daemon loop and real build executor are still separate work.

## Transport bindings and stdio discipline

The remote-build state machine is independent of its byte transport. A `RemoteLink` binding can be in-process loopback, child stdio, SSH-stdio, or a NAT-friendly P2P stream such as Iroh. Each binding presents authenticated ordered frames to the same protocol core; transport-specific setup, keepalive, process spawning, and endpoint discovery stay in the imperative shell.

Stdio mode is deliberately strict. Stdout is reserved for framed remote-build protocol bytes. Logs, progress diagnostics, tracing, and human-readable errors must go to stderr or through an explicit framed `BuildLog` message. Any unframed stdout byte is a protocol error and must fail the session before queue admission or output import. Client-side stdio exchange validation now turns a successful child stdout frame stream into an output-admission report and classifies untrusted remote output keys as `output-import` failures, not transport failures. The integration fixture starts the real `mantle remote serve --binding stdio-once` binary, sends framed stdin, validates framed stdout, and checks ticket redemption in state.

The initial handshake must be cheap. `Hello`, authorization, and capability reporting should complete before expensive store scans, process-table-like host probes, missing-input walks, or build scheduling. Remote platform and capability facts come from Mantle's own capability message and builder configuration, not from a kernel-name shortcut such as `uname`.

## Access model

The server supports two access paths:

- Trusted clients configured by endpoint id and policy.
- Bearer tickets that embed the builder endpoint address plus a generated secret.

Ticket state lives on the builder and includes display name, creation time, expiry time, uses remaining, maximum build time, maximum upload bytes, optional bound client endpoint id, and revoked state. Operators can create, inspect, list, reveal, and revoke tickets. List and status views must never print bearer secrets; reveal is the explicit secret-disclosure command.

The server may check a ticket during auth, but it should redeem uses only after the build request has passed shape validation and the job has entered the queue. That prevents malformed requests from consuming one-use tickets.

## Output trust model

Builder access and output import trust are separate. A ticket only authorizes CPU, disk, and upload quota on the builder. The client must still trust the builder signing key, release/attestation policy, or configured remote-output authority before importing PathInfo or artifact attestations. If trust is missing, the client can fail before dispatch or after auth preflight, but it must not accept remote outputs simply because auth succeeded.

Remote output import should reuse Mantle's existing signed PathInfo, artifact attestation, and substitution report surfaces. The builder signs output PathInfo and attaches artifact/closure attestations. The client verifies signatures, store prefix, expected output identity, and action/derivation match before accepting the result.

The current bounded admission core validates framed builder results before any import side effect: request id and store prefix must match the original concrete request, the output digest must be canonical lowercase BLAKE3 hex, the transfer report must be self-consistent, the transfer key must match the build result key, and that key must be trusted by the client. This is still pre-persistence validation; durable signed PathInfo/artifact import remains separate work.

## Input synchronization

The client sends an input manifest containing concrete derivations/action specs, PathInfo refs, source-input refs, CAS object refs, and declared closure metadata. The builder computes the missing set against its local state, replies with only missing refs, and accepts bounded upload streams for those refs. Uploads must verify content digests and PathInfo signatures before the build sandbox starts.

This is a Mantle-native replacement for drv-thru's `nix-store --export` path. It must not assume `/nix/store` and must respect the configured logical store prefix.

## Output transfer

After a successful build, the builder computes the requested output closure and advertises transfer capabilities. The client should prefer delta transfer when both sides support compatible protocol versions and candidate manifests. It falls back to full object/NAR/castore transfer when delta is unavailable or fails. The build report records mode, transferred bytes, reused bytes, fallback reason when present, builder identity, and verified signing key identity.

## Status and limits

The server persists a redacted status snapshot with endpoint id, configured concurrency, queued jobs, active jobs, recent jobs, phases, durations, and short errors. Status must redact bearer ticket secrets, uploaded path contents, environment values, and any untrusted log payload that could be interpreted as structured control data.

Concurrency, maximum build time, maximum upload bytes, path/object list limits, and parallel transfer limits are enforced by the server. The client receives deterministic diagnostics when a limit is exceeded.

## Session lifecycle and failure taxonomy

Each remote spawn or connection attempt has a stable session identity that survives client proxy/object recreation. Reconnect logic compares this stable identity, not a freshly materialized client object, so a dead transport cannot trigger an event-loop-speed reconnect spin.

Failures are classified by phase before reporting or retry. Transport and network failures are retryable according to client policy. Authentication, ticket, builder configuration, capability, and output-trust failures are terminal until operator action changes the inputs. Build failures remain build outcomes, not transport failures. Every user-requested reconnect or build dispatch must surface the real error instead of resolving success from stale state.

Live remote-build artifacts need explicit leases or roots scoped to the active session or queued job. Helper binaries, uploaded inputs, produced outputs, and temporary transfer state must not be collected while they can still be referenced, and they should become collectible when the session/job lease is replaced or released.

## Validation strategy

- Pure core tests for protocol state transitions, ticket validation, output-trust selection, input-missing calculation, transport-failure classification, session-lease planning, list limit checks, and status redaction.
- Positive integration test with a local in-process or stdio/loopback fixture: ticket auth, one missing input upload, remote build success, signed output import, and build report substitution details. A later Iroh fixture should reuse the same protocol core rather than a second remote-build implementation.
- Negative tests for expired/revoked/bound tickets, malformed requests not consuming one-use tickets, stdout protocol pollution in stdio mode, untrusted builder signing key, version mismatch, oversized lists, digest mismatch on uploaded input, output signature mismatch, reconnect spin guards, and delta failure falling back to full transfer without claiming a delta hit.
