# Source Transports Specification

## Purpose

Adds vendored source manifest contracts for Mantle and reusable stack vendor workflows.

## Requirements

### Requirement: Vendor source manifest contract
r[mantle.source_transports.vendor_source_manifests.contract] Mantle MUST support a vendor source manifest contract that records upstream repo, revision, filter, selected paths, measured identities, local edits, refresh command, and non-claims.

#### Scenario: Fresh vendor manifest passes
r[mantle.source_transports.vendor_source_manifests.fixtures.positive]
- GIVEN a manifest row names upstream identity, revision, selected paths, BLAKE3 identities, declared local edits, and required non-claims
- WHEN vendor validation runs
- THEN validation MUST pass and preserve the vendor row identity.

#### Scenario: Stale vendor manifest fails
r[mantle.source_transports.vendor_source_manifests.fixtures.negative]
- GIVEN a manifest row has stale digest, missing revision metadata, unsafe path, or undeclared local edit
- WHEN vendor validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Vendor manifest validation
r[mantle.source_transports.vendor_source_manifests.validation] Mantle MUST validate vendor manifests over parsed rows and measured file identities.

#### Scenario: Validation reports identity only
r[mantle.source_transports.vendor_source_manifests.docs]
- GIVEN a vendor manifest validates
- WHEN the supported claim is rendered
- THEN it MUST state that validation proves vendored-source identity/freshness only and not upstream correctness or license compatibility.

### Requirement: Final validation
r[mantle.source_transports.vendor_source_manifests.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers vendor health
r[mantle.source_transports.vendor_source_manifests.final_validation.fixtures]
- GIVEN valid and invalid vendor manifest fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
