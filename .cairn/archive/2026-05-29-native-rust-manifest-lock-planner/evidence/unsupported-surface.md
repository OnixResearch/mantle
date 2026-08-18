# Unsupported surface evidence

Task-ID: I4
Covers: rust_package_planning.native_manifest_lock_planner

## Implementation proof

- Added `NativeManifestLockUnsupportedBlocker` facts for unsupported manifest and lockfile material.
- Added pure `native_manifest_lock_unsupported_blockers(...)` over gathered text inputs.
- Manifest blockers now identify unsupported `[patch]`, `[replace]`, and non-`dependencies` target cfg tables.
- Lockfile blockers now identify malformed lockfile text, unsupported package source prefixes, and malformed empty dependency edges.
- Added positive supported-subset coverage and negative coverage for patch/replace/target build-dependency/unknown lock source cases.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle unsupported_blockers -- --nocapture`
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
