# Verification Evidence Delta

## ADDED Requirements

### Requirement: Optional build-witness policy

r[verification_evidence.optional_build_witness_policy] Mantle MUST support trusted release build-witness collection and verification without requiring witness quorum, and it MUST apply witness quorum only when an operator selects an explicit positive-threshold policy.

#### Scenario: Optional profile has no witness

GIVEN an operator selects `optional-witness` with a valid trusted release signer and no witness sidecars are present
WHEN Mantle evaluates release policy
THEN the release policy MUST report witness quorum as `not-required`
AND the absence of witness evidence MUST NOT make the release policy insufficient.

#### Scenario: Optional profile verifies available witnesses

GIVEN an operator selects `optional-witness`, configures trusted witness identities and public keys, and supplies one or more witness sidecars
WHEN Mantle evaluates release policy
THEN every signature-valid digest-matching witness MUST remain visible as accepted individual evidence
AND Mantle MUST NOT claim that a quorum was required or satisfied.

#### Scenario: Invalid optional witness stays uncounted

GIVEN optional witness material has an unknown key, bad signature, revoked key or attestation, stale request, wrong release digest, duplicate identity, or rebuilt digest mismatch
WHEN Mantle evaluates release policy
THEN it MUST classify that witness as skipped or failed with a deterministic reason
AND it MUST NOT promote that witness into accepted evidence or fail an otherwise valid release solely because optional quorum was not required.

#### Scenario: Operator selects witness quorum

GIVEN an operator selects `witness-quorum` with an explicit positive matching-witness minimum, one supported independence selector, and sufficient trusted witness identities
WHEN Mantle constructs and evaluates the policy
THEN it MUST require the configured count of matching witnesses with distinct selected independence values
AND only cryptographically valid, active, digest-matching witnesses accepted by that policy MAY satisfy the quorum.

#### Scenario: Selected quorum is insufficient

GIVEN an explicit witness-quorum policy requires more accepted independent witnesses than the supplied evidence provides
WHEN Mantle evaluates the selected policy
THEN it MUST report quorum as `insufficient` with required and observed counts
AND it MUST preserve valid individual witness evidence without changing bootstrap parity, StageX no-quorum status, or an unrelated policy decision.

#### Scenario: Quorum parameters are invalid

GIVEN a witness-quorum request omits its minimum, uses a zero or unbounded minimum, selects an unsupported independence field, or cannot name enough trusted identities
WHEN Mantle plans policy creation
THEN it MUST reject the request before writing policy or revocation files
AND it MUST NOT fall back to optional, self-proof, or single-witness behavior.

#### Scenario: Compatibility profiles retain meaning

GIVEN an existing `self-proof-only`, `single-witness`, or hand-authored `mantle-release-policy-v1` policy
WHEN Mantle loads or evaluates it after this change
THEN its canonical policy fields and prior threshold meaning MUST remain unchanged
AND profile additions MUST NOT rewrite existing policy files implicitly.

#### Scenario: StageX technical verification stays no-quorum

GIVEN bootstrap parity or release verification selects the StageX no-quorum technical profile
WHEN no explicit witness-quorum policy is also selected
THEN Mantle MUST evaluate the technical profile without a witness-count requirement
AND any later witness evaluation MUST remain a separate policy result with separate evidence and non-claims.
