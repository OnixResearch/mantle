## Context

The archived `independent-source-witness-replay` change added `source_acquisition.kind = external-archive` and proved Aspen1 could run `--require-independent-source`, fetch a release-declared archive URL, verify its BLAKE3 digest, and reach quorum. That evidence remains bounded: the archive bytes were prepared by the publisher and transferred as an archive artifact.

A Git-origin witness claim needs one more boundary. The release manifest must carry enough source-origin facts for a witness to independently fetch source history, select the intended commit/ref/tag, materialize the same release source archive bytes, and compare that archive digest to `source_archive.digest_blake3` before the self-hosting workflow sees the source tree.

## Decisions

### 1. Keep archive digest as the acceptance root

**Choice:** Git-source replay accepts only when the witness-generated source archive BLAKE3 equals `manifest.source_archive.digest_blake3` and `manifest.proof_linkage.source_archive_digest_blake3`.

**Rationale:** The existing release, proof, and witness contracts already bind the source archive digest. Reusing that digest avoids inventing a parallel trust root and keeps external-archive and Git-source claims comparable.

### 2. Add a typed Git source acquisition mode

**Choice:** Extend release source acquisition metadata with a Git mode carrying at least: remote URL, commit SHA, optional ref/tag, archive profile/version, and BLAKE3 archive digest. The schema must reject unsupported URL schemes, empty refs, malformed commit IDs, digest mismatch, and credential-bearing URLs.

**Rationale:** The current external archive shape (`kind`, `url`, `digest_blake3`) is insufficient to identify a Git commit and optional tag policy. A typed mode keeps validation deterministic while preserving backward compatibility for existing external-archive manifests.

### 3. Deterministic archive reconstruction uses Mantle's release source inclusion policy

**Choice:** Witness Git replay must materialize source from the selected commit and produce the same canonical release source archive that `mantle release create` would package for that commit. The pure core should decide normalized member paths, ordering, exclusion rules, and digest inputs from in-memory entries; the shell handles Git fetch/checkout and tar I/O.

**Rationale:** The important logic is deterministic archive membership, not Git process orchestration. Keeping path filtering/order in a functional core makes fixture tests cheap and prevents host checkout details from leaking into archive bytes.

### 4. Witness Git replay is opt-in and fail-closed

**Choice:** Add a strict witness gate such as `--require-git-source`. When enabled, the witness must reject manifests without Git source metadata, unsupported remotes, tag/signature policy failures, checkout mismatches, archive digest mismatches, and archive-generation errors before extracting source or launching the proof workflow.

**Rationale:** Operators need an explicit claim boundary. Existing copied-source and external-archive modes must not accidentally be summarized as Git-origin replay.

### 5. Audit records source derivation details, not secrets

**Choice:** Witness audit metadata should record source mode, remote URL with credentials forbidden, commit SHA, ref/tag if present, generated archive digest, generated archive path, verification status, and failure reason. It must not record private tokens or credential material.

**Rationale:** Release-readiness evidence needs enough data to replay and review the source-origin claim without leaking secrets.

## Risks / Trade-offs

- Git archive byte-for-byte parity can be fragile if source inclusion policy depends on working-tree state. Mitigate by deriving archive members from the checked-out commit plus explicit allow/exclude rules and by testing local bare repositories.
- Tag signature verification may require operator-provided trust roots. Implement the core metadata and fail-closed hooks first; keep unsigned commit pinning as a separately named policy mode if tag trust roots are not configured.
- Large repositories may make witness replay expensive. Keep bounded limits for path count, member size, archive size, and fetch timeouts.
- Git submodules can hide additional source origins. Reject submodules initially unless the manifest records and verifies them explicitly.
