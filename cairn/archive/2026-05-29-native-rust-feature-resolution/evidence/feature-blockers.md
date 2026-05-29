# Native feature blocker evidence

Task-ID: I5
Covers: rust_package_planning.native_feature_resolution

## Implementation proof

- Feature resolver emits deterministic blockers for malformed feature entries and unknown feature references.
- Resolver has a fixed step bound (`NATIVE_FEATURE_RESOLUTION_MAX_STEPS`) that emits `feature-resolution-step-limit` rather than looping unbounded.
- Added negative optional dependency coverage proving absent feature selection leaves optional dependencies unselected.
- Added malformed edge coverage for empty `dep:` and malformed `dependency/` entries.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_feature -- --nocapture`
  - Result: 5 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_derivation_emits -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle oracle_comparison -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_host -- --nocapture`
  - Result: 15 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 6`, `specs_validated: 7`.
- `git diff --check`
  - Result: pass.
