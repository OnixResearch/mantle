# Change: Harden the remote credential boundary

## Why

Mantle currently derives a remote ticket secret from predictable public inputs and BLAKE3. Hashing predictable inputs does not create secret entropy. The same ticket value is also persisted in plaintext JSON. An attacker who can infer the inputs or read copied state can impersonate a ticket holder.

This is a production blocker for any public remote-build service. Mantle needs random one-time credentials, verifier-only state, explicit migration, and a reviewed service-secret boundary before wider Nix or CI compatibility is enabled.

## What Changes

- Generate ticket material from operating-system cryptographic randomness in the imperative shell. Keep ticket policy and verifier construction in pure functions.
- Deliver a ticket secret once through an explicit caller-owned secret sink. Keep normal output free of bearer material and persist only a keyed BLAKE3 verifier plus public metadata.
- Represent ticket IDs, bearer tokens, verifier values, key IDs, validity windows, use limits, build-time limits, and upload limits with checked nominal types.
- Verify presented ticket material in constant time and preserve explicit expiry, revocation, scope, use-count, and verifier-key rotation policy.
- Add an explicit migration that invalidates legacy deterministic and plaintext tickets, removes their secret fields, and requires replacement issuance.
- Pin the SecretSpec Rust SDK at `v0.17.0` for Mantle service secrets, including the ticket-verifier key and result-signing key.
- Resolve SecretSpec values only in a bounded imperative worker and prefer systemd credentials for deployed services.
- Harden state-file creation, replacement, permissions, symlink handling, diagnostics, and evidence.
- Add positive and negative tests for entropy injection, typed admission, verifier behavior, migration, corruption, replay, expiry, scope, redaction, and provider failure.

## Non-Goals

- Password hashing or a human-password KDF.
- Preservation of unsafe legacy ticket validity.
- Storing ticket plaintext in another file or receipt.
- Treating SecretSpec as authorization policy.
- Replacing short-lived UCAN capabilities for identity-aware service access.

## Dependencies

- SecretSpec `v0.17.0`, upstream tag commit `a8794e46ec9664a0e1a3869cc3105d0853937e48`.
- OS cryptographic random source.
- `bounded-exec` or Mantle's equivalent owned bounded-process mechanism for providers that launch child processes.

## Impact

- **Affected specs:** `remote-builds`
- **Affected code:** remote ticket nominal types, creation and verification, remote state schema, state I/O shell, service-key loading, migration command, and diagnostics
- **Operator impact:** all legacy tickets become invalid and must be reissued
- **Security impact:** this change must archive before the public remote-service gateway is enabled
