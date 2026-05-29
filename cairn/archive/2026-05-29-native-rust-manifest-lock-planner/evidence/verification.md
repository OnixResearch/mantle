# Native manifest/lock planner verification

Task-ID: V1
Covers: rust_package_planning.native_manifest_lock_planner

## Verification commands

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle oracle_comparison -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle unsupported_blockers -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle collect_native_manifest_lock_texts -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_lockfile -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Result: 12 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_registry_source -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_git -- --nocapture`
  - Result: 5 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 7`, `specs_validated: 8`.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks native-rust-manifest-lock-planner --root .`
  - Result: `verdict: PASS`.
- `git diff --check`
  - Result: pass.

## Coverage summary

- Positive manifest parser coverage: workspace/package/target/dependency facts.
- Negative manifest parser coverage: invalid inherited package version.
- Positive lockfile parser coverage: path, registry, git source facts and dependency edges.
- Negative lockfile parser coverage: malformed package record.
- Positive filesystem shell coverage: root/member manifests and lockfile gathered.
- Negative filesystem shell coverage: missing lockfile fails closed.
- Positive unsupported-surface coverage: supported subset produces no blockers.
- Negative unsupported-surface coverage: patch, replace, target build-dependencies, and unsupported lock source produce deterministic blockers.
- Positive oracle coverage: native package facts match Cargo metadata.
- Negative oracle coverage: identity mismatch and missing native package produce deterministic blockers.
