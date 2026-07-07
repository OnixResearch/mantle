## ADDED Requirements

### Requirement: Release evidence provenance coverage field
r[mantle.release_evidence_provenance] The `ReleaseEvidenceManifest` MUST support an optional `provenance_coverage` field recording adapter-supplied source IDs, function-object IDs, and requirement IDs as opaque release metadata.

#### Scenario: Manifest without coverage is valid
r[mantle.release_evidence_provenance.optional]
- GIVEN a ReleaseEvidenceManifest without provenance_coverage
- WHEN validated
- THEN it is accepted (the field is optional and backward-compatible)

#### Scenario: Manifest with valid coverage is accepted
r[mantle.release_evidence_provenance.valid]
- GIVEN a ReleaseEvidenceManifest with provenance_coverage carrying a valid binary_hash, at least one opaque covered ID, and the correct boundary text
- WHEN validated
- THEN it is accepted without Mantle interpreting adapter-specific ID semantics

#### Scenario: Empty coverage is rejected
r[mantle.release_evidence_provenance.empty_rejected]
- GIVEN a ReleaseEvidenceManifest with provenance_coverage where all covered ID lists are empty
- WHEN validated
- THEN it is rejected with "at least one covered"

#### Scenario: Weakened boundary is rejected
r[mantle.release_evidence_provenance.boundary_rejected]
- GIVEN a ReleaseEvidenceManifest with provenance_coverage where the coverage_boundary does not match the required text
- WHEN validated
- THEN it is rejected with "coverage_boundary"

#### Scenario: Invalid binary hash is rejected
r[mantle.release_evidence_provenance.hash_rejected]
- GIVEN a ReleaseEvidenceManifest with provenance_coverage where binary_hash is not a valid BLAKE3 hex hash
- WHEN validated
- THEN it is rejected with "provenance_coverage.binary_hash"
