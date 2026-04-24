# release-evidence Specification

## Purpose
Define packaged release-evidence bundles that capture tracked-worktree source,
release binaries, full proof artifacts, and prerequisite inventory, and allow
bundle-local integrity verification without over-claiming bootstrap or global
reproducibility guarantees.
## Requirements
### Requirement: Release evidence bundle format

Crunch MUST define a release evidence bundle format driven by a manifest.

A valid bundle MUST contain at least:
- a top-level manifest,
- the staged-source archive,
- the produced release binary artifact or artifacts,
- a bundled proof artifact from a full proof run,
- and the bundled prerequisite inventory.

The staged-source archive MUST be exported from the current tracked worktree
contents rather than from a stale `HEAD` archive.

The manifest MUST record at least the release identifier, source digest,
produced binary digest or digests, bundled proof-artifact digest, and bundled
prerequisite-inventory digest.

#### Scenario: Bundle manifest names required release evidence

- GIVEN a release evidence bundle produced by crunch
- WHEN the manifest is inspected
- THEN it names the release identifier and required evidence artifact digests
- AND another tool can determine which bundled files are mandatory for
  verification

#### Scenario: Staged-source archive reflects the current tracked worktree

- GIVEN local tracked worktree content differs from `HEAD`
- WHEN crunch creates a release evidence bundle
- THEN the bundled staged-source archive reflects the current tracked worktree
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

### Requirement: Witnessed self-hosting workflow starts from verified release evidence

A full self-hosting proof bundle MUST be promotable into a witnessed release
verification workflow without hand-writing attestation or policy JSON.
ID: release.evidence.workflow.witnessed.selfhosting

The checked-in CLI workflow MUST let an operator:
- verify a release-evidence bundle,
- create a signed release attestation in a verification directory,
- scaffold verifier-local policy material for either self-proof-only or
  single-witness publication,
- add matching witness attestations from rebuilt binaries, and
- re-run verification to observe the technical class, policy status, and final
  class.

#### Scenario: Single matching witness promotes a self-hosting release

- GIVEN a release-evidence bundle with a full proof artifact that passes
  `crunch release verify`
- AND an operator scaffolds single-witness policy material for one trusted
  release signer and one trusted witness identity
- AND a witness rebuilder publishes rebuilt binaries matching the released
  digests
- WHEN `crunch attest release-verify` runs with the trusted public keys
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

