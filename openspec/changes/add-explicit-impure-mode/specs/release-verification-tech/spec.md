## ADDED Requirements

### Requirement: Release verification rejects impure material for existing proof classes

Release verification MUST treat impure build material as proof-blocking for the
existing reproducibility and deterministic proof classes. If an artifact,
witness rebuild, or deterministic proof receipt records hermeticity mode
`impure`, the verifier MUST fail closed for proof classes that require pure,
practical, or strict evidence.

#### Scenario: Impure release artifact is not reproducible evidence

- GIVEN a release evidence bundle records an artifact built in impure mode
- WHEN release verification evaluates `self-rebuild-match`,
  `external-witness-match`, `policy-satisfied`, or deterministic-release claim
  eligibility
- THEN the verifier does not promote the artifact into those classes
- AND it reports impure execution as the blocker

#### Scenario: Impure witness is rejected for agreement

- GIVEN a witness rebuild attestation records hermeticity mode `impure`
- WHEN release verification evaluates independent witness agreement
- THEN the witness cannot satisfy `external-witness-match`
- AND the verifier reports the witness as skipped or failed due to impure mode
