# Lockfile parser evidence

Task-ID: I2
Covers: rust_package_planning.native_manifest_lock_planner

## Implementation proof

- Added pure `parse_native_lockfile_text(path_label, text)` over Cargo.lock text.
- Added normalized lockfile facts for package name/version/source/checksum, git revision extraction, and dependency lock edges.
- Kept `parse_lockfile_packages(...)` as filesystem shell that reads `Cargo.lock` and projects pure lock facts into the legacy `LockPackage` view.
- Added positive parser coverage for path, registry, and git package records plus dependency edges.
- Added negative parser coverage for malformed package records.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_lockfile -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Result: 8 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_registry_source -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_git -- --nocapture`
  - Result: 5 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 7`, `specs_validated: 8`.
- `git diff --check`
  - Result: pass.
