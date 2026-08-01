## ADDED Requirements

### Requirement: Live cutover validation is scoped to artifact-auth source facts r[mantle.artifact_auth_adoption.live_validation_scope]

Mantle MUST preserve the accepted historical cutover receipt while live validation checks the current artifact-auth source declaration, Cargo manifests and lock, Nix lock identity, package set, and forbidden fallback state. Live validation MUST NOT require the current whole `flake.nix` file to retain the historical cutover digest.

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
