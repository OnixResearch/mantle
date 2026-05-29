# Native feature role separation evidence

Task-ID: I3
Covers: rust_package_planning.native_feature_resolution

## Implementation proof

- Added pure `NativeFeatureRoleResolutionRequest` and `NativeFeatureRoleResolution`.
- Normal, build, and host feature requests resolve independently through the same fixed-point core.
- Added focused test proving normal default features, build optional dependency activation, and host proc-macro dependency feature edges do not leak across roles.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_feature -- --nocapture`
  - Result: 3 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_graph -- --nocapture`
  - Result: 10 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 6`, `specs_validated: 7`.
- `git diff --check`
  - Result: pass.
