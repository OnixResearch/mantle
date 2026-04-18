## ADDED Requirements

### Requirement: Social trust policy MUST stay external to technical artifact digests

Crunch MUST define trusted-role, quorum, independence, and revocation policy
outside the canonical digest material for release attestations and witness
attestations.

#### Scenario: Policy update does not change technical artifact digests

- GIVEN an unchanged release attestation and unchanged witness attestations
- WHEN a local trust policy is tightened or relaxed
- THEN the attestation digests stay unchanged
- AND verification may change policy status without changing technical status

### Requirement: Verifier MUST support witness-role and quorum policy

Crunch MUST support policy rules for which witnesses count toward decentralized
release verification and how many matching witnesses are required.

#### Scenario: Insufficient quorum fails policy even after technical agreement

- GIVEN a technically valid release with fewer matching witnesses than the
  configured quorum requires
- WHEN verification runs
- THEN the command reports policy failure
- AND it identifies the missing quorum class

#### Scenario: Satisfied quorum promotes final release class

- GIVEN a technically valid release whose witness set satisfies the configured
  quorum policy
- WHEN verification runs
- THEN the command reports policy success
- AND the final release class is `quorum-satisfied`

### Requirement: Verifier MUST support witness-independence policy

Crunch MUST support policy rules that reject witness sets lacking required
independence across actors, keys, organizations, or equivalent configured
witness domains.

#### Scenario: Same actor cannot satisfy all required witness slots

- GIVEN multiple matching witness attestations from the same actor or same
  configured witness domain
- WHEN policy requires independent witnesses
- THEN verification reports policy failure
- AND it explains that the witness set is not independent enough

### Requirement: Social policy MUST define revocation and dispute handling

Crunch MUST define how revocations and disputes affect the policy status of
previously published witness material. The first phase MUST use file-based
revocation and dispute input, not an external service dependency. The file-
based input MUST be a verifier-local policy artifact that can name revoked
witness keys and revoked witness-attestation digests.

#### Scenario: Revoked witness no longer satisfies release policy

- GIVEN a release whose witness set previously satisfied policy
- AND one of the required witness keys is later revoked under the configured
  trust policy
- WHEN verification runs with the updated policy input
- THEN the command reports policy degradation
- AND it keeps the underlying technical verification result separate

#### Scenario: Revocation input comes from a file-based policy artifact

- GIVEN a verifier-local revocation file naming revoked witness keys or witness
  attestation digests
- WHEN verification runs
- THEN the command applies that file-based policy input before quorum and
  independence evaluation
- AND it does not require an external revocation service
