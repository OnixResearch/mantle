# Native feature resolver core evidence

Task-ID: I1-I2
Covers: rust_package_planning.native_feature_resolution

## Implementation proof

- Added pure `NativeFeatureResolutionRequest` / `NativeFeatureResolution` model.
- Added bounded fixed-point resolver over deterministic queues and sorted sets.
- Resolver covers default features, explicit features, `--all-features`, transitive local feature edges, `dep:<name>` optional dependency activation, implicit optional dependency features, and dependency feature edge facts (`dep/feature`, `dep?/feature`).
- `native_selected_features(...)` now derives native package selected features through the resolver instead of one-level feature selection.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_feature -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Result: 12 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_graph -- --nocapture`
  - Result: 10 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_host -- --nocapture`
  - Result: 15 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 6`, `specs_validated: 7`.
- `git diff --check`
  - Result: pass.
