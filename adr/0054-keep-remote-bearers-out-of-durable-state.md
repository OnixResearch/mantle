# ADR 0054: Keep remote bearers out of durable state

## Status

Accepted

## Context

Mantle created remote ticket values from public request fields. It also stored each bearer value in `tickets.json`. A later CLI command could reveal the value.

The remote service also generated result-signing keys in local state. This mixed authentication, secret storage, and result trust with ordinary service state.

## Decision Drivers

- A stolen state file must not provide usable bearer credentials.
- Public ticket fields must not predict credentials.
- Service startup must fail when required keys are unavailable.
- Operators must select the secret profile and provider.
- Provider failures must not select another provider.
- Normal output and evidence must not contain secret-derived values.

## Decision

Mantle gets 32-byte ticket entropy from the operating system. A pure core converts that entropy to canonical base64url without padding.

Mantle stores a keyed BLAKE3 verifier and its key identifier. It does not store the bearer or a reversible bearer form.

Mantle compares recomputed verifier bytes with a constant-time operation. It checks expiry, use count, revocation, and endpoint policy after authentication.

The service supplies current time. A transport supplies authenticated peer identity when it has that capability. Client claims do not provide either authority fact.

Mantle resolves the ticket-verifier key and result-signing key through SecretSpec `0.17.0`. Every request selects an explicit profile and the `mantle-remote` scope.

The `production` profile accepts only `systemd-credential://`. The `bootstrap` profile accepts only a declared `sops://` provider. The `rotation` profile accepts either provider.

SecretSpec runs in a bounded worker process. The parent sets a deadline, process group, memory limit, CPU limit, and retained output limits.

Ticket creation writes the bearer once to a caller-owned file descriptor. Standard output contains only a verifier-safe report. Mantle cannot reveal the bearer later.

Remote build and replay commands also read bearers from caller-owned, non-terminal file descriptors. They do not accept bearer values in argv.

Legacy plaintext state requires an explicit invalidating migration. The migration records old ticket identifiers, removes plaintext values, and requires replacement tickets.

Key rotation invalidates tickets that name a retired verifier key. Mantle does not silently preserve those ticket credentials.

## Alternatives Considered

### Encrypt bearer values in ticket state

Rejected. Decryption would preserve later bearer recovery and increase key exposure in the remote service.

### Store an unkeyed token hash

Rejected. A keyed verifier gives defense against offline guesses and separates verifier state from bearer material.

### Use environment variables as the production provider

Rejected. Ambient environment lookup can add undeclared fallback and inherited-process exposure.

### Keep the reveal command behind a confirmation flag

Rejected. A later reveal requires recoverable bearer storage. The safer design removes that recovery path.

## Consequences

- Operators must provision two SecretSpec values before ticket creation or service startup.
- Lost bearer values cannot be recovered. Operators must revoke or replace the ticket.
- Old plaintext tickets stop working after migration.
- Result-signing trust now follows the external SecretSpec key lifecycle.
- The worker limits bound observations. They do not prove the external provider or its cryptography is correct.
