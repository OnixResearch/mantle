## Why

Mantle owns bounded build, cache, and release evidence. It records release evidence but does not yet express a release-layout contract that keeps every released object immutable behind one current pointer.

Celld keeps verified releases as immutable objects behind one atomic `current` pointer; a previous SHA is the rollback. That discipline transfers directly: a released object is immutable and content-addressed, the current pointer names one active release, and rollback selects a previous release by its identity.

Mantle should own the immutable-release and current-pointer evidence contract. The caller owns distribution, deployment, retention, and deletion authority.

## What Changes

- Add an immutable-release object contract keyed by content identity (BLAKE3).
- Add a single current-pointer contract that names one active release.
- Bind release evidence to the exact object identity and current pointer.
- Make a previous release identity the rollback path; never mutate a published immutable object.
- Add positive, negative, and boundary fixtures for pointer, rollback, and immutability cases.
- Reference the reviewed Celld release-layout design as a bounded, non-parity input.

## Impact

The change gives callers a truthful, evidence-bound release layout. It does not distribute, deploy, or retain objects.

Release objects and receipts remain BLAKE3.

## Dependencies

This change has no downstream prerequisite.

## Non-goals

- Do not add a distribution channel or a deployment agent.
- Do not implement a store or an object cache.
- Do not claim durable retention or deletion authority.
- Do not claim parity with, or equivalence to, the Celld installer.
- Do not convert a release-pointer binding into a proof of deployment or release readiness.
