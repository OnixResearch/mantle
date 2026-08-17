# Verification Evidence Delta

## ADDED Requirements

### Requirement: Release source-review evidence

r[verification_evidence.release_source_review_evidence] Mantle MUST bind any policy-counted source-review approval to the exact released source and independently verified reviewer authority, and it MUST require source-review approval only when an operator selects an explicit reviewed-source policy.

#### Scenario: Generic release does not require source review

GIVEN a release has valid source, proof, artifact, and release-signature evidence but no source-review attachment
WHEN Mantle evaluates the generic release profile
THEN source-review status MUST be `not-required`
AND missing source-review evidence MUST NOT block the generic release solely because no reviewed-source policy was selected.

#### Scenario: Optional review evidence is present

GIVEN a generic release carries a valid source-review attachment bound to its exact source archive BLAKE3
WHEN Mantle verifies the release
THEN it MUST verify and report the review evidence as a separate bounded fact
AND it MUST NOT promote optional approval into a required-policy or source-correctness claim.

#### Scenario: Reviewed-source policy is satisfied

GIVEN an operator selects a reviewed-source policy with an explicit distinct-reviewer threshold and accepted reviewer authority
AND the attachment contains sufficient current approved statements over the exact release source and review policy
WHEN Mantle independently verifies the attachment and signatures
THEN it MUST report the reviewed-source policy as satisfied
AND it MUST bind the source digest, claim root, policy digest, distinct reviewer full-key BLAKE3 identities, approval statement identities, Valence evidence identity, and attachment identity into release evidence.

#### Scenario: StageX-inspired source review uses two distinct reviewers

GIVEN an operator selects the StageX-inspired reviewed-source preset
WHEN Mantle evaluates source-review policy
THEN it MUST require the named two-reviewer threshold from typed policy and count only distinct authorized full public-key identities
AND duplicate names, duplicate signatures, one key under multiple labels, excluded author identity, unknown keys, and revoked keys MUST NOT inflate the approval count.

#### Scenario: Review subject is stale or mismatched

GIVEN an approval names a different source archive digest, source revision, claim root, review policy, or release subject
WHEN Mantle verifies the source-review attachment
THEN the reviewed-source policy MUST fail with a deterministic mismatch or stale-evidence reason
AND producer status fields or matching reviewer display names MUST NOT authorize the release.

#### Scenario: Review evidence is malformed or insufficient

GIVEN required review evidence is missing, malformed, signature-invalid, role-invalid, revoked, unsupported, `needs_revision`, threshold-insufficient, or linked through tampered Valence evidence
WHEN Mantle evaluates a selected reviewed-source policy
THEN it MUST fail that policy before release eligibility
AND it MUST preserve bounded diagnostics without fabricating approvals or silently falling back to optional mode.

#### Scenario: Reviewer and build-witness roles remain separate

GIVEN one key or person appears in source-review, release-signing, or build-witness material
WHEN Mantle evaluates role-specific policies
THEN each signature MUST count only for its domain-separated statement and authorized role
AND source-review approvals MUST NOT satisfy build-witness quorum, while build-witness attestations MUST NOT satisfy source-review policy.

#### Scenario: Build witnesses remain optional

GIVEN a reviewed-source policy is satisfied and no build-witness quorum policy is selected
WHEN Mantle verifies the release
THEN it MUST NOT require a build-witness count
AND any available build-witness evidence MUST remain governed by the separate optional-witness or explicit witness-quorum policy.

#### Scenario: Mantle does not own review workflow

GIVEN Cairn or another admitted producer creates review workflow state, findings, dispositions, and signed approvals
WHEN Mantle consumes the stable review artifact
THEN Mantle MUST limit its role to exact identity, signature, linkage, policy, and release-subject verification
AND it MUST NOT claim reviewer competence, review completeness, source correctness, collaboration convergence, or pull-request approval.
