## ADDED Requirements

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

#### Scenario: Release docs do not over-claim

- GIVEN a reader following the release evidence documentation
- WHEN they read what bundle verification proves
- THEN the docs distinguish release evidence from full-source bootstrap claims
- AND they do not claim bundle verification alone proves global reproducibility
