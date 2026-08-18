# Change: Capture Mantle artifact-auth operational receipts

## Why

Mantle now signs and independently verifies exact `artifact-auth.statement.v1` bytes, but the result is transient and key currentness is supplied directly by the caller. Operators need a durable receipt tied to Mantle's existing trusted-key, revocation, and validity-window context without changing action-result authority.

## What Changes

- Derive a bounded key-currentness observation from `TrustVerificationContext` using full Nix verifying-key material, configured revocations, validity time, and policy identity.
- Build a deterministic BLAKE3-identified operational receipt from the standalone carrier, independently recomputed shell report, and trust observation.
- Persist and reload receipts under the local action-result state root, then re-evaluate them against a fresh product trust context.
- Keep action-result admission authoritative, standalone authority disabled, and rollback explicit.

## Scope

This change owns local receipt identity, local persistence, replay validation, tests, and operator documentation. It does not add network trust discovery, revocation-list refresh, registry publication, build admission, release eligibility, or authority cutover.
