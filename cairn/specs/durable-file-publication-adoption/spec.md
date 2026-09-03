# Durable File Publication Adoption Specification

## Purpose

Defines the `durable-file-publication-adoption` capability.

## Requirements

### Requirement: Immutable reviewed Radicle source

r[mantle.durable_file_publication.source]

Mantle MUST consume `durable-file-publication` from RID `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM` through the governed Radicle HTTPS adapter at exact revision `951c27f59003cea9bfdb40ed4d89653d50fada1f`, with aligned Cargo and Nix identities and no executable sibling or GitHub fallback.

#### Scenario: Exact source is admitted

- GIVEN the reviewed RID, revision, package, license, Cargo lock, and Nix lock
- WHEN source-admission validation runs
- THEN every identity MUST match the reviewed source.

#### Scenario: Source identity drifts

- GIVEN a changed URL, RID, revision, package, license, lock source, sibling path, or GitHub fallback
- WHEN source-admission validation runs
- THEN adoption MUST fail closed.

### Requirement: Exact immutable-object request mapping

r[mantle.durable_file_publication.mapping]

Mantle MUST map one validated immutable remote-attempt segment, remote-attempt anchor, source record, source observation, or source pin to the exact destination leaf, payload byte count, payload limit, final mode, collision bound, no-replace behavior, and durability requirement without moving product policy into the shared crate.

#### Scenario: Valid immutable object maps losslessly

- GIVEN canonical bounded remote-attempt or source-state bytes and a validated destination
- WHEN Mantle builds the shared publication request
- THEN every mechanical field MUST match the Mantle-owned facts.

#### Scenario: Invalid object facts are supplied

- GIVEN an unsafe leaf, empty limit, oversized payload, invalid mode, or zero collision bound
- WHEN request admission runs
- THEN no filesystem mutation MUST occur.

### Requirement: Capability-relative durable adapter

r[mantle.durable_file_publication.adapter]

Mantle MUST open and authorize one no-follow parent directory before invoking the shared shell, and the shared shell MUST use only that parent for exclusive stage creation, exact writes, permissions, payload synchronization, no-replace rename, cleanup, and post-rename parent synchronization.

#### Scenario: Durable no-replace publication succeeds

- GIVEN a synchronizable opened parent, an absent destination, and valid bytes
- WHEN immutable publication runs on Linux
- THEN the result MUST be `CommittedAndParentSynchronized` and the exact bytes MUST exist at the destination.

#### Scenario: Parent cannot synchronize

- GIVEN an opened parent capability that cannot perform real synchronization
- WHEN durability-required preflight runs
- THEN publication MUST fail before stage creation.

### Requirement: Complete product outcome mapping

r[mantle.durable_file_publication.outcomes]

Mantle MUST preserve every shared commit, primary-failure, and cleanup distinction. It MUST interpret existing content only after `DestinationExists`, and it MUST keep `CommittedDurabilityUnknown` distinct from uncommitted failure and durable success.

#### Scenario: Existing content is identical

- GIVEN no-replace observes an existing regular file with exact expected bytes
- WHEN Mantle performs its bounded no-follow comparison
- THEN the operation MUST be accepted as idempotent without editing the destination.

#### Scenario: Existing content differs or is unsafe

- GIVEN existing bytes differ or the destination is a symlink or non-regular file
- WHEN Mantle performs product interpretation
- THEN publication MUST fail as an immutable conflict or unsafe destination.

#### Scenario: Parent synchronization fails after rename

- GIVEN rename committed and parent synchronization failed
- WHEN the shared result is mapped
- THEN Mantle MUST report committed durability unknown without cleanup or automatic rollback.

### Requirement: Mantle retains authority and rollback

r[mantle.durable_file_publication.authority]

Mantle MUST retain canonical JSON, object identity, limits, existing-content equivalence, source-ingest policy, source readiness, manifest replacement, chain validation, retention, deletion, receipts, retry policy, and diagnostics. The existing local publisher MUST remain an explicit rollback backend, and production MUST NOT fall back automatically after a shared-path failure.

#### Scenario: Shared immutable publication is selected

- GIVEN production immutable remote-attempt or source-state publication
- WHEN the adapter is selected
- THEN the shared backend MUST run and Mantle-owned policy MUST remain unchanged.

#### Scenario: Replaceable or directory publication is requested

- GIVEN a remote-attempt manifest or release-bundle directory
- WHEN publication routing runs
- THEN the request MUST remain on its existing Mantle-owned mechanism.

### Requirement: Corpus, integration, and typed evidence

r[mantle.durable_file_publication.validation]

Mantle MUST replay all 17 producer corpus rows and run positive and negative Linux integration tests for request mapping, commit state, existing-content interpretation, race resistance, no-follow behavior, bounds, cleanup, and source-state ingest.

#### Scenario: Shared corpus is replayed

- GIVEN the immutable producer corpus at the reviewed revision
- WHEN Mantle's consumer mapping test runs
- THEN all rows MUST preserve their expected mechanical disposition and cleanup facts.

#### Scenario: Source-state adapter is tested

- GIVEN admitted add, identical-reuse, conflict, invalid, interruption, collision, and no-follow cases
- WHEN the source-state integration tests run
- THEN each case MUST preserve the shared result and Mantle-owned ingest policy.

#### Scenario: A mapped outcome or source fact drifts

- GIVEN changed commit state, cleanup state, source identity, request field, or non-claim
- WHEN validation runs
- THEN the adoption check MUST fail.

### Requirement: Typed adoption evidence

r[mantle.durable_file_publication.evidence]

Mantle MUST emit typed Nickel and deterministic JSON with a BLAKE3 sidecar that binds the source, Cargo and Nix locks, mapping profile, remote-attempt and source-state test observations, rollback boundary, authority boundary, and platform non-claims.

#### Scenario: Complete evidence matches implementation

- GIVEN matching source, locks, mapping, tests, and non-claims
- WHEN evidence validation runs
- THEN the evidence MUST be accepted deterministically.

#### Scenario: Evidence omits a boundary

- GIVEN missing source identity, source-state surface, outcome mapping, rollback, authority, platform limit, or test observation
- WHEN evidence validation runs
- THEN the evidence MUST fail closed.
