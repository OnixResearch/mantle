## MODIFIED Requirements

### Requirement: Release evidence claims stay narrower than bootstrap claims

Docs and manifests for release evidence MUST describe the bundle as packaged
integrity and proof-context evidence, not as automatic proof of full-source
bootstrap or global reproducibility.

README release-evidence wording and bootstrap-facing docs MUST use the same
bounded claim language for what `crunch release verify` proves.

Bootstrap-facing docs MUST also keep their trust-boundary inventory aligned
with the current proof modes and the current reduced seed/provider description
used by the repo.

#### Scenario: Release docs do not over-claim

- GIVEN a reader following the release evidence documentation
- WHEN they read what bundle verification proves
- THEN the docs distinguish release evidence from full-source bootstrap claims
- AND they do not claim bundle verification alone proves global reproducibility

#### Scenario: README and bootstrap-facing docs agree on release verification

- GIVEN a reader compares the README release section with
  `docs/bootstrap-stage0-inventory.md`
- WHEN they read what `crunch release verify` proves today
- THEN both docs describe bundle-local integrity and proof-context checks
- AND neither doc claims independent rebuild agreement or a stronger bootstrap
  proof than the current evidence supports

#### Scenario: Docs do not treat prerequisite-only checks as release proof

- GIVEN a reader follows the documented release-evidence workflow
- WHEN they read which proof artifact is required
- THEN the docs require a full proof bundle rather than a prerequisite-only
  `--check` result
- AND they keep that requirement aligned with the creation and verification
  behavior

#### Scenario: Bootstrap inventory stays aligned with the current proof boundary

- GIVEN a reader inspects `docs/bootstrap-stage0-inventory.md`
- WHEN they compare its trust-boundary claims with the current proof modes and
  current reduced seed/provider description
- THEN the doc names the current proof modes accurately
- AND it does not claim a wider stage0 trust boundary than the repo currently
  uses