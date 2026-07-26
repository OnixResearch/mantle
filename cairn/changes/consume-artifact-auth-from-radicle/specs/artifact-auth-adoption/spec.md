# Artifact-auth adoption

## ADDED Requirements

### Requirement: Radicle-backed source transport r[mantle.artifact_auth_adoption.radicle_transport]

Mantle MUST consume the accepted public artifact-auth RID through the upload-pack-only Radicle HTTPS adapter at exact revision `799459346d5416fbd7b9f55840a7371441b55afa`.

#### Scenario: Exact public source is admitted

- GIVEN an accepted publication receipt, RID, URL, commit, and source archive identity
- WHEN Mantle evaluates dependency source admission
- THEN all identities MUST agree before the source is used.

#### Scenario: Unaccepted source is rejected

- GIVEN a different RID, URL, commit, visibility, or archive identity
- WHEN source admission runs
- THEN the dependency MUST be rejected.

### Requirement: Cargo and Nix lock agreement r[mantle.artifact_auth_adoption.lock_agreement]

Cargo manifests, `Cargo.lock`, `flake.nix`, and `flake.lock` MUST identify the same Radicle HTTPS repository and reviewed Git revision while preserving the locked Nix content identity.

#### Scenario: Owning tools preserve identity

- GIVEN regenerated Cargo and Nix locks
- WHEN agreement validation runs
- THEN both artifact-auth packages, the exact revision, the public URL, and the expected Nix content hash MUST match.

#### Scenario: Lock drift fails closed

- GIVEN a stale GitHub source, mismatched revision, duplicate or missing package, or changed Nix content hash
- WHEN agreement validation runs
- THEN validation MUST fail.

### Requirement: Behavioral parity checks r[mantle.artifact_auth_adoption.behavior]

The transport cutover MUST preserve existing Mantle and artifact-auth behavior under focused action-result, shell verification, operational receipt, formatting, and source-admission checks.

#### Scenario: Existing positive and negative behavior passes

- GIVEN the Radicle-backed source at the reviewed commit
- WHEN the focused suites run
- THEN existing acceptance and rejection behavior MUST pass without Rust implementation changes.

#### Scenario: Behavioral drift blocks acceptance

- GIVEN any changed fixture result, feature graph, package version, or authority boundary
- WHEN parity is evaluated
- THEN the cutover MUST NOT be accepted.

### Requirement: No executable GitHub fallback r[mantle.artifact_auth_adoption.fallback]

Mantle MUST NOT retain an executable GitHub source fallback for artifact-auth.

#### Scenario: Radicle source is unavailable

- GIVEN the admitted Radicle HTTPS source cannot serve the exact object
- WHEN dependency resolution runs
- THEN resolution MUST fail visibly rather than selecting GitHub.

#### Scenario: Historical documentation remains bounded

- GIVEN documentation names historical GitHub provenance
- WHEN fallback validation runs
- THEN documentation MAY remain while manifests and locks contain no executable artifact-auth GitHub source.

### Requirement: Typed Radicle cutover evidence r[mantle.artifact_auth_adoption.radicle_evidence]

Mantle MUST emit typed Nickel/JSON evidence with a BLAKE3 sidecar binding publication, source, locks, test observations, rollback boundary, and non-claims.

#### Scenario: Complete cutover evidence passes

- GIVEN matching publication, source, lock, and test observations
- WHEN receipt validation runs
- THEN it MUST accept deterministically.

#### Scenario: Missing linkage or overclaim fails

- GIVEN missing publication linkage, drifted locks, missing negative evidence, or weakened non-claims
- WHEN receipt validation runs
- THEN it MUST fail closed.
