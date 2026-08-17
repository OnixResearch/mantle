# Remote Builds Credential Delta

## ADDED Requirements

### Requirement: remote_builds.ticket_randomness

r[remote_builds.ticket_randomness]

Mantle SHALL create each remote ticket from fresh operating-system cryptographic randomness obtained by the imperative shell. A pure core SHALL receive that randomness as an explicit input and SHALL reject input that does not meet the named entropy contract.

#### Scenario: Ticket receives valid random input

- **GIVEN** fresh random bytes that meet `TICKET_ENTROPY_BYTES`
- **WHEN** the shell invokes the ticket core
- **THEN** the core SHALL produce one opaque bearer token and public ticket metadata
- **AND** the shell SHALL reveal the bearer token once

#### Scenario: Random input is invalid

- **GIVEN** missing, short, or structurally invalid random input
- **WHEN** the ticket core validates issuance
- **THEN** issuance SHALL fail
- **AND** no ticket state SHALL be written

### Requirement: remote_builds.ticket_verifier_state

r[remote_builds.ticket_verifier_state]

Mantle SHALL persist only a keyed BLAKE3 verifier and public policy metadata for a remote ticket. It SHALL keep the verifier key outside remote state and SHALL NOT persist recoverable bearer material.

#### Scenario: Ticket state is written

- **GIVEN** a newly issued ticket and an active verifier key
- **WHEN** Mantle commits remote state
- **THEN** state SHALL contain the public identifier, key identifier, verifier, and policy metadata
- **AND** state SHALL NOT contain token bytes, an encoded token, the verifier key, or another recoverable bearer representation

#### Scenario: State is copied without the verifier key

- **GIVEN** an attacker who obtains remote state but not the SecretSpec-owned verifier key
- **WHEN** the attacker inspects ticket records
- **THEN** the records SHALL NOT directly disclose a usable bearer token

#### Scenario: Verifier key is retired

- **GIVEN** active tickets bound to a retiring verifier-key identifier
- **WHEN** the operator commits key retirement
- **THEN** Mantle SHALL invalidate those tickets
- **AND** it SHALL require replacement issuance under the new key
- **AND** it SHALL NOT load an undeclared old key or silently extend overlap

### Requirement: remote_builds.ticket_one_time_delivery

r[remote_builds.ticket_one_time_delivery]

Mantle SHALL deliver newly issued bearer material once through an explicit caller-owned secret file descriptor. Normal stdout and stderr SHALL remain free of bearer material, and interactive terminal reveal SHALL require a separate explicit operator action.

#### Scenario: Caller provides a secret sink

- **GIVEN** successful issuance and a valid caller-owned secret file descriptor
- **WHEN** Mantle completes the state commit
- **THEN** it SHALL write the bearer token once to that descriptor
- **AND** it SHALL close its copy of the descriptor
- **AND** it SHALL NOT repeat the value through another output channel

#### Scenario: Secret delivery fails

- **GIVEN** a closed, invalid, or failing secret sink
- **WHEN** Mantle attempts one-time delivery
- **THEN** it SHALL return a typed redacted failure
- **AND** normal stdout, stderr, logs, diagnostics, and evidence SHALL NOT contain the token
- **AND** recovery SHALL require explicit revocation or replacement issuance rather than value replay

### Requirement: remote_builds.ticket_constant_time_verification

r[remote_builds.ticket_constant_time_verification]

Mantle SHALL decode presented ticket material strictly, recompute the keyed verifier, and compare verifier bytes with a constant-time primitive before it applies ticket policy.

#### Scenario: Presented ticket is valid

- **GIVEN** a well-formed token, the matching verifier key, and an active ticket policy
- **WHEN** Mantle authenticates the request
- **THEN** verifier comparison SHALL succeed
- **AND** Mantle SHALL apply expiry, revocation, scope, and use-count policy

#### Scenario: Presented ticket does not match

- **GIVEN** a well-formed but incorrect token
- **WHEN** Mantle authenticates the request
- **THEN** constant-time comparison SHALL reject it
- **AND** diagnostics SHALL NOT distinguish which verifier bytes differed

### Requirement: remote_builds.ticket_nominal_secret_boundary

r[remote_builds.ticket_nominal_secret_boundary]

Mantle SHALL convert structural ticket input into distinct checked ticket, bearer-token, verifier, key-identity, TTL, validity-window, use-limit, remaining-use, build-time-limit, and upload-limit types before authentication or policy evaluation.

#### Scenario: Valid structural credential is admitted

- **GIVEN** a bounded ticket request contains a valid public ID, bearer token, explicit time, and policy limits
- **WHEN** credential admission runs
- **THEN** Mantle SHALL construct distinct checked values before verifier comparison or policy evaluation
- **AND** the pure credential core SHALL retain those roles until a redacted diagnostic or explicit secret sink requires projection

#### Scenario: Malformed credential cannot bypass admission

- **GIVEN** a credential has an empty or oversized ID, malformed bearer token, overflowing TTL, invalid validity window, zero use limit, or excessive resource limit
- **WHEN** direct protocol admission or deserialization runs
- **THEN** Mantle SHALL reject the credential before authentication
- **AND** derived deserialization SHALL NOT bypass checked construction
- **AND** no ticket state SHALL be written or redeemed

#### Scenario: Secret-bearing type reaches a diagnostic

- **GIVEN** an issued or presented bearer token enters an error, debug, report, or serialization path
- **WHEN** Mantle renders that path
- **THEN** it SHALL omit the token, its length, prefix, suffix, digest, and other value-derived data
- **AND** only the explicit one-time secret sink MAY receive the issued bearer value

#### Scenario: Resource limits cannot be exchanged

- **GIVEN** build-time and upload limits use numeric primitive representations
- **WHEN** source passes an upload limit to an API that requires a build-time limit
- **THEN** the source SHALL fail compilation or require an explicit checked conversion

### Requirement: remote_builds.secretspec_service_keys

r[remote_builds.secretspec_service_keys]

Mantle SHALL resolve service-secret values through the pinned SecretSpec Rust SDK in an imperative shell under an explicit profile and `mantle-remote` scope. Provider paths that can launch processes SHALL run inside an owned bounded worker.

#### Scenario: Production key resolves

- **GIVEN** a metadata-only declaration and an allowed systemd credential provider
- **WHEN** Mantle starts the remote service
- **THEN** the shell SHALL resolve the key before accepting requests
- **AND** pure cores SHALL receive only the required opaque key input

#### Scenario: Provider resolution fails

- **GIVEN** a missing declaration, wrong scope, missing provider credential, timeout, cancellation, or oversized provider output
- **WHEN** Mantle starts or rotates a service key
- **THEN** the operation SHALL fail closed
- **AND** no remote authority SHALL be enabled with a fallback value

### Requirement: remote_builds.legacy_ticket_invalidation

r[remote_builds.legacy_ticket_invalidation]

Mantle SHALL require an explicit migration that invalidates all legacy deterministic or plaintext ticket records before remote service startup continues.

#### Scenario: Operator migrates legacy state

- **GIVEN** state that contains legacy ticket secrets
- **WHEN** the operator confirms invalidating migration
- **THEN** Mantle SHALL atomically remove legacy secret fields
- **AND** Mantle SHALL mark the old identifiers invalid
- **AND** the report SHALL require replacement issuance

#### Scenario: Legacy state remains

- **GIVEN** one or more live legacy ticket records
- **WHEN** the remote service starts without completed migration evidence
- **THEN** startup SHALL fail closed
- **AND** the service SHALL NOT authenticate a legacy token

### Requirement: remote_builds.private_atomic_state

r[remote_builds.private_atomic_state]

Mantle SHALL read and replace credential state through a private, no-follow, regular-file boundary with same-directory atomic replacement and strict schema limits.

#### Scenario: Valid state replacement completes

- **GIVEN** a private regular state file and valid next state
- **WHEN** Mantle commits the update
- **THEN** it SHALL write and flush a private temporary file
- **AND** it SHALL atomically replace the target
- **AND** readers SHALL observe either the prior complete state or the next complete state

#### Scenario: State path is unsafe

- **GIVEN** a link, non-regular target, unsafe permissions, oversized input, malformed content, or unknown schema version
- **WHEN** Mantle opens credential state
- **THEN** it SHALL fail closed before authentication or mutation

### Requirement: remote_builds.no_secret_evidence

r[remote_builds.no_secret_evidence]

Mantle SHALL NOT place ticket material, service keys, provider credentials, verifiers, or hashes derived from those values in logs, diagnostics, reports, receipts, snapshots, or lifecycle evidence.

#### Scenario: Credential operation emits evidence

- **GIVEN** issuance, verification, migration, rotation, or failure
- **WHEN** Mantle emits durable evidence
- **THEN** evidence MAY contain public identifiers, schema versions, key identifiers, policy outcomes, and redacted categories
- **AND** evidence SHALL omit every secret and secret-derived value
