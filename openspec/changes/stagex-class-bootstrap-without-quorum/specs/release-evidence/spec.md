## ADDED Requirements

### Requirement: StageX-class verified release profile excludes quorum

Crunch MUST define a StageX-class verified release profile that requires full-source lineage proof and byte-identical rebuild evidence while explicitly excluding social quorum from the profile decision.
ID: release.evidence.stagex.profile.noquorum

The profile MUST be available through `crunch release verify <bundle-dir>
--require-stagex-no-quorum` and JSON field `stagex_no_quorum`. The profile MUST
require a verified release-evidence bundle, a full self-build proof whose
provider kind is StageX-class lineage, a canonical reproducibility report whose
artifact set matches the published artifact set, and verifier output that keeps
quorum status unset, absent, or explicitly out of scope. The profile output MUST
bind the proof bundle digest and reproducibility report digest in the same
canonical verification result so the verified-build claim cannot be assembled
from unrelated artifacts. The profile MUST NOT report `quorum-satisfied`,
require multiple maintainers, or use multi-signer policy as a success condition.
A later quorum change MAY layer social policy on top of this profile.

#### Scenario: Full-source and reproducible evidence satisfy profile

- GIVEN a release-evidence bundle verifies bundle-local integrity
- AND its proof bundle records StageX-class lineage provider evidence
- AND its reproducibility report verifies byte-identical artifacts for the
  complete published artifact set
- WHEN `crunch release verify <bundle-dir> --require-stagex-no-quorum --json`
  runs
- THEN `stagex_no_quorum.status` is `satisfied`
- AND `stagex_no_quorum.class` is `stagex-verified-no-quorum`
- AND `stagex_no_quorum.quorum_status` is `not_evaluated`

#### Scenario: Missing reproducibility report blocks profile

- GIVEN a release-evidence bundle verifies bundle-local integrity
- AND its proof bundle records StageX-class lineage provider evidence
- BUT no verified reproducibility report is present
- WHEN the StageX-class no-quorum profile is evaluated
- THEN the profile reports unsatisfied
- AND the diagnostic names the missing reproducibility report

#### Scenario: Profile binds proof and reproducibility digests

- GIVEN a release-evidence bundle with StageX-class lineage proof
- AND a verified reproducibility report for the same release artifact set
- WHEN the StageX-class no-quorum profile is evaluated
- THEN the canonical profile result records the proof bundle digest
- AND it records the reproducibility report digest
- AND both digests are tied to the same release identifier and artifact set

#### Scenario: Quorum evidence is not required

- GIVEN full-source lineage proof and reproducibility evidence are valid
- AND no multi-signer quorum evidence is present
- WHEN the StageX-class no-quorum profile is evaluated
- THEN the profile can still report satisfied
- AND it does not emit `quorum-satisfied`

### Requirement: StageX-class profile rejects weaker bootstrap evidence

Crunch MUST reject seed-assisted, host-tool-trusted, legacy fetched-provider, self-proof-only, or prerequisite-only evidence for the StageX-class verified release profile.
ID: release.evidence.stagex.profile.rejects.weaker

The verifier MUST distinguish StageX-class lineage proof from existing
seed-assisted self-hosting proof, source-root provider proof that still trusts
host compiler/build tools, ordinary bundle integrity, external witness match,
and independent agreement evidence. Those weaker evidence classes MAY remain
valid for their own reports, but they MUST NOT satisfy the StageX-class
no-quorum profile.

#### Scenario: Legacy fetched-provider proof is rejected

- GIVEN a release-evidence bundle with a full self-hosting proof using the
  legacy fetched provider
- AND the bundle otherwise verifies
- WHEN the StageX-class no-quorum profile is evaluated
- THEN the profile reports unsatisfied
- AND the diagnostic says the provider evidence is seed-assisted

#### Scenario: Source-root proof with host compiler trust is rejected

- GIVEN a release-evidence bundle with a source-root provider proof that records
  host `cc`, `make`, or archive tools as trusted roots
- WHEN the StageX-class no-quorum profile is evaluated
- THEN the profile reports unsatisfied
- AND the diagnostic names the remaining host-tool trust edge

#### Scenario: External witness match alone is insufficient

- GIVEN a release has external witness material whose rebuilt digests match
- BUT the bootstrap proof is not StageX-class lineage proof
- WHEN the StageX-class no-quorum profile is evaluated
- THEN the profile reports unsatisfied
- AND it keeps witness agreement separate from bootstrap lineage evidence
