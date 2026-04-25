## ADDED Requirements

### Requirement: Technical verification surfaces reproducibility separately

Crunch MUST report reproducibility status separately from signature trust, witness agreement, and basic bundle integrity.
ID: release.verification.tech.reproducibility.status

The JSON output MUST make it possible for callers to require reproducibility evidence without confusing it with independent rebuild agreement or social quorum. Reproducibility status MUST include at least `absent`, `matched`, and `mismatched` states.

#### Scenario: Caller requires reproducibility

- GIVEN a release with valid bundle integrity and self-proof
- BUT no reproducibility report
- WHEN verification runs with a require-reproducible option
- THEN verification exits non-zero
- AND JSON output reports reproducibility status `absent`

#### Scenario: Reproducibility mismatch stays distinct

- GIVEN a release with valid signatures and matching witness policy
- BUT a reproducibility report records byte drift
- WHEN verification runs
- THEN the output reports the signature and witness statuses separately
- AND reproducibility status is `mismatched`
