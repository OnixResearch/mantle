## ADDED Requirements

### Requirement: Release verification consumes deterministic proof receipts without overclaiming

Release verification MUST promote a release artifact beyond `self-rebuild-match`
to a stronger deterministic-release claim only when it is backed by
deterministic build proof receipts whose verdicts are `deterministic-match` for
every required artifact output in the release digest set.

A deterministic-release claim MUST remain scoped to the named release artifacts,
workflow identity, derivation identities, toolchain/provider identities, and
recorded proof matrix. It MUST NOT claim global Mantle determinism, all-package
reproducibility, or full-source bootstrap determinism unless separate evidence
proves those broader claims.

#### Scenario: Release has deterministic proof for every artifact

- GIVEN a release evidence bundle with required artifact digests
- AND each required artifact has a deterministic-build proof receipt with verdict
  `deterministic-match`
- AND the receipt BLAKE3 digest set equals the release artifact digest set
- WHEN release verification evaluates deterministic claim eligibility
- THEN it may report a deterministic-release claim scoped to those artifacts

#### Scenario: One artifact lacks deterministic proof

- GIVEN a release with multiple required artifacts
- AND at least one artifact lacks a `deterministic-match` proof receipt
- WHEN release verification evaluates deterministic claim eligibility
- THEN the release remains at the strongest lower satisfied proof class
- AND the report identifies the missing deterministic proof evidence

#### Scenario: Deterministic release claim remains bounded

- GIVEN a release artifact has a valid deterministic-build proof receipt
- WHEN Mantle renders human-readable release verification output
- THEN it states the claim scope as release-artifact determinism under the
  recorded workflow and proof matrix
- AND it does not call the whole build system Nix-like deterministic by default
