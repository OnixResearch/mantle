# Build Correctness Store Capability Delta

## ADDED Requirements

### Requirement: Store roles use distinct capability values

r[build_correctness.store_capabilities.roles] Mantle MUST give build realization, output lookup, root retention, source admission, action-result exchange, and store administration distinct Rust capability values with only their required operations.

#### Scenario: Builder receives bounded store authority

- **GIVEN** Mantle constructs a builder for local or remote realization
- **WHEN** the builder receives its store dependencies
- **THEN** it MUST receive only build-store and fixed action-result operations
- **AND** garbage collection, source import, backend replacement, repair, and arbitrary root mutation MUST NOT be available through those values

#### Scenario: Pipeline retains root authority separately

- **GIVEN** pipeline orchestration needs output facts and selected root registration after a build
- **WHEN** it constructs the builder and post-build shell
- **THEN** output lookup and root registration MUST remain separate from builder-owned authority
- **AND** the builder MUST NOT gain arbitrary pin, unpin, or garbage-collection access

### Requirement: Raw writable store services remain confined

r[build_correctness.store_capabilities.raw_service_confinement] Mantle MUST keep writable blob, directory, PathInfo, publisher, and action-result backend objects private to `crunch-store` or shell-owned compatibility code.

#### Scenario: Build code performs a supported store operation

- **GIVEN** build code needs closure resolution, NAR calculation, castore transformation, cache lookup, substitution, or output persistence
- **WHEN** it requests that operation
- **THEN** it MUST use a named high-level capability method
- **AND** it MUST NOT receive a raw service object that permits unrelated mutation

#### Scenario: Raw service escape is introduced

- **GIVEN** a build or pipeline API returns a writable store trait object, broad callback, or generic service escape hatch
- **WHEN** compile-fail examples or source-policy guards run
- **THEN** validation MUST fail with a deterministic authority-boundary finding
- **AND** a test-only constructor MUST NOT disable the production boundary broadly

### Requirement: Capability migration preserves store behavior

r[build_correctness.store_capabilities.compatibility] Mantle MUST preserve accepted store formats, PathInfo and attestation facts, report schemas, output identities, cache behavior, substitution behavior, and root outcomes during the capability migration.

#### Scenario: Supported realization paths remain equivalent

- **GIVEN** accepted fixtures for local builds, cache hits, remote substitution, content-addressed outputs, action-result reuse, publication, and selected root retention
- **WHEN** the fixtures run before and after the capability migration
- **THEN** their accepted output identities and machine-visible facts MUST remain equal
- **AND** any intentional compatibility difference MUST require a separate versioned change

#### Scenario: Rejected output stays uncommitted

- **GIVEN** output hash, signature, identity, policy, or admission checks reject a candidate
- **WHEN** the restricted store capabilities apply the decision
- **THEN** no PathInfo, exported output, attestation, root, action result, or success report MUST be committed for that candidate
- **AND** restricted authority MUST NOT weaken the existing fail-closed path

### Requirement: Store capability claims remain local

r[build_correctness.store_capabilities.claim_boundary] Mantle MUST limit store-capability claims to Rust API reachability and preserved tested behavior.

#### Scenario: Capability tests pass

- **GIVEN** positive behavior tests and negative API guards pass
- **WHEN** Mantle reports the result
- **THEN** it MAY claim that selected Rust callers lack the excluded methods
- **AND** it MUST NOT claim host filesystem confinement, sandbox correctness, output correctness, cache trust, or release eligibility from the type split alone
