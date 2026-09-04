# Design: Nix-compatible remote service gateway

## Context

Nix clients expect a daemon-store protocol over transports such as `ssh-ng`. Hosted build integrations also need an asynchronous request and observation API. Mantle should provide these edges while retaining its own durable execution semantics.

The gateway is an adapter. It is not a new scheduler, store of truth, cache authority, or evaluator.

## Architecture

```text
Nix client or CI orchestrator
  -> transport and protocol shell
  -> typed authorization context
  -> pure request translation and policy checks
  -> existing Mantle attempt, store, CAS, and scheduler services
  -> bounded observations and signed completion event
```

### Protocol shell

The shell terminates SSH or another configured transport, negotiates a supported Nix daemon protocol version, enforces frame and message limits, and parses operations with the vendored Nix compatibility layer.

Only an explicit operation table is enabled. Concrete store-object transfer, PathInfo queries, substitution checks, and build requests can be mapped to Mantle services. Unsupported operations fail with stable protocol errors. The gateway never evaluates flakes, resolves registries, fetches arbitrary sources on behalf of a caller, or exposes a command shell.

### Functional translation core

The core accepts a parsed operation, authenticated authority facts, service policy, and explicit time or sequence inputs. It returns a typed Mantle command or a rejection. It performs no transport, filesystem, clock, network, store, or scheduler I/O.

The core preserves the concrete-request boundary. A build command identifies admitted derivations and store inputs. It cannot carry evaluator expressions or arbitrary executable commands.

### Existing Mantle services

The imperative adapter calls existing attempt creation, fencing, transfer, locality, lease, scheduler, CAS, PathInfo, cache, and signed-result interfaces. It does not mutate their invariants or combine their authority domains.

A client disconnect does not erase a durable attempt. Reconnection uses an opaque public attempt identifier plus valid read authority. Duplicate submissions use an explicit idempotency key and return the same admitted attempt only when caller and request identities match.

## Authorization

The gateway converts either a hardened compatibility ticket or a verified UCAN into one internal `RemoteAuthority` value. The value carries subject identity, project or account scope, allowed operations, audience, expiry, and evidence references.

Policy checks distinguish at least:

- submit build;
- upload admitted input;
- read store object;
- read status and logs;
- cancel owned attempt;
- publish cache result;
- administer service.

No broad bearer token implies all operations. Compatibility tickets are an explicit migration surface and cannot obtain administration authority. UCAN verification occurs before request translation and uses the existing UCAN verification boundary.

## Build API

The versioned API provides bounded operations for:

- submit a concrete request with an idempotency key;
- inspect one authorized attempt;
- read a bounded log range by cursor;
- read a bounded event page by cursor;
- request cancellation;
- discover signed results;
- read bounded project or account usage summaries.

Every list or byte response has a named policy limit or protocol constant. Cursors are opaque, versioned, and authenticated against query scope. The pure API planner returns typed reason codes. The shell maps failures to stable redacted operator errors.

## Completion events

Mantle appends a completion event when an admitted attempt reaches a terminal state. Completion events include public attempt and request identities, terminal class, result-evidence identity when present, policy identity, sequence, and producer signature.

Delivery is at least once. Consumers deduplicate by event identity and sequence. Events can reference result evidence. They contain no log body or credential.

## Store and cache boundaries

Nix store compatibility does not make the transport authoritative for bytes or PathInfo. Mantle verifies imported content, checks PathInfo and CAS availability, and applies store-write authority before publication.

Result discovery remains separate from execution authority. A signed action result is usable only after current policy, platform identity, output identity, and CAS availability checks pass.

## Failure and abuse controls

The gateway limits connections, negotiations, frames, request sizes, concurrent operations, log windows, event pages, and idle time with named policy values. It rejects malformed lengths before allocation.

Disconnect, timeout, cancellation, duplicate messages, stale fences, invalid cursors, unknown protocol versions, unauthorized operations, and partial uploads have explicit outcomes. Durable attempt ownership and transfer resumption remain intact.

## Evidence and non-claims

The Build API returns bounded operation class, authority evidence identity, attempt identity, byte count, and outcome evidence. Nix protocol metadata separately identifies the supported protocol range and operation table. Neither surface records bearer material or raw OIDC tokens.

Each record carries a typed `RecordedOnly` projection to `valence.build-service-evidence.v1`. The projection is a bundle-assembly input. It is not a complete Valence bundle and does not claim validation by Valence.

Compatibility evidence proves only that selected operations interoperate under the tested client and policy. It does not prove arbitrary Nix compatibility, hermetic builds, sandboxing, output correctness, or release eligibility.

## Rollout

The service starts on a private test endpoint. Interoperability fixtures then cover supported Nix clients and explicit negative cases. Public exposure requires archived credential-hardening evidence and OnixOS deployment policy. The internal Mantle protocol remains available throughout rollout and rollback.
