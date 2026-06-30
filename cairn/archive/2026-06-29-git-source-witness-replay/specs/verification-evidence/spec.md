# Verification Evidence Specification Delta

## ADDED Requirements

### Requirement: Git source witness replay [r[verification_evidence.git_source_witness_replay]]

Mantle release witness rebuilds that are requested to support a Git-source claim MUST derive the release source archive from a release-declared Git origin, verify the selected commit and configured ref/tag policy, regenerate the deterministic Mantle source archive, verify its BLAKE3 digest equals the release manifest source archive digest, and fail closed before extraction, workflow launch, or signing when any source-origin check fails.

#### Scenario: release evidence records Git source origin

GIVEN an operator creates release evidence with Git source-origin metadata
WHEN Mantle writes `manifest.json`
THEN the manifest MUST record the Git remote URL, pinned commit, archive profile/version, and generated source archive BLAKE3 digest
AND the generated digest MUST match `source_archive.digest_blake3` and `proof_linkage.source_archive_digest_blake3`.

#### Scenario: Git source archive is deterministic

GIVEN a pinned Git commit contains the files admitted by Mantle's release source policy
WHEN Mantle reconstructs the release source archive from that commit
THEN the archive member set, member order, path normalization, metadata normalization, and BLAKE3 digest MUST be deterministic for equivalent Git tree contents
AND private runtime paths, lifecycle evidence paths, absolute paths, parent-directory paths, and unsupported submodule entries MUST NOT enter the archive.

#### Scenario: Git-source witness fetches before rebuilding

GIVEN a witness request contains a release manifest with Git source acquisition metadata
WHEN `mantle release witness-rebuild --require-git-source` prepares scratch
THEN it MUST fetch the declared Git origin, resolve the pinned commit/ref according to the configured policy, regenerate the source archive, and verify the regenerated archive digest before extracting or launching the workflow.

#### Scenario: tag or ref policy fails closed

GIVEN a release manifest declares a tag/ref or signature policy for Git source acquisition
WHEN the witness cannot prove that the fetched Git object satisfies that policy
THEN the witness rebuild MUST exit non-zero before extracting source or launching the workflow
AND the diagnostic MUST identify the failed Git source policy stage without leaking credential material.

#### Scenario: generated archive digest mismatch fails closed

GIVEN a witness fetches the declared Git origin but regenerates a source archive whose BLAKE3 digest differs from `source_archive.digest_blake3`
WHEN `mantle release witness-rebuild --require-git-source` prepares scratch
THEN it MUST exit non-zero before extracting source or launching the workflow
AND the diagnostic and audit metadata MUST name the expected digest, generated digest, remote URL, and pinned commit.

#### Scenario: non-Git source claims stay bounded

GIVEN a witness rebuild uses copied source or external-archive source acquisition instead of `--require-git-source`
WHEN the result is summarized
THEN the claim MUST remain bounded to the selected non-Git source mode
AND it MUST NOT claim Git-derived source replay.
