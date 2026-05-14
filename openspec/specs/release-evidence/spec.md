# release-evidence Specification

## Purpose
Define packaged release-evidence bundles that capture tracked-worktree source
plus verified vendored Cargo inputs, release binaries, full proof artifacts,
and prerequisite inventory, and allow
bundle-local integrity verification without over-claiming bootstrap or global
reproducibility guarantees.
## Requirements
### Requirement: Release evidence bundle format

Mantle MUST define a release evidence bundle format driven by a manifest.

A valid bundle MUST contain at least:
- a top-level manifest,
- the staged-source archive,
- the produced release binary artifact or artifacts,
- a bundled proof artifact from a full proof run,
- and the bundled prerequisite inventory.

The staged-source archive MUST be exported from the current tracked worktree
contents plus the verified `vendor-deps/` Cargo inputs required by self-build,
rather than from a stale `HEAD` archive.

The manifest MUST record at least the release identifier, source digest,
produced binary digest or digests, bundled proof-artifact digest, and bundled
prerequisite-inventory digest.

#### Scenario: Bundle manifest names required release evidence

- GIVEN a release evidence bundle produced by mantle
- WHEN the manifest is inspected
- THEN it names the release identifier and required evidence artifact digests
- AND another tool can determine which bundled files are mandatory for
  verification

#### Scenario: Staged-source archive reflects the current tracked worktree and vendored inputs

- GIVEN local tracked worktree content differs from `HEAD`
- AND `vendor-deps/` is present, verified against `Cargo.lock`, and ignored by Git
- WHEN mantle creates a release evidence bundle
- THEN the bundled staged-source archive reflects the current tracked worktree
- AND it includes the verified vendored Cargo inputs needed by self-build
- AND it does not silently fall back to a stale `HEAD` archive

### Requirement: CLI can create release evidence bundles

The CLI MUST provide a command to create a release evidence bundle from local
build and proof outputs.

Bundle creation MUST fail if required evidence artifacts are missing.

#### Scenario: Missing proof artifact blocks bundle creation

- GIVEN the operator requests a release evidence bundle
- BUT the required proof artifact is absent
- WHEN bundle creation runs
- THEN the command exits non-zero
- AND it names the missing required artifact

### Requirement: CLI can verify release evidence bundles

The CLI MUST provide a command to verify a release evidence bundle.

Verification MUST consume the bundle alone. It MUST confirm that required
bundled artifacts exist and that the manifest digests match those artifacts.
The manifest encoding itself MUST be canonical so verification can treat it as a
deterministic bundle description.

#### Scenario: Digest mismatch rejects bundle

- GIVEN a release evidence bundle whose binary artifact digest no longer matches
  the manifest
- WHEN bundle verification runs
- THEN the command exits non-zero
- AND it identifies the mismatched artifact

#### Scenario: Valid bundle verifies successfully

- GIVEN a valid release evidence bundle
- WHEN bundle verification runs
- THEN the command reports success
- AND it reports the verified release identifier

### Requirement: Proof evidence is linked to the bundled release candidate

The bundle manifest and verification path MUST bind the proof artifact and the
prerequisite inventory to the same release identifier, source digest, and binary
digest or digests recorded for the bundled release candidate.

A prerequisite-only check artifact such as `./scripts/prove-self-hosting.sh --check`
MUST NOT satisfy the bundle's proof-artifact requirement.

#### Scenario: Unrelated proof artifact is rejected

- GIVEN a bundle whose proof artifact does not correspond to the manifest's
  release identifier, source digest, or binary digest set
- WHEN bundle verification runs
- THEN the command exits non-zero
- AND it identifies the linkage mismatch

#### Scenario: Prerequisite-only check is rejected as proof evidence

- GIVEN an operator tries to build a release evidence bundle from a
  prerequisite-only proof check
- WHEN bundle creation or verification runs
- THEN the command exits non-zero
- AND it says that a full proof artifact is required

### Requirement: Release evidence claims stay narrower than bootstrap claims

Docs and manifests for release evidence MUST describe the bundle as packaged
integrity and proof-context evidence, not as automatic proof of full-source
bootstrap or global reproducibility.

README release-evidence wording and bootstrap-facing docs MUST use the same
bounded claim language for what `mantle release verify` proves.

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
- WHEN they read what `mantle release verify` proves today
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

### Requirement: Witnessed self-hosting workflow starts from verified release evidence

A full self-hosting proof bundle MUST be promotable into a witnessed release
verification workflow without hand-writing attestation or policy JSON, and
without requiring the witness environment to invent its own rebuild recipe.
ID: release.evidence.workflow.witnessed.selfhosting

The checked-in CLI workflow MUST let an operator:
- verify a release-evidence bundle,
- create a signed release attestation in a verification directory,
- scaffold verifier-local policy material for either self-proof-only or
  single-witness publication,
- export a public request directory for the witness environment,
- run the checked-in witness rebuild entry point
  `./scripts/rebuild-witness-request.sh` in the second environment so it
  verifies the request directory, rebuilds the published outputs, creates
  witness sidecars, and records rebuild evidence, and
- re-run verification after witness import to observe the technical class,
  policy status, and final class.

#### Scenario: Checked-in witness rebuild runner produces publishable witness material

- GIVEN an exported witness request directory from a release-evidence bundle
  that passes `mantle release verify`
- WHEN the witness operator runs `./scripts/rebuild-witness-request.sh` with
  signing key and witness metadata
- THEN the runner verifies the bundled release-evidence request before rebuild
- AND it emits witness sidecars whose release-attestation digest matches the
  request's release attestation
- AND it records rebuild evidence naming the replayed workflow identity and the
  rebuilt output digests

#### Scenario: Single matching witness promotes a self-hosting release

- GIVEN a release-evidence bundle with a full proof artifact that passes
  `mantle release verify`
- AND an operator scaffolds single-witness policy material for one trusted
  release signer and one trusted witness identity
- AND a witness rebuilder runs `./scripts/rebuild-witness-request.sh` and
  produces matching witness sidecars
- WHEN `mantle attest release-verify` runs with the trusted public keys after
  witness import
- THEN the technical class is `external-witness-match`
- AND the policy status is `satisfied`
- AND the final class is `quorum-satisfied`

#### Scenario: Workflow docs keep verified self-hosting claims bounded

- GIVEN a contributor follows the documented witnessed self-hosting workflow
- WHEN they read what a successful witness result proves
- THEN the docs describe external witness agreement plus configured policy
  satisfaction
- AND they do not claim full-source bootstrap or globally reproducible release
  artifacts

### Requirement: Witnessed self-hosting workflow MUST show key exchange steps

The documented witnessed self-hosting workflow MUST show how release signers and
witness rebuilders obtain the exact `--trusted-public-key <name:base64>` values
needed for verification.
ID: release.evidence.workflow.witnessed.keyexchange

#### Scenario: Witness workflow docs show trusted-key export

- GIVEN a contributor follows the documented witnessed self-hosting workflow
- WHEN they reach the `release-verify` step
- THEN the docs show how the release signer and witness rebuilder run
  `mantle attest key-show` for their signing keys
- AND the docs use those exported values in the `--trusted-public-key`
  examples instead of only placeholder text

### Requirement: Witnessed release workflow MUST export a portable request directory

Mantle MUST provide a checked-in way to export one verified release-evidence
bundle plus the signed release-attestation seed into a portable request
directory for a second environment.
ID: release.evidence.workflow.witnessed.request-export

The exported request directory MUST:
- contain a verified copy of the release-evidence bundle,
- contain the signed `release-attestation.json` and matching `.sig` sidecar,
- include deterministic metadata naming the release identifier and request
  layout version, and
- exclude private signing-key material.

#### Scenario: Publisher exports a witness request from verified artifacts

- GIVEN a release-evidence bundle that passes `mantle release verify`
- AND a verification directory containing a signed `release-attestation.json`
- WHEN the operator runs `mantle release witness-export`
- THEN the command writes a portable request directory with the verified bundle
- AND it copies `release-attestation.json` plus `release-attestation.json.sig`
- AND it does not copy any signing key file into the request directory

#### Scenario: Export rejects mismatched release identifiers

- GIVEN a release-evidence bundle whose release identifier differs from the
  supplied verification directory's signed release attestation
- WHEN the operator runs `mantle release witness-export`
- THEN the command exits non-zero
- AND it names the release-identifier mismatch

### Requirement: Cross-machine witnessed-self-hosting docs MUST show the request handoff

The documented witnessed-self-hosting workflow MUST show how a publisher exports
public verification material for a second environment, how the witness runs the
checked-in rebuild entry point from that request directory, and how the
publisher imports the returned witness sidecars without sharing signing keys.
ID: release.evidence.workflow.witnessed.crossmachine.docs

#### Scenario: Docs show publisher-to-witness rebuild flow

- GIVEN a contributor follows the witnessed-self-hosting workflow docs
- WHEN they reach the second-environment handoff step
- THEN the docs show `mantle release witness-export`
- AND the docs show `./scripts/rebuild-witness-request.sh` in the witness
  environment
- AND the docs describe the request directory as public verification material
- AND the docs keep publisher and witness signing keys in their respective
  environments

### Requirement: Canonical reproducible release report

Mantle MUST define a canonical reproducible release report that records byte-for-byte comparison evidence for every published release artifact.
ID: release.evidence.reproducible.report

The report MUST bind the release identifier, source archive digest, proof bundle digest, rebuild command identity, artifact names, byte lengths, BLAKE3 digests, and comparison result. The report identity MUST be the BLAKE3 digest of canonical compact JSON bytes.

#### Scenario: Byte-identical report is stable

- GIVEN two reproducibility checks over the same published and rebuilt artifact
  bytes
- WHEN the report is serialized
- THEN the canonical bytes are identical
- AND the report digest is identical

#### Scenario: One-byte drift is recorded

- GIVEN a rebuilt binary differs from the published binary by one byte
- WHEN reproducibility comparison runs
- THEN the report marks that artifact as mismatched
- AND it records the expected and observed BLAKE3 digests

### Requirement: CLI rebuilds and compares release artifacts

Mantle MUST provide a release reproducibility workflow that rebuilds published artifacts into a separate output area and compares the rebuilt bytes against the bundled release artifacts.
ID: release.evidence.reproducible.cli

The workflow MUST fail closed on missing artifacts, output-name drift, byte-length drift, digest drift, non-canonical report encoding, or proof-linkage mismatch. A successful prerequisite check MUST NOT count as a reproducible release check.

#### Scenario: Byte-identical rebuild succeeds

- GIVEN a release evidence bundle with published binary artifacts
- AND a reproducibility workflow rebuilds the same output names with identical
  bytes
- WHEN the release reproducibility command runs
- THEN it exits successfully
- AND it writes a canonical reproducible release report

#### Scenario: Missing rebuilt artifact fails

- GIVEN the published release bundle names `crunch-x86_64-linux`
- AND the rebuild output directory does not contain that artifact
- WHEN reproducibility comparison runs
- THEN the command exits non-zero
- AND the diagnostic names the missing rebuilt artifact

### Requirement: Reproducible release claim is evidence-gated

Mantle MUST reserve the bit-for-bit reproducible release claim for release evidence that includes a verified reproducibility report whose artifact set matches the published release artifact set.
ID: release.evidence.reproducible.claim.gate

Docs, manifests, and verifier output MUST keep ordinary bundle-local integrity separate from reproducible-release evidence. A release MUST NOT be labeled bit-for-bit reproducible when only self-proof or prerequisite evidence is present.

#### Scenario: Ordinary bundle is not labeled reproducible

- GIVEN a release evidence bundle verifies basic integrity and self-proof
- BUT it has no reproducibility report
- WHEN release verification reports classes
- THEN it does not label the release bit-for-bit reproducible
- AND it names the missing reproducibility evidence

#### Scenario: Verified report enables reproducible label

- GIVEN a release evidence bundle with a verified reproducibility report
- AND the report artifact set matches the published artifact set
- WHEN release verification reports classes
- THEN it may label the release bit-for-bit reproducible
- AND it includes the reproducibility report digest

### Requirement: Release evidence may carry independent agreement evidence

Mantle MUST allow release evidence or verification directories to carry an independent rebuild agreement report without making that report mandatory for bundle-local integrity verification.
ID: release.evidence.independent.agreement.attachment

Bundle verification MUST continue to distinguish basic bundle integrity, self-proof validity, matching external witnesses, and independent rebuild agreement. Missing agreement evidence MUST NOT invalidate a basic release-evidence bundle, but any present agreement report MUST verify against the release attestation and witness material it names. A release-evidence bundle MAY store the report at `independent-agreement/agreement-report.json`; a verification directory MAY store it at `agreement-report.json`. Discovery MUST reject any additional agreement-report filename for the same release to avoid ambiguous attachments.

#### Scenario: Bundle without agreement remains basic-valid

- GIVEN a release evidence bundle with valid source, binary, manifest, and proof
  artifacts
- AND no independent agreement report
- WHEN bundle verification runs
- THEN basic bundle verification succeeds
- AND the output says independent agreement evidence is absent

#### Scenario: Mismatched agreement report is rejected

- GIVEN a verification directory with an agreement report naming a different
  release-attestation digest
- WHEN release verification loads the report
- THEN verification fails for the agreement attachment
- AND the diagnostic names the digest mismatch

### Requirement: StageX-class verified release profile excludes quorum

Mantle MUST define a StageX-class verified release profile that requires full-source lineage proof and byte-identical rebuild evidence while explicitly excluding social quorum from the profile decision.
ID: release.evidence.stagex.profile.noquorum

The profile MUST be available through `mantle release verify <bundle-dir>
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
- WHEN `mantle release verify <bundle-dir> --require-stagex-no-quorum --json`
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

Mantle MUST reject seed-assisted, host-tool-trusted, legacy fetched-provider, self-proof-only, or prerequisite-only evidence for the StageX-class verified release profile.
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
