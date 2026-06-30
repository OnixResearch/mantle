# Current blocker: P2P remote builders

Date: 2026-06-30

## Current state

The package now captures the first pure/access slice in `src/remote_build.rs` plus the newly specified transport/lifecycle scope:

- protocol metadata for `mantle-remote-build/1`;
- ticket creation/list/inspect/reveal/revoke state and redaction;
- version/ALPN/endpoint/capability validation;
- delayed ticket redemption for valid queued requests;
- raw frontend/Nickel request rejection in pure request-shape validation;
- missing-input set derivation and output-trust separation helpers;
- a diagnostic `mantle remote serve` surface that does not bind a transport;
- Cairn requirements for pluggable loopback/stdio/SSH-stdio/P2P transport bindings, stdout-as-wire discipline, cheap handshakes, phase-classified failures, stable session identity, and session-scoped leases.

## Blocker

Still not drainable as a complete remote-builder product surface. The active tasks require a concrete authenticated transport binding stack, server session handling, client-side `mantle build --builder/--ticket` dispatch, CAS/source input upload, real build execution on the remote side, signed output transfer/import, delta/full fallback integration, redacted queue/status snapshots, stdio/loopback positive tests, and lifecycle negative tests.

The design intentionally keeps the protocol independent of the binding. Completing the change honestly still requires a product decision for the production network transport and wire encoding before session compatibility can be claimed.

## Next best step

Decide the concrete frame schema and first implementation binding: loopback/stdio for deterministic integration proof, then Iroh or another authenticated P2P binding for production networking. Then split or implement the next slice around a real loopback or stdio session: ticket auth → one missing input upload → concrete build → signed output import → build report evidence.
