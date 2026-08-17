## Phase 1: Implementation

- [x] [serial] r[mantle.release_evidence_provenance] Add `ProvenanceCoverage` type and optional `provenance_coverage` field to `ReleaseEvidenceManifest` in `crunch-release-core/src/manifest.rs`, with validation and 5 positive/negative tests.
- [x] [serial] r[mantle.release_evidence_provenance] Export `ProvenanceCoverage` and `PROVENANCE_COVERAGE_BOUNDARY` from the crate lib.rs.
- [x] [serial] r[mantle.release_evidence_provenance] Run validation: `cargo test -p crunch-release-core` (108 passed), `cargo fmt --check` (clean), `cargo clippy` (clean for new code).

## Phase 2: Archive

- [x] [serial] r[mantle.release_evidence_provenance] Verify and archive the change.

## Verification Coverage

| Scenario | Evidence |
|---|---|
| `cargo test -p crunch-release-core` passes | 108 passed (103 original + 5 new) |
| `cargo fmt --check` clean | fmt check passes |
| `cargo clippy` clean for new code | no new warnings from provenance_coverage |
| Manifest without coverage accepted | `validate_accepts_manifest_without_provenance_coverage` |
| Manifest with valid coverage accepted | `validate_accepts_manifest_with_valid_provenance_coverage` |
| Empty coverage rejected | `validate_rejects_provenance_coverage_with_empty_all_covered_ids` |
| Weakened boundary rejected | `validate_rejects_provenance_coverage_with_weakened_boundary` |
| Invalid binary hash rejected | `validate_rejects_provenance_coverage_with_invalid_binary_hash` |
