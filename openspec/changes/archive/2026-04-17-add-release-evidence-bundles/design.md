# Design: Add release evidence bundles

## Context

crunch already has artifacts worth preserving for a release claim:

- staged source identity
- built binary digests
- self-hosting proof bundles
- prerequisite inventory and environment facts

Today those artifacts are scattered across local output directories and README
text. A release process should package them into one stable structure that can
be copied, inspected, and verified later.

## Goals / Non-Goals

**Goals:**

- define one bundle format for release evidence
- generate bundles from local proof and build outputs
- verify bundle integrity later without re-running the proof automatically
- bind bundled proof evidence to the same release, source, and binary digests
- keep bootstrap and release-evidence claims clearly separated

**Non-Goals:**

- assert that every bundle proves global reproducibility
- replace the underlying self-hosting proof
- require online services to verify a bundle
- add signing policy or key-distribution policy in the first version

## Decisions

### 1. Bundle around a manifest, not a loose directory

**Choice:** the release artifact is driven by a canonical manifest that names
required files, digests, and claim metadata.

**Rationale:** a stable manifest makes later verification simple and keeps the
bundle inspectable without guessing which files matter. Canonicalization also
ensures the same logical bundle description yields byte-identical manifest
output.

**Implementation:** the manifest records at least the release identifier,
source digest, binary digest or digests, proof bundle digest, prerequisite
inventory digest, and the commands or workflow version used to produce them.
Canonical manifest encoding uses a deterministic field order and stable
serialization rules so the same logical content always produces the same bytes.
The bundle contains mandatory members for the staged-source archive, release
binary artifact or artifacts, proof bundle copy, and prerequisite inventory.

### 2. Verification checks bundle integrity, not global truth

**Choice:** `crunch release verify` checks that the bundle is self-consistent and
that embedded digests match the bundled artifacts. It does not silently claim
that the world has independently reproduced the release.

**Rationale:** bundle verification should be honest about what it proves.

**Implementation:** verification fails on missing required artifacts, digest
mismatch, schema mismatch, or claim-boundary violations in the manifest.
Verification consumes the bundle alone and does not require extra external
inputs.

### 3. Bundle the full proof artifact, not prerequisite-only checks

**Choice:** the required proof artifact for a release bundle comes from a full
proof run, not from prerequisite-only checks such as
`./scripts/prove-self-hosting.sh --check`.

**Rationale:** prerequisite-only checks do not establish the same claim as a
finished proof bundle.

**Implementation:** the bundle generator requires proof outputs that contain the
fields expected from a full proof run, such as later-stage binary identity and
proof summary data. A prerequisite-only check output does not satisfy that
contract.

### 4. Proof outputs are bundled, not merely referenced by ephemeral paths

**Choice:** the bundle contains the proof bundle or a stable embedded copy of
its required contents, rather than only pointing at a local temporary path.

**Rationale:** release evidence needs to travel with the release candidate.

**Implementation:** bundle creation copies the required proof artifacts and
records their digests in the manifest. The staged-source archive is exported
from the current tracked worktree contents rather than from a stale `HEAD`
archive. Bundle creation also writes an internal linkage record binding the
bundled proof and inventory artifacts to the same release identifier, source
digest, and binary digests recorded in the top-level manifest.

## Risks / Trade-offs

**[False confidence]**
Operators may over-read what bundle verification proves.

**Mitigation:** make the manifest and docs explicit that bundle verification is
integrity and claim-packaging evidence, not full-source bootstrap proof.

**[Bundle size]**
Proof artifacts can be large.

**Mitigation:** define the minimum required proof contents clearly and avoid
bundling irrelevant scratch state.

**[Process drift]**
Release commands can drift away from the documented proof workflow.

**Mitigation:** record workflow version or command identity in the manifest and
add verification tests for missing required fields.
