## Context

`release witness-export` currently copies the full release evidence bundle, including the bundled source archive, into a witness request. `witness-rebuild` validates the copied bundle and extracts `manifest.source_archive.relative_path` from that request. The resulting claim is bounded to publisher-supplied source bytes.

The next stronger claim needs two separate facts: the release must describe where the witness can independently acquire the source archive, and the witness must prove it fetched bytes whose digest matches the manifest before using them.

## Decisions

### 1. External archive origin first

**Choice:** add a `source_acquisition` manifest field with kind `external-archive`, `url`, and `digest_blake3`, where `digest_blake3` MUST equal `source_archive.digest_blake3`.

**Rationale:** Mantle's release source archive includes verified vendored Cargo inputs and source packaging policy, so raw Git checkout reconstruction is not yet equivalent to the release archive. Binding an external source archive URL gives witnesses independent acquisition without changing the source-packaging contract. Git tag/commit acquisition can layer on later once vendored source material has a first-class origin model.

### 2. Fail-closed witness gate

**Choice:** add `mantle release witness-rebuild --require-independent-source`. When the flag is present, the plan MUST require `source_acquisition`, fetch the external archive, verify the BLAKE3 digest, and extract the fetched archive instead of the copied request archive.

**Rationale:** Existing witness requests remain replayable. Stronger independent-source claims become explicit and fail closed instead of silently falling back to copied source bytes.

### 3. Audit source acquisition

**Choice:** extend witness audit metadata with a nullable `source_acquisition` object containing mode, URL, digest, fetched path, and status/error.

**Rationale:** Operators need the audit to distinguish copied-source replay from independently fetched-source replay. Failure records should explain whether the request lacked an origin, fetching failed, or digest verification failed.

## Risks / Trade-offs

- External archive URL support proves independent acquisition of release source bytes, not independent Git tag reconstruction.
- Network fetch during witness replay can fail for external reasons; failures stay explicit in audit metadata.
- The manifest schema remains version `mantle-release-evidence-v1` with an optional field for backward compatibility.
