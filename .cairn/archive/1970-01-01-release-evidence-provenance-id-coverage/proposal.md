## Why

Mantle's release-evidence bundles record binary hashes, proof bundles, and reproducibility reports, but they have only a compact optional place to summarize external provenance coverage when an operator already has adapter-produced traceability data. This change adds an optional `provenance_coverage` field to the `ReleaseEvidenceManifest` so a release can record opaque source/function/requirement ID coverage without making Mantle responsible for Valence, Octet, Trellis, or Cairn semantics.

## What Changes

- Add `ProvenanceCoverage` type to `crunch-release-core/src/manifest.rs`: carries `binary_hash`, `covered_source_ids`, `covered_function_object_ids`, `covered_requirement_ids`, and `coverage_boundary` (non-claim).
- Add optional `provenance_coverage: Option<ProvenanceCoverage>` field to `ReleaseEvidenceManifest` (backward-compatible: `#[serde(default, skip_serializing_if = "Option::is_none")]`).
- Add `validate_provenance_coverage` validation: fails closed on invalid binary hash, weakened boundary, or empty coverage (at least one covered ID required).
- Export `ProvenanceCoverage` and `PROVENANCE_COVERAGE_BOUNDARY` from the crate.
- 5 positive and negative tests: valid coverage accepted, no-coverage accepted, empty-coverage rejected, weakened-boundary rejected, invalid-binary-hash rejected.

## Impact

- **Files**: `crates/crunch-release-core/src/manifest.rs`, `crates/crunch-release-core/src/lib.rs`.
- **Testing**: `cargo test -p crunch-release-core` (108 passed), `cargo fmt --check` (clean), `cargo clippy` (clean for new code).
- **Non-claims**: provenance coverage records identity and linkage only. It does not prove behavioral correctness, semantic equivalence, adapter-side semantic validity, or that the binary satisfies the requirements.
