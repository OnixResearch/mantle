# Design: Harden the remote credential boundary

## Context

`create_ticket_core` currently constructs a seed from public request data and hashes it with BLAKE3. The resulting digest is predictable and is written as the ticket secret in remote-build state. This design has neither secret entropy nor verifier separation.

Mantle already has durable attempts, explicit authority records, signed action-result discovery, and state mutation rails. The correction must preserve those boundaries while changing credential creation and storage.

## Goals

- Make ticket guessing infeasible under the operating-system randomness assumption.
- Make remote state insufficient to recover a usable ticket.
- Keep secret-provider I/O out of pure cores.
- Fail closed during migration and provider failure.
- Produce useful security evidence without secret-derived data.

## Ticket construction

The imperative shell obtains random bytes from the operating-system cryptographic random source. The byte count is a named constant, `TICKET_ENTROPY_BYTES`, selected through a documented security review.

The pure ticket core accepts the random bytes as an input. It constructs:

- a public ticket identifier from public metadata and a BLAKE3 domain-separated identity function;
- an opaque bearer token from the random bytes using a canonical base64url encoding;
- a verifier from keyed BLAKE3 over a domain separator and the decoded token bytes.

The keyed verifier key is separate from state and is resolved through SecretSpec. The shell delivers the bearer token once through an explicit caller-owned secret file descriptor. Normal stdout and stderr never contain it. An interactive terminal reveal requires a separate explicit operator flag. List, inspect, status, and evidence commands never return it.

Presented tokens are decoded strictly, recomputed with the active verifier key, and compared with a constant-time primitive. Ticket policy checks remain explicit and occur only after the verifier comparison has produced an authentication result.

Ticket-verifier key rotation is invalidating by default. State binds each verifier to a public key identifier. Retiring a key invalidates every remaining ticket under that identifier and requires replacement issuance. Mantle does not keep an undeclared old key or silently extend an overlap window.

## Credential nominal boundary

The structural protocol and legacy-state DTOs retain raw text long enough to produce bounded migration and malformed-input diagnostics. One pure admission function converts accepted values into these core roles:

- `TicketId` for bounded public ticket identity;
- `IssuedBearerToken` for one-time secret delivery;
- `PresentedBearerToken` for strict authentication input;
- `TicketVerifier` for keyed BLAKE3 verifier bytes;
- `TicketVerifierKeyId` for public key selection;
- `TicketTtl` for bounded issuance lifetime;
- `TicketValidityWindow` for ordered creation and expiry times;
- `TicketUseLimit` for nonzero issuance policy;
- `TicketUsesRemaining` for state that can reach zero;
- `BuildTimeLimit` for bounded execution duration;
- `UploadByteLimit` for bounded input transfer.

Secret-bearing types do not implement ordinary `Display`, derived `Debug`, or unrestricted serialization. They expose bytes only to verifier construction, constant-time matching, or the explicit caller-owned secret sink. Their redacted debug output contains no length, prefix, suffix, digest, or value-derived data.

The public compatibility parser can continue to accept existing non-empty ticket IDs such as `ticket-1`. Production issuance may derive a stronger public ID, but the nominal type does not add an undocumented digest-only wire rule.

A `UnixSeconds` scalar does not prove time ordering. `TicketValidityWindow::from_ttl` uses checked addition and confirms that expiry follows creation. The shell supplies the observed current time explicitly to pure policy evaluation.

## Functional core and imperative shell

### Functional core

Pure functions validate random-input length, construct public metadata and verifier records, parse state versions, plan migration, evaluate scope and expiry, and classify authentication outcomes. Tests inject deterministic random bytes and explicit time values.

### Imperative shell

The shell obtains OS randomness, resolves service keys, reads and atomically replaces state, sets file permissions, rejects links and non-regular files, and sends a new bearer token once to the explicit secret sink. No core reads files, environment values, process state, clocks, or providers.

## SecretSpec integration

Mantle pins SecretSpec `v0.17.0` and records tag commit `a8794e46ec9664a0e1a3869cc3105d0853937e48`. `secretspec.toml` declares metadata for service keys but contains no values.

The service uses an explicit profile and `mantle-remote` scope. Production profiles prefer the systemd credential provider. SOPS is allowed only for operator-controlled bootstrap or rotation profiles.

Because the upstream SOPS provider can launch an unbounded child process, Mantle executes SecretSpec resolution in an internal bounded worker. The parent receives one bounded value through a private pipe. SecretSpec write operations and plaintext value caches are disabled.

## State schema and migration

A new schema version replaces each `secret` field with verifier metadata, key identifier, public policy, and issuance facts. The state does not contain the verifier key.

The migration is intentionally invalidating:

1. Detect every legacy ticket record.
2. Require an explicit `invalidate-legacy` operator action.
3. Remove legacy secret fields through an atomic state replacement.
4. Mark old ticket identifiers invalid.
5. Emit counts and public identifiers only.
6. Require new ticket issuance after the rewrite.

Mantle does not claim that rewriting one file deletes backups, snapshots, logs, or copied state. Operator guidance requires rotation and copy review.

## State I/O hardening

The shell validates the parent directory and target type, rejects symbolic links, creates private temporary files in the same directory, flushes file content, applies private permissions, atomically renames, and flushes the parent directory when the platform supports it.

Malformed, unknown-version, overly large, or partially migrated state fails closed. Recovery requires an explicit operator command and never recreates bearer material from metadata.

## Evidence and diagnostics

Evidence can include the public ticket identifier, state-schema version, verifier-key identifier, operation class, migration counts, policy outcome, and redacted error category. It cannot include token bytes, plaintext credentials, verifier bytes, provider credentials, or hashes derived from token or key material. Secret-sink failures report only a typed category and close the sink without retrying token delivery to another channel.

## Rollout

1. Add read support for the new state schema and the invalidating migration command.
2. Block remote service startup when legacy live tickets remain.
3. Run migration and rotate all tickets and service keys.
4. Enable new ticket issuance.
5. Remove legacy schema read support in a later change after archived migration evidence exists.

Rollback restores the prior binary only for offline recovery. It does not restore invalidated ticket authority.

## Validation

Positive tests cover ticket issuance, one-time reveal, valid verification, policy checks, state round trips, and migration completion. Negative tests cover deterministic or short random input, wrong keys, malformed tokens, timing-safe mismatch paths, replay limits, expiry, revocation, malformed state, links, permissions, interrupted writes, provider failures, and attempted secret disclosure.
