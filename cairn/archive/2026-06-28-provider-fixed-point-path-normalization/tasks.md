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
  - Evidence: `evidence/provider-fixed-point-path-normalization-proof.md` records the final 2026-06-28 provider C-toolchain remap/order proof, provider bundle `/home/brittonr/.cargo-target/repo-targets/mantle/provider-fixed-point-provider-remap-order-fix`, `fixed_point: true`, matching provider stage BLAKE3 `aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20`, successful witness replay pueue task 88, and final `quorum-satisfied` release verification.
