# Tasks

## Contract

- [ ] [serial] Define the versioned remote-build protocol, capability negotiation, bounded message/list limits, and fail-closed state-machine validation. r[remote_builds.p2p_protocol]
- [ ] [serial] Define pluggable loopback, stdio, SSH-stdio, and P2P transport-binding semantics, including stdout-as-wire discipline and cheap initial handshakes. r[remote_builds.transport_bindings]
- [ ] [serial] Define session lifecycle semantics for stable session identity, phase-classified failures, retry class reporting, and session-scoped leases or roots. r[remote_builds.session_lifecycle]
- [ ] [serial] Define ticket and trusted-client access policy records, including expiry, use count, build-time limit, upload-byte limit, optional endpoint binding, revocation, secret-safe list/status behavior, and delayed ticket redemption. r[remote_builds.access_tickets]
- [ ] [serial] Define the output-trust preflight and import policy that keeps builder resource access separate from signed output acceptance. r[remote_builds.output_trust_separation]
- [ ] [serial] Define the concrete remote build request model produced after local evaluation/lowering, and reject raw Nickel/front-end module evaluation requests at the remote builder boundary. r[remote_builds.local_eval_concrete_execution]
- [ ] [serial] Define the Mantle-native input manifest, missing-input negotiation, bounded upload, digest verification, and store-prefix-aware closure validation model. r[remote_builds.missing_input_sync]
- [ ] [serial] Define signed output transfer and import semantics, including PathInfo/artifact attestation verification, delta-first negotiation, full-transfer fallback, and substitution reporting. r[remote_builds.signed_delta_output_import]
- [ ] [serial] Define queue/status snapshots and server-side enforcement of concurrency, build-time, upload-byte, list-size, and transfer limits. r[remote_builds.queue_status_limits]

## Implementation

- [ ] [serial] Implement pure core validation for protocol transitions, access policy decisions, ticket lifecycle, input-missing set derivation, output trust decisions, transfer-mode selection, transport failure classification, session-lease planning, limit checks, and status redaction. r[remote_builds.p2p_protocol] r[remote_builds.transport_bindings] r[remote_builds.session_lifecycle] r[remote_builds.access_tickets] r[remote_builds.output_trust_separation] r[remote_builds.missing_input_sync] r[remote_builds.signed_delta_output_import] r[remote_builds.queue_status_limits]
- [ ] [serial] Implement a transport-agnostic `RemoteLink` shell with loopback and stdio/SSH-stdio bindings that preserve stdout protocol discipline before adding the production P2P binding. r[remote_builds.transport_bindings]
- [ ] [serial] Implement a thin `mantle remote serve` shell that opens the configured remote transport endpoint, loads builder state, persists ticket/status data, runs the normal Mantle builder, and never performs frontend-specific evaluation. r[remote_builds.p2p_protocol] r[remote_builds.transport_bindings] r[remote_builds.local_eval_concrete_execution] r[remote_builds.queue_status_limits]
- [ ] [serial] Implement `mantle remote ticket create|inspect|list|reveal|revoke` with secret-safe defaults and explicit reveal behavior. r[remote_builds.access_tickets]
- [ ] [serial] Implement client-side `mantle build --builder <name>` and `mantle build --ticket <ticket>` dispatch that evaluates/lower inputs locally, preflights output trust, uploads only missing inputs, and imports verified signed outputs. r[remote_builds.local_eval_concrete_execution] r[remote_builds.output_trust_separation] r[remote_builds.missing_input_sync] r[remote_builds.signed_delta_output_import]
- [ ] [serial] Integrate remote-build output reports with existing JSON build reports, artifact attestation references, phase-classified failure causes, and substitution mode/byte/fallback fields. r[remote_builds.signed_delta_output_import] r[remote_builds.queue_status_limits] r[remote_builds.session_lifecycle]

## Verification

- [ ] [serial] Add positive tests proving the same protocol core can run over loopback or stdio binding, upload one missing Mantle input, run a concrete build, import signed outputs, and record a verified remote substitution report. r[remote_builds.transport_bindings] r[remote_builds.access_tickets] r[remote_builds.missing_input_sync] r[remote_builds.signed_delta_output_import]
- [ ] [serial] Add negative tests proving expired, revoked, exhausted, and wrong-client tickets are rejected, and malformed pre-queue requests do not consume one-use tickets. r[remote_builds.access_tickets]
- [ ] [serial] Add negative tests proving untrusted builder keys, output signature mismatch, store-prefix mismatch, stale action/derivation identity, and missing attestation proof prevent output import even when builder auth succeeds. r[remote_builds.output_trust_separation] r[remote_builds.signed_delta_output_import]
- [ ] [serial] Add negative tests proving raw Nickel/frontend module evaluation requests, protocol version mismatch, unexpected message order, stdout protocol pollution, oversized lists, upload byte overflow, and input digest mismatch fail closed. r[remote_builds.p2p_protocol] r[remote_builds.transport_bindings] r[remote_builds.local_eval_concrete_execution] r[remote_builds.missing_input_sync] r[remote_builds.queue_status_limits]
- [ ] [serial] Add negative tests proving phase-classified network/config/build/output-trust failures are reported distinctly, stale client object identity cannot cause reconnect spin, and live session leases are released only after a job no longer needs them. r[remote_builds.session_lifecycle]
- [ ] [serial] Add transfer tests proving compatible delta transfer is preferred, delta failure falls back to full transfer with a fallback reason, and failed delta attempts are not reported as successful delta hits. r[remote_builds.signed_delta_output_import]
- [ ] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and the proposal/design/tasks gates before implementation claims, then record focused implementation evidence before checking tasks. r[remote_builds.p2p_protocol]
