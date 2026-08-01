# Artifact Auth Adoption Specification

## Purpose

Defines the `artifact-auth-adoption` capability.

## Requirements

### Requirement: Mantle adopts one immutable reviewed source

r[mantle.artifact_auth_adoption.source] Mantle MUST consume one immutable reviewed `artifact-auth` revision with aligned Cargo and Nix identities and MUST bind the Mantle mapping profile and checked projection from that same revision before implementation or cutover.

#### Scenario: Source identity is admissible

- GIVEN Cargo, Nix, the mapping profile, and its checked projection resolve revision `799459346d5416fbd7b9f55840a7371441b55afa`
- WHEN Mantle evaluates dependency admission
- THEN it SHALL reject floating, duplicate, mismatched, sibling-path, product-dependent, or license-incompatible source selections.

### Requirement: Mantle retains product authority

r[mantle.artifact_auth_adoption.authority] Mantle MUST retain OCI canonicalization, repository authorization, registry routing, credentials, signing, trust/currentness collection, cache/build admission, receipt composition, and release policy while treating standalone authentication as one bounded input.

#### Scenario: Authentication passes without product admission

- GIVEN a standalone signature and policy decision passes
- WHEN repository, cache, build, or release admission runs
- THEN Mantle MUST still require its product-owned checks and MUST NOT promote standalone success into product authority.

### Requirement: Cutover requires explained dual-run evidence

r[mantle.artifact_auth_adoption.cutover] Mantle MUST dual-run legacy and standalone paths over identical observations, classify every preimage, identity, decision, issue, and non-claim difference, reject unrelated-failure false parity, and preserve a bounded legacy rollback until standalone authority is explicitly admitted.

#### Scenario: Unexplained drift blocks cutover

- GIVEN any unexplained compatibility or source-identity difference
- WHEN Mantle evaluates cutover admission
- THEN the legacy path SHALL remain authoritative and the exact blocker SHALL be recorded without weakening current product gates.

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

### Requirement: Live cutover validation is scoped to artifact-auth source facts

r[mantle.artifact_auth_adoption.live_validation_scope] Mantle MUST preserve the accepted historical cutover receipt while live validation checks the current artifact-auth source declaration, Cargo manifests and lock, Nix lock identity, package set, and forbidden fallback state. Live validation MUST NOT require the current whole `flake.nix` file to retain the historical cutover digest.

#### Scenario: Unrelated flake maintenance preserves admission

- GIVEN the historical cutover receipt remains byte-identical
- AND current artifact-auth source, package, lock, NAR, and fallback facts match the accepted values
- WHEN unrelated `flake.nix` content changes
- THEN live cutover validation MUST accept the artifact-auth source agreement.

#### Scenario: Current source drift fails closed

- GIVEN the current artifact-auth URL, revision, package set, Cargo lock source, Nix lock source, or NAR identity differs from the accepted value
- WHEN live cutover validation runs
- THEN validation MUST fail before Mantle uses the dependency.

#### Scenario: Historical receipt drift fails closed

- GIVEN the Nickel receipt, JSON export, or BLAKE3 sidecar differs from the accepted historical cutover evidence
- WHEN receipt validation runs
- THEN validation MUST fail without rewriting the historical receipt.
