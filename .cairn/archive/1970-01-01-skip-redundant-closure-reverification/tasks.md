## Phase 1: Lookup-only store access

- [x] [serial] T1.1 Add `StoreHandle::cached_path_info` with the completeness and PathInfo identity checks but no physical export and no built-output record, and refactor `export_cached_path_info` onto it. r[foreign_derivation_import.hydrated_input_verification_once]
  - Evidence: `crates/crunch-store/src/handle.rs` unit test for a complete local path and an unknown path.
- [x] [serial] T1.2 Use `cached_path_info` in the hydration member walk and in `locally_present_input_paths`, keeping `export_cached_path_info` for selected roots, the cache-only route, and route v1. r[foreign_derivation_import.hydrated_input_verification_once]
  - Evidence: unchanged `tests/cache_inputs_build_route.rs` realization results.

## Phase 2: Verification and records

- [x] [serial] T2.1 Run `-p crunch-store --lib`, the route CLI suites, `-p mantle --bin mantle -- foreign_`, fmt, and strict Clippy; record the exact transcripts in `evidence/`. r[foreign_derivation_import.hydrated_input_verification_once]
  - Evidence: `evidence/checks.md` indexing the captured logs.
- [x] [serial] T2.2 Run `cairn validate`, the proposal/design/tasks gates, and `git diff --check`; commit the change separately. r[foreign_derivation_import.hydrated_input_verification_once]
  - Evidence: gate transcripts under `evidence/`.
