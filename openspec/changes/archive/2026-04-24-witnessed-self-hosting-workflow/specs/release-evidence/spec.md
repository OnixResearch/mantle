## ADDED Requirements

### Requirement: Witnessed self-hosting workflow starts from verified release evidence

A full self-hosting proof bundle MUST be promotable into a witnessed release
verification workflow without hand-writing attestation or policy JSON.
ID: release.evidence.workflow.witnessed.selfhosting

The checked-in CLI workflow MUST let an operator:
- verify a release-evidence bundle,
- create a signed release attestation in a verification directory,
- scaffold verifier-local policy material for either self-proof-only or
  single-witness publication,
- add matching witness attestations from rebuilt binaries, and
- re-run verification to observe the technical class, policy status, and final
  class.

#### Scenario: Single matching witness promotes a self-hosting release

- GIVEN a release-evidence bundle with a full proof artifact that passes
  `crunch release verify`
- AND an operator scaffolds single-witness policy material for one trusted
  release signer and one trusted witness identity
- AND a witness rebuilder publishes rebuilt binaries matching the released
  digests
- WHEN `crunch attest release-verify` runs with the trusted public keys
- THEN the technical class is `external-witness-match`
- AND the policy status is `satisfied`
- AND the final class is `quorum-satisfied`

#### Scenario: Workflow docs keep verified self-hosting claims bounded

- GIVEN a contributor follows the documented witnessed self-hosting workflow
- WHEN they read what a successful witness result proves
- THEN the docs describe external witness agreement plus configured policy
  satisfaction
- AND they do not claim full-source bootstrap or globally reproducible release
  artifacts
