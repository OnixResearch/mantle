# ADR 0128: Fence remote entry-point bindings and retract future sessions

## Status

Proposed (2026-10-23). Current-source receiver and coordination-socket verification
remains required before change acceptance. Historical receipts are not current
acceptance.

## Context

Remote builders already authenticate tickets, transfers already bind manifests,
attempts and fences, and store sources already enforce their own signatures and
content identities. Those checks do not provide one durable, locally revocable
name admission boundary. A cached descriptor must not resurrect a revoked name,
and revocation must not invalidate content that already passed admission.

## Decision Drivers

- Reject revoked or substituted name bindings before admitting a new session.
- Keep source-specific authentication and content checks independently required.
- Preserve already admitted content while retracting future session eligibility.

## Decision

Use the pure `crunch-remote-core::endpoint` binding table and the owner-local
coordination socket. A binding names its typed purpose, durable name, exact
scope, trust requirement, store prefix, half-open expiry and nonzero generation.
The local daemon creates, resolves and retracts descriptors; a successful reply
is not credential, execution or content authority. Client resolution requires the
exact requested descriptor, and creation requires the exact successor generation
and specification. Wrong-operation or substituted-descriptor replies fail closed.

Receivers independently load their own operator configuration and current durable
table. They compare all requirements and the generation before manifest,
checkpoint or payload processing. The caller's configuration path is never a
receiver configuration path. A shared binding lease remains held through the
operation's independent authentication, quota and effect boundary. Mutations use
an exclusive independent binding lock and reload the current table; ticket state
uses its existing separate mutation lock. Repeated resolution of a current name
therefore retains the same generation without pinning a stale in-memory table.

Revocation retracts exactly the complete descriptor and generation. It neither
deletes admitted content nor changes any content verification rule. Tombstones
remain bounded durable state. Recreation requires the previous generation and
increments it without wrapping; an exhausted generation cannot be recreated.
The exclusive revocation operation waits for already-admitted shared leases;
new admission after successful revocation is refused. This is not cancellation
of work already admitted before revocation.

The production build session resolves its builder-ticket binding before reading
the input manifest and holds that admission through its transfer and ticket
commit. Transfer is part of that session, not a separate currently exposed
`Transfer`-kind receiver. Substituter and base-layer reads independently resolve
their configured typed source binding and retain the existing source-specific
trust and content checks. Generic support for a binding kind alone does not prove
an additional production entry point exists.

## Alternatives considered

- Retain only per-entry ticket, URL and layer checks: rejected as the name
  revocation boundary; those independent payload checks remain mandatory.
- Treat a resolved descriptor as a bearer: rejected; it authenticates neither
  its presenter nor content, and can be copied.
- Cache the binding table across operations: rejected; it can revive revoked
  generations.
- Delete revoked records or reset generations: rejected; an old descriptor can
  otherwise alias a recreated name.
- Serialize every ticket commit under the binding lock: rejected; separate
  binding leases and ticket transactions preserve independent lock ownership.

## Verification and non-claims

The change's existing design records a real `mantle remote live serve` socket
smoke command. Core, persistence, source-admission, production wire and socket
fixtures exercise exact revocation, generation recreation, early silent denial,
and descriptor substitution. They must be run against current sources before
acceptance; this ADR is not an execution receipt.

Name admission does not verify tickets, UCAN proofs, worker execution, signed
PathInfo, output trust, content bytes or release eligibility. Revocation does not
retroactively revoke admitted content, prove atomic persistence under every host
failure, or guarantee cancellation, availability or liveness.
