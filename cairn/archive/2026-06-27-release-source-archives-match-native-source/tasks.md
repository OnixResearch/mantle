# Tasks

## Phase 1: Implementation

- [x] [serial] Update release source archive path selection to include tracked files visible to native path-source hashing. r[verification_evidence.release_source_archive_native_parity]
  - Evidence: `evidence/source-archive-native-parity-validation.md` records the implementation and focused validation; `src/release_source.rs` now uses native-source skip constants instead of the self-build staging allowlist.
- [x] [serial] Add positive and negative source archive tests for tracked package files, verified vendor files, untracked files, and private skipped paths. r[verification_evidence.release_source_archive_native_parity]
  - Evidence: `release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files` passed for both `mantle` and `crunch` binary targets in `evidence/source-archive-native-parity-validation.md`.

## Phase 2: Validation

- [x] [serial] Run focused release source tests and record output. r[verification_evidence.release_source_archive_native_parity]
  - Evidence: `evidence/source-archive-native-parity-validation.md` includes exact `cargo test -p mantle --bin mantle ...` and `cargo test -p mantle --bin crunch ...` output with `test result: ok. 1 passed` for each target.
- [x] [serial] Validate Cairn lifecycle artifacts and archive the change only after implementation evidence is recorded. r[verification_evidence.release_source_archive_native_parity]
  - Evidence: `evidence/source-archive-native-parity-validation.md` records `cairn validate`, proposal gate, design gate, and tasks gate all passing before archive.
