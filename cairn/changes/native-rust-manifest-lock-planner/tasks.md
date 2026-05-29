# Tasks

## Spec

- [x] [serial] Add native manifest/lockfile planner requirement and design. r[rust_package_planning.native_manifest_lock_planner]

## Implementation

- [x] [serial] Add pure manifest parser for package/workspace/target/dependency facts. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/manifest-parser.md` records positive/negative pure parser tests plus edition propagation fix and focused validation.
- [x] [serial] Add pure lockfile parser for registry/git/path source facts. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/lockfile-parser.md` records normalized source/checksum/revision/dependency-edge parsing and focused validation.
- [ ] [serial] Add filesystem shell that gathers manifests and lockfile text. r[rust_package_planning.native_manifest_lock_planner]
- [ ] [serial] Add fail-closed blockers for unsupported manifest and lockfile surface. r[rust_package_planning.native_manifest_lock_planner]
- [ ] [serial] Add oracle comparison against Cargo metadata for supported fixtures. r[rust_package_planning.native_manifest_lock_planner]

## Verification

- [ ] [serial] Run positive and negative parser tests, oracle comparison fixtures, and Cairn validation. r[rust_package_planning.native_manifest_lock_planner]
