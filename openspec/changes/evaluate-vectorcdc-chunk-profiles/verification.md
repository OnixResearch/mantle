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
