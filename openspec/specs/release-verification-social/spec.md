# release-verification-social Specification

## Purpose
Define verifier-local social trust policy for decentralized release
verification, including trusted roles, quorum, independence, revocations, and
policy-facing operator workflows.
## Requirements
### Requirement: Social trust policy MUST stay external to technical artifact digests

Mantle MUST define trusted-role, quorum, independence, and revocation policy
outside the canonical digest material for release attestations and witness
attestations.

#### Scenario: Policy update does not change technical artifact digests

- GIVEN an unchanged release attestation and unchanged witness attestations
- WHEN a local trust policy is tightened or relaxed
- THEN the attestation digests stay unchanged
- AND verification may change policy status without changing technical status

### Requirement: Verifier MUST support witness-role and quorum policy

Mantle MUST support policy rules for which witnesses count toward decentralized
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

Mantle MUST support policy rules that reject witness sets lacking required
independence across actors, keys, organizations, or equivalent configured
witness domains.

#### Scenario: Same actor cannot satisfy all required witness slots

- GIVEN multiple matching witness attestations from the same actor or same
  configured witness domain
- WHEN policy requires independent witnesses
- THEN verification reports policy failure
- AND it explains that the witness set is not independent enough

### Requirement: Social policy MUST define revocation and dispute handling

Mantle MUST define how revocations and disputes affect the policy status of
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

### Requirement: CLI MUST scaffold verifier-local release policy artifacts

Mantle MUST provide `mantle attest policy-init <verification-dir>` to write the
first-phase `policy.json` and `revocations.json` files for a verification
directory without requiring the operator to hand-author JSON.
ID: release.verification.social.policy.scaffold.cli

The command MUST:
- refuse to overwrite existing policy or revocation files unless an explicit
  overwrite flag is set,
- support at least `self-proof-only` and `single-witness` profiles,
- require explicit trusted release signer names for all profiles,
- require explicit trusted witness identities for witness-count profiles, and
- write deterministic JSON that `mantle attest release-verify` can consume
  directly.

#### Scenario: Self-proof-only profile initializes zero-witness policy

- GIVEN a verification directory with a release attestation and no policy files
- WHEN the operator initializes the `self-proof-only` profile with one trusted
  release signer
- THEN `policy.json` sets `min_matching_witnesses` to `0`
- AND `policy.json` leaves `trusted_witness_signers` empty
- AND `revocations.json` exists with empty revocation arrays

#### Scenario: Single-witness profile initializes quorum policy

- GIVEN a verification directory with a release attestation and no policy files
- WHEN the operator initializes the `single-witness` profile with one or more
  trusted witness identities
- THEN `policy.json` sets `min_matching_witnesses` to `1`
- AND `policy.json` sets `independence_field` to `witness_identity`
- AND `policy.json` records the configured trusted witness identities

#### Scenario: Existing policy artifacts are not clobbered by default

- GIVEN a verification directory with existing `policy.json` or
  `revocations.json`
- WHEN the operator initializes policy without an explicit overwrite flag
- THEN the command exits non-zero
- AND the existing policy artifacts remain unchanged

### Requirement: Policy defines independent rebuild agreement thresholds

Mantle MUST let verifier-local policy define the witness count and independence domains required for independent rebuild agreement.
ID: release.verification.social.independent.agreement.policy

The policy MUST support at least witness identity, signer key name, and host-class independence selectors. Policy evaluation MUST reject witness sets that meet the count threshold only by duplicating the same configured independence domain.

#### Scenario: Host-class independence is required

- GIVEN policy requires two matching witnesses across distinct host classes
- AND two witnesses match the release digests but report the same host class
- WHEN policy evaluation runs
- THEN independent rebuild agreement is not satisfied
- AND the diagnostic names the repeated host class

#### Scenario: Signer-key independence is required

- GIVEN policy requires two matching witnesses across distinct signer key names
- AND two witnesses match with distinct trusted signing keys
- WHEN policy evaluation runs
- THEN the independence predicate is satisfied

