# ADR 0116: Adapt Nix remote clients without transferring authority

- **Status:** Accepted
- **Date:** 2026-09-02

## Context

Mantle already owns durable remote attempts, fencing, resource leases, transfer checks, store integrity, and signed results. Nix clients use a different daemon-store protocol.

CI clients also need asynchronous submission and observation. A new scheduler or store duplicates authority and weakens the existing remote model.

## Decision

Add a bounded gateway at the transport edge. Use the vendored `nix-compat` daemon handler for the declared Nix store-operation subset.

Keep admission, capability checks, translation, idempotency, reconnect, cursor, and completion-event identity in the pure `crunch-build` gateway core.

Keep protocol I/O, clocks, file descriptors, key loading, persistence, NAR ingestion, and service calls in the binary shell.

Map concrete build submissions to the existing durable coordinator. Preserve its current worker selection, fencing, locality, lease, transfer, and result rules.

Accept only verifier-produced authority facts. Require an exact match between the trusted authority file and the client request.

Keep build, upload, store-read, status-read, log-read, cancellation, publication, and administration capabilities separate. Never grant administration to a compatibility ticket.

Use private endpoint policy by default. Require the archived credential-hardening receipt before public exposure.

## Consequences

- Existing Mantle clients keep the internal remote protocol.
- Nix clients receive only the declared store-operation subset.
- CI clients receive a versioned bounded Build API.
- Client disconnects do not remove durable attempts.
- Exact duplicate submissions recover the same public attempt.
- Gateway cursors are scoped, authenticated, and time-limited.
- Completion events are signed and safe for duplicate delivery.
- Gateway evidence includes a `RecordedOnly` mapping to `valence.build-service-evidence.v1`; Valence still owns complete-bundle validation.
- Transport metadata does not become store or result truth.
- Rollback can remove the gateway endpoint without removing coordinator state.
- Gateway evidence does not prove arbitrary Nix compatibility, evaluator correctness, build correctness, or release eligibility.
