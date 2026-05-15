## MODIFIED Requirements

### Requirement: Deterministic release promotion requires sandbox isolation evidence

Deterministic release promotion MUST require a canonical deterministic sandbox isolation evidence artifact generated from the actual proof sandbox profile family used by repeated deterministic proof runs. The evidence artifact MUST use schema `mantle-deterministic-sandbox-isolation-evidence-v1`, profile family `mantle-proof-sandbox-v1`, status `passed`, include the required isolation checks `denies-undeclared-host-access`, `denies-host-network-by-default`, and `denies-main-output-and-proof-store-reuse`, and carry a BLAKE3 digest over canonical profile/evidence material that is not self-referential.

#### Scenario: Deterministic reproduce emits isolation evidence

- GIVEN a release evidence bundle and a reproducible rebuild command
- WHEN `mantle release reproduce` runs with at least two deterministic proof runs under the supported proof sandbox profile
- THEN it writes `deterministic-sandbox-isolation-evidence.json` beside `deterministic-build-proof.json`
- AND the CLI reports the evidence path and BLAKE3 digest
- AND the evidence validates under the release verifier contract.

#### Scenario: Evidence binds the proof sandbox family

- GIVEN deterministic proof receipts with sandbox profile identities in the `mantle-proof-sandbox-v1` family
- WHEN the release workflow generates sandbox isolation evidence
- THEN the evidence profile family is `mantle-proof-sandbox-v1`
- AND its digest material includes the concrete generated profile identities.
