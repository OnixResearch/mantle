# Tasks

## Phase 1: Implementation

- [x] [serial] Add explicit deterministic rust-plan path mode for cargo-free provider fixed-point stages. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `src/cargo_free_self_build.rs` now passes `--deterministic-release-paths`; `src/rust_plan.rs` records `deterministic_release_paths` and path remaps in receipts.
- [x] [serial] Normalize provider compile-time helper paths without breaking build-script filesystem execution. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env` and `rust_plan::tests::custom_build_manifest_dir_stays_real_in_deterministic_mode` passed in `evidence/provider-fixed-point-path-normalization-validation.md`; provider proof rerun in `evidence/provider-fixed-point-path-normalization-proof.md` confirms aws-lc-sys build-script execution from the real package root.

## Phase 2: Validation

- [x] [serial] Run focused rust-plan/cargo-free tests and Cairn validation/gates. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/provider-fixed-point-path-normalization-validation.md` records `cargo fmt -p mantle --check`, `cargo test -p mantle --bin mantle rust_plan`, `cargo test -p mantle --bin mantle cargo_free`, `cairn validate`, and proposal/design/tasks gates all passing.
- [x] [serial] Regenerate provider/release/witness replay evidence from the corrected source snapshot. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/provider-fixed-point-path-normalization-proof.md` records pueue task 321, proof bundle `/tmp/mantle-provider-fixed-point-4731da6c-fix4`, `fixed_point: true`, and matching stage BLAKE3 `fcf2c1329d6976a1d3377c04f20df9f3540cbcf4463a6adbdcf03f15c3e4a408`.
