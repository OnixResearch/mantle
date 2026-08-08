# ADR 0072: Bind immutable release objects to one current pointer

## Status

Accepted

## Context

Mantle can publish release-evidence bundles without replacement. It did not define a separate contract for immutable release objects and one active pointer.

A mutable release object makes rollback ambiguous. A pointer without object and evidence linkage can name missing or substituted bytes.

The Celld release layout uses immutable releases behind one atomic `current` pointer. Mantle uses this pattern as a bounded reference only.

## Decision Drivers

- Keep release objects immutable and content-addressed with BLAKE3.
- Keep one current pointer that names an existing object.
- Preserve the previous identity as the rollback target.
- Bind the object, pointer, metadata, and digest roles in canonical evidence.
- Keep filesystem effects outside the no-std core.
- Keep distribution, deployment, retention, and deletion caller-owned.

## Decision

`crunch-release-core` validates supplied object bytes against a declared BLAKE3 identity. An existing object passes only when its bytes match exactly.

The core plans pointer changes from normalized identity and existence facts. Rollback uses the same plan with a previous release identity as the target.

Canonical evidence binds the object identity and current-pointer identity. It also binds exact object and pointer digest roles and declared metadata.

The evidence records caller authority for distribution, deployment, retention, and deletion. The validator rejects release-readiness claims.

The shell receives explicit object, pointer, and evidence paths from the caller. It does not discover or own a release store.

The shell creates object and evidence files without replacement. It atomically replaces the pointer through a synchronized temporary file and reads it again.

## Alternatives Considered

### Put the pointer inside each immutable object

Rejected. More than one object can then claim to be current, and rollback has no single selection point.

### Use a mutable release directory

Rejected. A changed directory can retain the same visible release name while its bytes change.

### Treat the release identifier as content identity

Rejected. A human release identifier does not prove object-byte identity.

### Add deployment and retention policy

Rejected. These policies belong to callers and release-channel owners, not the Mantle object contract.

## Consequences

- A published identity always names the same bytes at this boundary.
- A pointer can name only an object that the shell reads and validates.
- Rollback changes the pointer and preserves all published objects.
- Evidence fails when object and pointer identities or digest roles differ.
- The contract does not prove distribution, deployment, readiness, retention, or deletion.
- The Celld reference does not create a parity or equivalence claim.
