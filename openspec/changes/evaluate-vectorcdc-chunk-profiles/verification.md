# Verification: evaluate-vectorcdc-chunk-profiles

## 2026-05-08

- `openspec validate evaluate-vectorcdc-chunk-profiles --strict` → pass (`Change 'evaluate-vectorcdc-chunk-profiles' is valid`).
- `python ~/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify evaluate-vectorcdc-chunk-profiles --json || true` → expected incomplete-task warning for active spec-only change: `{'done': 1, 'todo': 8, 'in_progress': 0}` before V1 was marked complete.
- `git diff --check` → pass.
- `openspec_gate proposal/design/tasks evaluate-vectorcdc-chunk-profiles` → blocked locally: `openspec_gate unavailable`.

Implementation and benchmark tasks remain open intentionally; this change is an evaluation baseline, not a VectorCDC promotion.

## I2 deterministic chunker boundary

- Added `vendor/snix-castore/src/blobservice/chunker.rs` with:
  - `ChunkProfile` physical chunking parameters.
  - `ChunkBoundary` offset/length ranges.
  - `Chunker` pure boundary-selection trait.
  - `FastCdcChunker` default FastCDC implementation over finalized blob bytes.
  - `validate_chunk_boundaries()` for ordered, contiguous, non-overlapping full coverage.
- Positive verification:
  - `cargo check -p snix-castore --lib` → pass.
  - `cargo test -p snix-castore blobservice::chunker --lib` → pass, 6 tests.
  - `cargo test -p snix-castore test_chunk_and_upload --lib` → pass, 2 tests; existing object-store path still returns raw blob BLAKE3 digest after FastCDC upload.
- Negative verification:
  - chunk boundary validation rejects gaps, overlaps, zero-length ranges, and out-of-bounds non-final chunks.
- Formatting:
  - `cargo fmt --check` → pass.

## I3 benchmark corpus provenance

- Added `crates/crunch-delta/examples/vectorcdc_corpus.rs`, a deterministic corpus materializer for chunk-profile benchmarking.
- The corpus builder copies benchmark inputs to `target/vectorcdc-corpus/files/` and records provenance in `target/vectorcdc-corpus/manifest.json`.
- Committed provenance snapshot: `openspec/changes/evaluate-vectorcdc-chunk-profiles/evidence/i3-corpus-manifest.json`.
- Corpus contents from this run:
  - 20 entries total.
  - Source kinds: `repo-artifact`, `local-build-artifact`.
  - Mutation classes per source: `base`, `middle-64-byte-patch`, `prefix-insert`, `tail-append`.
  - Total corpus bytes across entries: `217349208`.
- Verification:
  - `cargo fmt --check -p crunch-delta` → pass after formatting.
  - `cargo check -p crunch-delta --example vectorcdc_corpus` → pass.
  - `cargo run -p crunch-delta --example vectorcdc_corpus -- target/vectorcdc-corpus` → pass, wrote 20 entries.
  - `python -m json.tool target/vectorcdc-corpus/manifest.json` → pass.
  - JSON invariant check asserted schema, at least 12 entries, presence of both repo and local-build artifact source kinds, and all mutation classes → pass.
