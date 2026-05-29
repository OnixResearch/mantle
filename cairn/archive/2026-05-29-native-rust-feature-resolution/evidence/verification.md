# Native feature resolution verification

Task-ID: V1
Covers: rust_package_planning.native_feature_resolution

## Verification commands

- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo fmt -p mantle -- --check`
  - Result: pass.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_feature -- --nocapture`
  - Result: 5 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_derivation_emits -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_unit_graph -- --nocapture`
  - Result: 10 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_manifest -- --nocapture`
  - Result: 12 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle oracle_comparison -- --nocapture`
  - Result: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=/home/brittonr/.cargo-target cargo test -p mantle --bin mantle native_host -- --nocapture`
  - Result: 15 passed, 0 failed.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid: true`, `changes: 6`, `specs_validated: 7`.
- `git diff --check`
  - Result: pass.

## Coverage summary

- Positive resolver coverage: default, explicit, transitive local feature edges, optional dependency activation, dependency feature edge facts, role-separated normal/build/host resolution, and rustc cfg emission.
- Negative resolver coverage: optional dependency remains inactive without feature selection; malformed feature entries emit deterministic blockers; unknown feature entries emit deterministic blockers.
- Oracle comparison coverage: existing native/Cargo metadata match and mismatch fixtures remain passing after resolver changes.
