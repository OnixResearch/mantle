## MODIFIED Requirements

### Requirement: Witnessed self-hosting workflow starts from verified release evidence

A full self-hosting proof bundle MUST be promotable into a witnessed release
verification workflow without hand-writing attestation or policy JSON, and
without requiring the witness environment to invent its own rebuild recipe.
ID: release.evidence.workflow.witnessed.selfhosting

The checked-in CLI workflow MUST let an operator:
- verify a release-evidence bundle,
- create a signed release attestation in a verification directory,
- scaffold verifier-local policy material for either self-proof-only or
  single-witness publication,
- export a public request directory for the witness environment,
- run the checked-in witness rebuild entry point
  `./scripts/rebuild-witness-request.sh` in the second environment so it
  verifies the request directory, rebuilds the published outputs, creates
  witness sidecars, and records rebuild evidence, and
- re-run verification after witness import to observe the technical class,
  policy status, and final class.

#### Scenario: Checked-in witness rebuild runner produces publishable witness material

- GIVEN an exported witness request directory from a release-evidence bundle
  that passes `crunch release verify`
- WHEN the witness operator runs `./scripts/rebuild-witness-request.sh` with
  signing key and witness metadata
- THEN the runner verifies the bundled release-evidence request before rebuild
- AND it emits witness sidecars whose release-attestation digest matches the
  request's release attestation
- AND it records rebuild evidence naming the replayed workflow identity and the
  rebuilt output digests

#### Scenario: Single matching witness promotes a self-hosting release

- GIVEN a release-evidence bundle with a full proof artifact that passes
  `crunch release verify`
- AND an operator scaffolds single-witness policy material for one trusted
  release signer and one trusted witness identity
- AND a witness rebuilder runs `./scripts/rebuild-witness-request.sh` and
  produces matching witness sidecars
- WHEN `crunch attest release-verify` runs with the trusted public keys after
  witness import
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

### Requirement: Cross-machine witnessed-self-hosting docs MUST show the request handoff

The documented witnessed-self-hosting workflow MUST show how a publisher exports
public verification material for a second environment, how the witness runs the
checked-in rebuild entry point from that request directory, and how the
publisher imports the returned witness sidecars without sharing signing keys.
ID: release.evidence.workflow.witnessed.crossmachine.docs

#### Scenario: Docs show publisher-to-witness rebuild flow

- GIVEN a contributor follows the witnessed-self-hosting workflow docs
- WHEN they reach the second-environment handoff step
- THEN the docs show `crunch release witness-export`
- AND the docs show `./scripts/rebuild-witness-request.sh` in the witness
  environment
- AND the docs describe the request directory as public verification material
- AND the docs keep publisher and witness signing keys in their respective
  environments
