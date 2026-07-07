## ADDED Requirements

### Requirement: External evidence adapter slot
r[mantle.release_external_evidence] Mantle release evidence MUST support an optional opaque external-evidence sidecar collection for adapter-produced artifacts without requiring Mantle core to depend on adapter-specific schemas or crates.

#### Scenario: Release evidence without external sidecars remains valid
r[mantle.release_external_evidence.optional]
- GIVEN a release evidence manifest has no external evidence entries
- WHEN Mantle validates the manifest under default release verification
- THEN validation MUST continue to accept the manifest when all ordinary release evidence checks pass.

#### Scenario: Adapter sidecar is bundled as opaque evidence
r[mantle.release_external_evidence.bundle]
- GIVEN an operator supplies an external evidence file with role, schema, claim scope, and non-claims
- WHEN Mantle creates a release evidence bundle
- THEN the bundle MUST record the sidecar role, schema, bundle-local path, BLAKE3 digest, claim scope, and non-claims.
- AND Mantle MUST NOT interpret the sidecar schema as a Mantle-native claim.

### Requirement: External evidence validation
r[mantle.release_external_evidence.validation] Mantle release verification MUST fail closed for malformed opaque external evidence metadata or sidecar identity mismatches while keeping schema-specific semantic validation outside Mantle core.

#### Scenario: Sidecar digest mismatch fails
r[mantle.release_external_evidence.validation.digest]
- GIVEN an external evidence entry records a BLAKE3 digest that does not match the bundled sidecar bytes
- WHEN Mantle verifies the release evidence bundle
- THEN verification MUST fail with a diagnostic naming the external evidence digest mismatch.

#### Scenario: Missing non-claims fail
r[mantle.release_external_evidence.validation.non_claims]
- GIVEN an external evidence entry has an empty non-claims list
- WHEN Mantle validates the release evidence manifest
- THEN validation MUST fail with a diagnostic naming the missing external evidence non-claims.

#### Scenario: Path escape fails
r[mantle.release_external_evidence.validation.path_escape]
- GIVEN an external evidence entry uses a relative path that escapes the release evidence bundle
- WHEN Mantle validates the release evidence manifest or verifies the bundle
- THEN validation MUST fail with a diagnostic naming the invalid external evidence path.

### Requirement: External evidence CLI integration
r[mantle.release_external_evidence.cli] Mantle SHOULD provide release creation and verification CLI options for explicitly bundling external evidence sidecars and requiring named external-evidence roles.

#### Scenario: Release create copies sidecar
r[mantle.release_external_evidence.cli.create]
- GIVEN an operator supplies an external evidence file with role, schema, and claim scope metadata
- WHEN Mantle release creation runs with the external evidence options
- THEN Mantle SHOULD copy the sidecar into the release evidence bundle and record its BLAKE3 digest.

#### Scenario: Release verify checks required role
r[mantle.release_external_evidence.cli.verify_role]
- GIVEN an operator requires an external evidence role during release verification
- WHEN Mantle verifies the release evidence bundle
- THEN Mantle SHOULD report whether that role is present after ordinary sidecar path and digest checks.

### Requirement: Opt-in external evidence role gate
r[mantle.release_external_evidence.role_gate] Mantle release verification SHOULD provide an opt-in gate that requires a named external evidence role to be present without validating adapter-specific sidecar semantics.

#### Scenario: Required role present passes generic role gate
r[mantle.release_external_evidence.role_gate.present]
- GIVEN a release evidence bundle contains an external evidence entry with role `stack-provenance-trace`
- WHEN Mantle release verification runs with a requirement for that role
- THEN the generic role gate SHOULD pass after ordinary path and digest checks pass.

#### Scenario: Required role missing fails generic role gate
r[mantle.release_external_evidence.role_gate.missing]
- GIVEN a release evidence bundle does not contain an external evidence entry with the required role
- WHEN Mantle release verification runs with a requirement for that role
- THEN verification SHOULD fail with a diagnostic naming the missing external evidence role.

### Requirement: Adapter semantics remain external
r[mantle.release_external_evidence.boundary] Mantle MUST treat Valence, Octet, Trellis, Cairn, and other adapter-specific provenance schemas as opaque external evidence unless a separate adapter verifier validates them outside Mantle core.

#### Scenario: Valence provenance sidecar is not promoted by Mantle
r[mantle.release_external_evidence.boundary.valence]
- GIVEN a release evidence bundle includes an external sidecar with schema `valence.provenance-chain.v1`
- WHEN Mantle verifies the release bundle
- THEN Mantle MUST verify only the sidecar identity and bounded metadata.
- AND Mantle MUST NOT claim the Valence provenance chain is semantically valid, complete, or sufficient for stack traceability.
