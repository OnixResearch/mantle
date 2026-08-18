# Change: Add a Nix-compatible remote service gateway

## Why

Mantle has a stronger internal remote-execution model than a conventional Nix builder. It already owns durable attempts, fencing, resumable transfer, verified locality, resource leases, signed result discovery, and separate cache and execution authority.

Nix and CI users still need standard compatibility surfaces. A thin gateway can accept Nix `ssh-ng` remote-store traffic and a bounded asynchronous Build API without weakening Mantle's internal protocol or accepting broad evaluator authority.

## What Changes

- Add a versioned Nix `ssh-ng` and daemon-store compatibility gateway using Mantle's vendored Nix protocol support.
- Translate supported concrete store and build operations into existing durable Mantle attempts.
- Add a bounded Build API for submission, status, log windows, event cursors, cancellation, result discovery, and public accounting summaries.
- Emit signed, idempotent completion events for orchestrators such as Lattice.
- Accept typed authorization contexts from hardened compatibility tickets or Basalt-issued UCAN capabilities.
- Preserve separate authority for execution, store upload, substitution, cache publication, log access, cancellation, and administration.
- Preserve CAS verification, PathInfo truth, attempt fencing, transfer resumption, and signed action-result checks behind the gateway.
- Add positive interoperability tests and negative protocol, authorization, disconnect, replay, and resource-bound tests.

## Non-Goals

- Replacing Mantle's internal protocol with the Nix daemon protocol.
- Server-side flake evaluation, source fetching, or arbitrary command execution.
- A terminal administration shell.
- Opaque limits or unbounded log streaming.
- Cross-project result reuse without explicit sharing policy.
- Resource prediction or OOM retry policy, which is a separate change.

## Dependencies

- `harden-remote-credential-boundary` must archive before public exposure.
- Basalt OIDC-to-UCAN exchange for identity-aware CI access.
- Valence build-service evidence profile for canonical cross-project linkage.

## Impact

- **Affected specs:** `remote-builds`
- **Affected code:** protocol gateway, API shell, authorization adapter, event journal, log reader, CLI, metrics, and interoperability fixtures
- **Compatibility:** existing Mantle clients continue to use the stronger internal protocol
