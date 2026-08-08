## Goals

- Keep every released object immutable and content-addressed.
- Name one active release through a single current pointer.
- Bind release evidence to the exact object identity and pointer.
- Leave distribution, deployment, retention, and deletion with the caller.

## Release object contract

A released object is immutable and addressed by a BLAKE3 content identity. Publishing an object with the same identity MUST produce the same object; publishing a different identity MUST create a distinct object.

The core accepts object bytes and their declared identity as values. It reads no store, clock, or network.

## Current pointer contract

A single current pointer names one active release identity.

The shell sets the pointer only when the named object exists. A previous release identity is the rollback path. The shell never mutates a published immutable object.

The caller supplies the pointer store and the object store. The core validates the binding facts.

## Evidence binding

Release evidence binds:

1. the object BLAKE3 identity;
2. the current-pointer value;
3. the declared release metadata;
4. the exact digest-role disposition.

The evidence states what was released and which pointer named it. It does not prove deployment or readiness.

## Functional core and shell

The pure core validates identity, pointer, and binding facts.

The shell owns publish, pointer-switch, and evidence-write effects. Distribution, deployment, retention, and deletion stay caller-owned.

## Reference

The Celld release-layout design (immutable releases behind a `current` pointer) is a bounded reference input. It is a comparison source, not a Mantle requirement, parity claim, or equivalence claim.

## Verification

Positive coverage includes an immutable object, a matching identity, a valid pointer switch, and a rollback to a previous identity.

Negative coverage includes a mutated published object, an identity mismatch, a pointer to a missing object, and a digest-role substitution.

Boundary coverage rejects an overwrite of a published object, a missing pointer binding, and a deployment or readiness claim.
