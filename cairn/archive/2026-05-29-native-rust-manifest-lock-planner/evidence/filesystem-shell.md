# Filesystem shell evidence

Task-ID: I3
Covers: rust_package_planning.native_manifest_lock_planner

## Implementation proof

- Added `collect_native_manifest_lock_texts(root)` as thin filesystem shell.
- Shell reads root `Cargo.toml`, expands supported workspace members through the existing bounded member-glob logic, gathers all manifest text inputs, and reads root `Cargo.lock`.
- Pure parsing remains in `parse_native_manifest_text(...)` and `parse_native_lockfile_text(...)`; shell returns raw text inputs for later parser/oracle phases.
- Added positive coverage for root workspace manifest + member manifest + lockfile text collection.
- Added negative coverage for missing lockfile fail-closed behavior.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle collect_native_manifest_lock_texts -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_lockfile -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Result: 10 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_registry_source -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_git -- --nocapture`
  - Result: 5 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 7`, `specs_validated: 8`.
- `git diff --check`
  - Result: pass.
