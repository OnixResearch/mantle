## Phase 1: Root manifest parsing

- [x] [serial] Add pure parsing of the workspace root manifest `[profile]` table with deterministic blockers for unknown setting keys. r[rust_package_planning.profile_root_manifest_authority]
- [x] [parallel] Add a negative test that dependency manifest `[profile]` tables are ignored and recorded as a non-claim. r[rust_package_planning.profile_root_manifest_authority]

## Phase 2: Custom profiles

- [x] [serial] Add bounded `inherits` resolution with cycle rejection and a positive test for a `release-lto` style custom profile. r[rust_package_planning.profile_custom_inheritance]
- [x] [parallel] Add negative tests for missing `inherits`, inheritance cycles, and unknown setting keys. r[rust_package_planning.profile_custom_inheritance]

## Phase 3: Overrides

- [x] [serial] Implement the first-match precedence ladder for named-package, `"*"`, and build-override tables, with `"*"` excluded from workspace members. r[rust_package_planning.profile_package_overrides]
- [x] [parallel] Add negative tests for forbidden `panic`, `lto`, and `rpath` override settings and for version-qualified package specs. r[rust_package_planning.profile_package_overrides]

## Phase 4: Selection and integration

- [x] [serial] Implement command default selection and `--release` equivalence, and route `src/cargo_import.rs` through the same resolver. r[rust_package_planning.profile_selection]
- [x] [serial] Add negative tests for unresolvable profile names on all entry points. r[rust_package_planning.profile_selection]

## Phase 5: Verification

- [x] [serial] Run focused `cargo test` for profile parsing, resolution, and selection, then rerun one oracle comparison where Cargo profile material exists and record any new mismatches. r[rust_package_planning.profile_custom_inheritance]
  - Evidence: `evidence/verification.md` records the baseline, final focused suites, Clippy, direct formatting checks, and the matched custom-profile Cargo-oracle receipt.
