# Tasks

## Spec

- [x] [serial] Add native feature resolution requirement and design. r[rust_package_planning.native_feature_resolution]

## Implementation

- [x] [serial] Add pure feature graph model and fixed-point resolver. r[rust_package_planning.native_feature_resolution]
  - Evidence: `cairn/changes/native-rust-feature-resolution/evidence/feature-resolver-core.md` records pure model, bounded fixed-point resolver, and focused tests.
- [x] [serial] Model default features, explicit features, optional dependency features, and dependency feature edges. r[rust_package_planning.native_feature_resolution]
  - Evidence: `cairn/changes/native-rust-feature-resolution/evidence/feature-resolver-core.md` records default/explicit/transitive optional dependency and dependency feature edge coverage.
- [x] [serial] Separate normal/build/host feature roles for resolver v2 subset. r[rust_package_planning.native_feature_resolution]
  - Evidence: `cairn/changes/native-rust-feature-resolution/evidence/feature-roles.md` records independent normal/build/host feature role resolution and no-leakage tests.
- [x] [serial] Emit selected feature cfgs into native unit derivations. r[rust_package_planning.native_feature_resolution]
  - Evidence: `cairn/changes/native-rust-feature-resolution/evidence/feature-cfg-emission.md` records resolved feature-closure rustc cfg emission coverage.
- [ ] [serial] Add fail-closed blockers for unsupported feature surfaces. r[rust_package_planning.native_feature_resolution]

## Verification

- [ ] [serial] Run resolver unit tests, optional dependency negative tests, oracle comparison fixtures, and Cairn validation. r[rust_package_planning.native_feature_resolution]
