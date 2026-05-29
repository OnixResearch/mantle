# Manifest parser evidence

Task-ID: I1
Covers: rust_package_planning.native_manifest_lock_planner

## Baseline

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Pre-change result: failed before edits. Existing `native_manifest_declared_edition_feeds_target_derivation_args` and `native_manifest_workspace_edition_feeds_target_derivation_args` expected `2024` but native target units fell back to `2015`.

## Implementation proof

- Added pure `parse_native_manifest_text(path_label, text)`; `read_native_manifest(...)` is now thin filesystem shell.
- Added positive parser coverage for workspace members, workspace package inheritance, package links, normal/workspace/build dependencies, lib/bin targets, and target cfg dependency tables.
- Added negative parser coverage for invalid package version inheritance.
- Fixed native target unit edition propagation so manifest-derived edition facts feed target rustc derivations when Cargo unit JSON lacks edition.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Result: 8 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_host -- --nocapture`
  - Result: 15 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_graph -- --nocapture`
  - Result: 10 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle build_script_metadata -- --nocapture`
  - Result: 8 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 7`, `specs_validated: 8`.
- `git diff --check`
  - Result: pass.
