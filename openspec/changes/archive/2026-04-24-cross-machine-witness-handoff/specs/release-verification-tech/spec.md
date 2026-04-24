## ADDED Requirements

### Requirement: CLI MUST import returned witness sidecars safely

Crunch MUST provide `crunch attest witness-import <verification-dir> <source>`
to validate and copy returned witness-attestation sidecars into a publisher
verification directory.
ID: release.verification.tech.witness.import.cli

The command MUST:
- accept either a directory containing `witnesses/*.json` plus matching `.sig`
  sidecars or a single witness-attestation `.json` file,
- reject missing signature sidecars,
- reject witness attestations whose referenced release-attestation digest does
  not match the destination verification directory,
- reject conflicting existing witness identities when the bytes differ, and
- write imported sidecars under `<verification-dir>/witnesses/`.

#### Scenario: Import accepts matching witness sidecars

- GIVEN a publisher verification directory with a signed release attestation
- AND a returned witness-attestation file plus matching `.sig` sidecar whose
  release-attestation digest matches the destination verification directory
- WHEN the operator runs `crunch attest witness-import`
- THEN the command exits successfully
- AND it writes the witness files under `<verification-dir>/witnesses/`

#### Scenario: Import rejects missing witness signature sidecar

- GIVEN a returned witness-attestation `.json` file without the matching `.sig`
  sidecar
- WHEN the operator runs `crunch attest witness-import`
- THEN the command exits non-zero
- AND it names the missing witness signature sidecar

#### Scenario: Import rejects wrong release-attestation digest

- GIVEN a returned witness attestation referencing a different
  release-attestation digest than the destination verification directory
- WHEN the operator runs `crunch attest witness-import`
- THEN the command exits non-zero
- AND it names the release-attestation digest mismatch

#### Scenario: Import accepts exact duplicate witness material idempotently

- GIVEN the destination verification directory already contains
  `witnesses/alice.json` and `witnesses/alice.json.sig`
- AND a second import for `alice` has byte-identical attestation and signature
  contents
- WHEN the operator runs `crunch attest witness-import`
- THEN the command exits successfully
- AND it leaves the existing files unchanged

#### Scenario: Import rejects conflicting duplicate witness identity

- GIVEN the destination verification directory already contains
  `witnesses/alice.json`
- AND a second import for `alice` has different attestation bytes or different
  signature bytes
- WHEN the operator runs `crunch attest witness-import`
- THEN the command exits non-zero
- AND it names the conflicting witness identity instead of overwriting it
