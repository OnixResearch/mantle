# Tasks

## Spec

- [x] [serial] Add native manifest/lockfile planner requirement and design. r[rust_package_planning.native_manifest_lock_planner]

## Implementation

- [x] [serial] Add pure manifest parser for package/workspace/target/dependency facts. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/manifest-parser.md` records positive/negative pure parser tests plus edition propagation fix and focused validation.
- [x] [serial] Add pure lockfile parser for registry/git/path source facts. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/lockfile-parser.md` records normalized source/checksum/revision/dependency-edge parsing and focused validation.
- [x] [serial] Add filesystem shell that gathers manifests and lockfile text. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/filesystem-shell.md` records root/member manifest and lockfile text collection tests plus fail-closed missing-lockfile coverage.
- [x] [serial] Add fail-closed blockers for unsupported manifest and lockfile surface. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/unsupported-surface.md` records pure blocker coverage for supported input and patch/replace/target-table/unsupported-lock-source failures.
- [x] [serial] Add oracle comparison against Cargo metadata for supported fixtures. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/oracle-comparison.md` records supported-match and mismatch/missing-package oracle comparison fixtures.

## Verification

- [x] [serial] Run positive and negative parser tests, oracle comparison fixtures, and Cairn validation. r[rust_package_planning.native_manifest_lock_planner]
  - Evidence: `cairn/changes/native-rust-manifest-lock-planner/evidence/verification.md` records focused positive/negative parser, blocker, filesystem-shell, oracle comparison, and Cairn validation commands.
