## Why

Mantle can now prove that a witness fetched a release-declared external source archive and verified its BLAKE3 digest before rebuilding. That is stronger than replaying only a copied request bundle, but it still trusts publisher-provided archive bytes. A witness that wants an independent source-origin claim needs to derive those archive bytes from a VCS origin, not just download an archive artifact.

This change defines the next release-verification boundary: release evidence may name a Git origin, pinned commit, and optional signed tag policy; witnesses can fetch that Git origin independently, materialize the deterministic Mantle release source archive, compare its BLAKE3 digest with the manifest, and only then rebuild/sign.

## What Changes

- Add Git source-origin metadata to release evidence alongside the existing external-archive source acquisition mode.
- Add deterministic source archive reconstruction from a pinned Git commit/ref using Mantle's release-source inclusion rules.
- Add `mantle release witness-rebuild --require-git-source` (or an equivalent strict mode) that fetches Git source, verifies commit/ref/tag policy, reconstructs the source archive, compares BLAKE3, and fails before extraction/workflow launch on any mismatch.
- Extend witness audit metadata to distinguish copied-source, external-archive-source, and Git-derived-source replay.
- Add local-repo positive/negative tests and an Aspen1 replay transcript proving the Git-source path on a separate host.

## Impact

- **Files**: `crates/crunch-release-core/src/manifest.rs`, `src/release_evidence.rs`, `src/release_cmd.rs`, `src/witness_rebuild.rs`, `src/main.rs`, test fixtures/helpers, and release/witness Cairn evidence.
- **Testing**: focused core manifest tests, deterministic archive fixture tests, witness rebuild tests with local `file://` Git remotes, CLI parse tests, Cairn validation/gates, and one Aspen1 end-to-end replay.

## Non-goals

- Do not claim compiler correctness, full bootstrap correctness, or deploy success.
- Do not require every release to use Git-origin metadata; external archive replay remains valid but bounded.
- Do not implement generic Git hosting APIs. The witness should use Git protocol/local transports and deterministic archive logic owned by Mantle.
