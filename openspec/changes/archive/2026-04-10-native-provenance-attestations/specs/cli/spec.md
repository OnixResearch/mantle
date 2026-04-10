## ADDED Requirements

### Requirement: Attestation command surface

The CLI MUST expose attestation commands as a first-class operator surface.

At minimum the CLI MUST provide commands to:
- show an artifact attestation
- assemble a closure attestation for a root
- verify artifact, closure, or project attestation digests against canonical reconstruction
- diff two native attestations
- render a project attestation view from the manifest, lockfile, and built roots

#### Scenario: Operator inspects artifact attestation

- GIVEN a built store path
- WHEN `crunch attest show <path>` runs
- THEN the CLI renders the native artifact attestation for that logical store path

#### Scenario: Operator verifies closure attestation

- GIVEN a rooted closure attestation already persisted by crunch
- WHEN `crunch attest verify <path>` runs for that root
- THEN crunch recomputes the canonical closure digest
- AND it reports whether the persisted digest matches the recomputed digest

### Requirement: Build reporting includes attestation references

`crunch build` and `crunch --json build` MUST report where the generated
attestations can be found or retrieved.

#### Scenario: JSON build report includes attestation references

- GIVEN a successful `crunch --json build`
- WHEN the JSON report is emitted
- THEN each successful outcome includes a reference to its persisted artifact attestation
- AND the report schema distinguishes those references from ordinary build outputs
