## MODIFIED Requirements

### Requirement: Mantle adopts one immutable reviewed source

r[mantle.artifact_auth_adoption.source] Mantle MUST consume `artifact-auth-core` and `artifact-auth-ed25519` from one immutable reviewed `OnixResearch/onix-artifact` revision. Cargo and Nix identities, source-workspace membership, licensing, and the two-package consumer graph MUST agree.

#### Scenario: Unified source identity is admissible

- GIVEN Cargo and Nix resolve revision `c932138d880ddf4c2967f4c024b489b5c0022bf1`
- WHEN Mantle evaluates dependency admission
- THEN it SHALL accept only the two authentication packages and reject floating, duplicate, mismatched, local, widened, or license-incompatible sources.

### Requirement: Unified Artifact workspace source transport r[mantle.artifact_auth_adoption.radicle_transport]

Mantle MUST consume the authentication packages from the immutable unified Artifact repository. It MUST retain the predecessor Radicle identity as historical evidence only.

#### Scenario: Exact unified source is admitted

- GIVEN the repository, revision, Nix NAR hash, workspace members, and authentication source identities agree
- WHEN Mantle evaluates the dependency source
- THEN it MAY use the source without admitting transfer, binding, or Artifact-owned authority.

#### Scenario: Mixed or widened source is rejected

- GIVEN a different repository, revision, NAR hash, package set, source identity, or executable predecessor dependency
- WHEN source admission runs
- THEN the dependency MUST be rejected.

### Requirement: Cargo and Nix lock agreement r[mantle.artifact_auth_adoption.lock_agreement]

Cargo manifests, `Cargo.lock`, `flake.nix`, and `flake.lock` MUST identify the same unified Artifact repository and reviewed revision. The lock MUST preserve the exact two-package Mantle consumer graph.

#### Scenario: Owning tools preserve identity

- GIVEN Cargo and Nix regenerated their locks
- WHEN agreement validation runs
- THEN both authentication packages, the revision, the repository, and the Nix content identity MUST match.

#### Scenario: Lock drift fails closed

- GIVEN a predecessor, floating, local, mismatched, duplicate, missing, or widened source selection
- WHEN agreement validation runs
- THEN validation MUST fail.

### Requirement: Behavioral parity checks r[mantle.artifact_auth_adoption.behavior]

The source migration MUST preserve existing Mantle authentication behavior under focused positive and negative checks without Rust implementation changes.

#### Scenario: Existing behavior passes

- GIVEN the unified source at the reviewed revision
- WHEN focused action-result and shell checks run
- THEN existing acceptance and rejection behavior MUST pass.

#### Scenario: Behavioral drift blocks acceptance

- GIVEN a changed fixture result, feature graph, package version, source entry point, or authority boundary
- WHEN parity is evaluated
- THEN the migration MUST not be accepted.

### Requirement: No mixed or automatic source fallback r[mantle.artifact_auth_adoption.fallback]

Mantle MUST use only the admitted unified source in executable manifests and locks. It MUST NOT automatically select the predecessor repository, a sibling path, or a floating source.

#### Scenario: Unified source is unavailable

- GIVEN the admitted source cannot serve the exact object
- WHEN dependency resolution runs
- THEN resolution MUST fail visibly.

#### Scenario: Historical evidence remains bounded

- GIVEN documentation and archived receipts name predecessor sources
- WHEN active-source validation runs
- THEN historical evidence MAY remain while executable manifests and locks use only the unified source.

### Requirement: Typed source migration evidence r[mantle.artifact_auth_adoption.radicle_evidence]

Mantle MUST emit typed Nickel and JSON evidence with a BLAKE3 sidecar. The evidence MUST bind the unified source, source and consumer package sets, source-byte identity, checks, rollback, and non-claims.

#### Scenario: Complete migration evidence passes

- GIVEN matching source, package, lock, parity, and check observations
- WHEN receipt validation runs
- THEN it MUST accept deterministically.

#### Scenario: Missing linkage or overclaim fails

- GIVEN source drift, package widening, missing negative evidence, automatic rollback, absolute evidence paths, or weakened non-claims
- WHEN receipt validation runs
- THEN it MUST fail closed.

### Requirement: Live cutover validation is scoped to active source facts

r[mantle.artifact_auth_adoption.live_validation_scope] Mantle MUST preserve the historical cutover receipt while live validation checks the current unified source declaration, manifests, locks, package set, source bytes, and fallback state.

#### Scenario: Unrelated flake maintenance preserves admission

- GIVEN the historical receipt remains byte-identical
- AND current Artifact source facts match the accepted values
- WHEN unrelated `flake.nix` content changes
- THEN live source validation MUST accept the source agreement.

#### Scenario: Current source drift fails closed

- GIVEN the active repository, revision, package set, lock source, NAR identity, or source bytes differ
- WHEN live source validation runs
- THEN validation MUST fail before Mantle uses the dependency.
