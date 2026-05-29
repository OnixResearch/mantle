# Oracle comparison evidence

Task-ID: I5
Covers: rust_package_planning.native_manifest_lock_planner

## Implementation proof

- Added focused Cargo oracle comparison fixture tests around `compare_native_packages_to_cargo(...)`.
- Positive fixture proves a supported native package fact matches Cargo metadata by package ID, manifest path, name, and version.
- Negative fixture proves deterministic blockers for native/Cargo identity mismatch and Cargo-required package missing from native facts.

## Verification

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
