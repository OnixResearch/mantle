# Native feature cfg emission evidence

Task-ID: I4
Covers: rust_package_planning.native_feature_resolution

## Implementation proof

- Native package selected features now flow through the fixed-point resolver via `native_selected_features(...)`.
- Existing native unit derivation cfg emission consumes the resolved selected feature closure.
- Added focused test proving a default feature closure (`default -> derive -> printing`) emits all resolved `--cfg feature="..."` rustc args.

## Verification

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_feature -- --nocapture`
  - Result: 3 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_derivation_emits -- --nocapture`
  - Result: 2 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 6`, `specs_validated: 7`.
- `git diff --check`
  - Result: pass.
