# Remote Builds Service Gateway Delta

## ADDED Requirements

### Requirement: remote_builds.nix_compatibility_gateway

r[remote_builds.nix_compatibility_gateway]

Mantle SHALL provide a bounded compatibility gateway for an explicit set of Nix daemon-store operations over `ssh-ng` or another declared transport. The gateway SHALL translate admitted concrete operations into existing Mantle services.

#### Scenario: Supported concrete build is submitted

- **GIVEN** a negotiated supported protocol, valid authority, and admitted derivation and store-input identities
- **WHEN** a Nix client submits a supported build operation
- **THEN** the gateway SHALL create or recover a durable Mantle attempt
- **AND** execution SHALL retain existing fencing, locality, lease, transfer, store, and result checks

#### Scenario: Unsupported operation is requested

- **GIVEN** an evaluator, registry, arbitrary-command, administration, or unknown daemon operation
- **WHEN** the gateway parses the request
- **THEN** it SHALL reject the operation with a stable protocol outcome
- **AND** it SHALL NOT create an attempt or mutate store state

### Requirement: remote_builds.gateway_functional_core

r[remote_builds.gateway_functional_core]

Gateway admission, translation, capability checks, idempotency, and reconnect decisions SHALL be pure functions over parsed inputs, authority facts, policy, and explicit sequence or time values.

#### Scenario: Translation is tested in isolation

- **GIVEN** a parsed supported operation and authorized policy facts
- **WHEN** the translation core is tested
- **THEN** it SHALL return a typed Mantle command without transport, filesystem, network, clock, store, or scheduler access

#### Scenario: Request lacks required authority

- **GIVEN** a valid parsed operation without its required capability
- **WHEN** the core evaluates admission
- **THEN** it SHALL return a typed authorization rejection
- **AND** no imperative service SHALL be called

### Requirement: remote_builds.granular_service_authority

r[remote_builds.granular_service_authority]

Mantle SHALL enforce separate authority for build submission, admitted input upload, store read, status and log read, owned-attempt cancellation, cache publication, and service administration.

#### Scenario: Read-only caller requests status

- **GIVEN** a caller with status-read authority for an attempt
- **WHEN** the caller requests bounded status
- **THEN** Mantle SHALL return authorized status
- **AND** it SHALL NOT grant build, upload, publication, cancellation, or administration authority

#### Scenario: Compatibility ticket requests administration

- **GIVEN** a valid compatibility ticket
- **WHEN** it requests a service-administration operation
- **THEN** Mantle SHALL reject the request regardless of other ticket scopes

### Requirement: remote_builds.versioned_build_api

r[remote_builds.versioned_build_api]

Mantle SHALL expose a versioned, bounded API for concrete submission, authorized status, bounded log ranges, bounded event pages, cancellation, signed-result discovery, and bounded usage summaries.

#### Scenario: Client follows an attempt asynchronously

- **GIVEN** an admitted attempt and valid read authority
- **WHEN** a client requests status and event pages with valid cursors
- **THEN** Mantle SHALL return bounded ordered observations
- **AND** the client SHALL NOT need to hold the submission connection open

#### Scenario: Cursor or response request is invalid

- **GIVEN** a forged, stale, wrong-scope, or oversized cursor request
- **WHEN** the API validates it
- **THEN** it SHALL fail before reading unbounded data
- **AND** it SHALL return a stable redacted error

### Requirement: remote_builds.idempotent_completion_events

r[remote_builds.idempotent_completion_events]

Mantle SHALL emit signed completion events with stable identities and monotonic per-attempt sequences. Delivery MAY repeat, and consumers SHALL have enough public identity to deduplicate events.

#### Scenario: Attempt reaches a terminal state

- **GIVEN** a durable attempt that transitions to a terminal class
- **WHEN** Mantle commits the transition
- **THEN** it SHALL append a signed completion event
- **AND** the event SHALL reference result evidence when present
- **AND** the event SHALL contain no log body or credential

#### Scenario: Event is delivered twice

- **GIVEN** two deliveries of the same signed event
- **WHEN** a consumer compares event identity and sequence
- **THEN** both deliveries SHALL identify the same transition
- **AND** duplicate processing SHALL be avoidable

### Requirement: remote_builds.gateway_store_integrity

r[remote_builds.gateway_store_integrity]

Nix transport input SHALL NOT be authoritative for store bytes, PathInfo, cache publication, or action-result usability. Mantle SHALL run its existing verification and policy checks before those facts become usable.

#### Scenario: Imported object verifies

- **GIVEN** authorized imported bytes whose identity and PathInfo checks succeed
- **WHEN** Mantle commits the object
- **THEN** the object MAY become available under current store policy

#### Scenario: Imported metadata conflicts

- **GIVEN** bytes, PathInfo, signature, platform, policy, or CAS availability that does not match the request
- **WHEN** Mantle validates import or result reuse
- **THEN** it SHALL reject usability
- **AND** transport success SHALL NOT override the rejection

### Requirement: remote_builds.gateway_bounds_and_recovery

r[remote_builds.gateway_bounds_and_recovery]

Mantle SHALL enforce named bounds for connections, protocol frames, messages, concurrent operations, idle periods, log windows, event pages, and partial transfers while preserving durable-attempt recovery.

#### Scenario: Client disconnects after admission

- **GIVEN** a committed durable attempt
- **WHEN** the client connection ends
- **THEN** attempt policy SHALL decide continued execution independently of the transport
- **AND** an authorized client SHALL be able to reconnect by public attempt identity

#### Scenario: Input exceeds a declared bound

- **GIVEN** an oversized frame, message, log range, event page, or partial transfer
- **WHEN** the shell checks the bound
- **THEN** it SHALL reject or truncate only as specified before unbounded allocation
- **AND** it SHALL preserve state consistency

### Requirement: remote_builds.gateway_non_claims

r[remote_builds.gateway_non_claims]

Gateway evidence SHALL state that tested interoperability does not prove arbitrary Nix compatibility, evaluation correctness, hermeticity, sandboxing, output correctness, or release eligibility.

#### Scenario: Interoperability fixture passes

- **GIVEN** a supported Nix client and operation set
- **WHEN** the fixture completes
- **THEN** evidence SHALL identify the tested versions and operations
- **AND** it SHALL retain the gateway non-claim boundary
