## ADDED Requirements

### Requirement: Witnessed release workflow MUST export a portable request directory

Crunch MUST provide a checked-in way to export one verified release-evidence
bundle plus the signed release-attestation seed into a portable request
directory for a second environment.
ID: release.evidence.workflow.witnessed.request-export

The exported request directory MUST:
- contain a verified copy of the release-evidence bundle,
- contain the signed `release-attestation.json` and matching `.sig` sidecar,
- include deterministic metadata naming the release identifier and request
  layout version, and
- exclude private signing-key material.

#### Scenario: Publisher exports a witness request from verified artifacts

- GIVEN a release-evidence bundle that passes `crunch release verify`
- AND a verification directory containing a signed `release-attestation.json`
- WHEN the operator runs `crunch release witness-export`
- THEN the command writes a portable request directory with the verified bundle
- AND it copies `release-attestation.json` plus `release-attestation.json.sig`
- AND it does not copy any signing key file into the request directory

#### Scenario: Export rejects mismatched release identifiers

- GIVEN a release-evidence bundle whose release identifier differs from the
  supplied verification directory's signed release attestation
- WHEN the operator runs `crunch release witness-export`
- THEN the command exits non-zero
- AND it names the release-identifier mismatch

### Requirement: Cross-machine witnessed-self-hosting docs MUST show the request handoff

The documented witnessed-self-hosting workflow MUST show how a publisher exports
public verification material for a second environment and how a witness uses
that material without receiving the publisher's signing keys.
ID: release.evidence.workflow.witnessed.crossmachine.docs

#### Scenario: Docs show publisher-to-witness request flow

- GIVEN a contributor follows the witnessed-self-hosting workflow docs
- WHEN they reach the second-environment handoff step
- THEN the docs show `crunch release witness-export`
- AND the docs describe the request directory as public verification material
- AND the docs keep publisher and witness signing keys in their respective
  environments
