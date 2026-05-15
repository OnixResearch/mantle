## MODIFIED Requirements

### Requirement: Release verification consumes deterministic proof receipts without overclaiming

Release verification MUST promote a release artifact beyond `self-rebuild-match` to a stronger deterministic-release claim only when it consumes canonical deterministic build proof receipts and canonical deterministic sandbox isolation evidence whose verdicts, profile family, and output digests prove every required artifact output in the release digest set. `mantle release verify` MUST expose explicit inputs for the generated proof and isolation-evidence artifacts and MUST expose a fail-closed option that rejects verification when deterministic-release eligibility is required but unavailable. The deterministic-release claim remains scoped to the named release artifacts, workflow identity, derivation identities, toolchain/provider identities, sandbox profile identity, and recorded proof matrix.

#### Scenario: Verify reports deterministic release when artifacts match

- GIVEN a release evidence bundle with matching artifact digests
- AND a canonical deterministic-build proof artifact generated for those artifacts
- AND a canonical deterministic sandbox isolation evidence artifact for the supported profile family
- WHEN `mantle release verify` receives both artifact paths
- THEN it reports deterministic-release eligibility
- AND it reports the proof and isolation-evidence BLAKE3 digests

#### Scenario: Required deterministic release rejects missing evidence

- GIVEN a release evidence bundle and deterministic proof artifact
- BUT no deterministic sandbox isolation evidence artifact is supplied
- WHEN `mantle release verify --require-deterministic-release` runs
- THEN the command exits non-zero
- AND the diagnostic identifies missing deterministic sandbox isolation evidence

#### Scenario: Malformed isolation evidence blocks deterministic promotion

- GIVEN deterministic proof output digests match the release artifacts
- BUT the deterministic sandbox isolation evidence artifact is malformed, non-canonical, failing, bypassed, stale, or bound to a different profile family
- WHEN release verification evaluates deterministic claim eligibility
- THEN deterministic-release promotion is not reported
- AND requiring deterministic release exits non-zero with the blocker
