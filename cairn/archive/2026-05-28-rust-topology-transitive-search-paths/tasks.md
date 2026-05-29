# Tasks

## Spec

- [x] [serial] Add transitive native rustc search-path requirement and design. r[rust_package_planning.native_transitive_search_paths]

## Implementation

- [x] [serial] Track all produced target-lib/proc-macro artifact paths separately from package-keyed direct binding maps. r[rust_package_planning.native_transitive_search_paths]
- [x] [serial] Append deterministic deduplicated `-L dependency` entries from full produced search-path history. r[rust_package_planning.native_transitive_search_paths]
- [x] [serial] Add focused positive and negative regression tests for duplicate same-package search paths and deduplication. r[rust_package_planning.native_transitive_search_paths]

## Verification

- [x] [serial] Run focused tests, native unit graph tests, dirty self-probe, Cairn validation, and archive readiness checks. Evidence: `evidence/verification.md`. r[rust_package_planning.native_transitive_search_paths]
