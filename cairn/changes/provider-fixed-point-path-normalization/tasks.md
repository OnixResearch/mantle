# Tasks

## Phase 1: Implementation

- [x] [serial] Add explicit deterministic rust-plan path mode for cargo-free provider fixed-point stages. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `src/cargo_free_self_build.rs` now passes `--deterministic-release-paths`; `src/rust_plan.rs` records `deterministic_release_paths` and path remaps in receipts.
- [x] [serial] Normalize provider compile-time helper paths without breaking build-script filesystem execution. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env` and `rust_plan::tests::build_script_package_root_falls_back_to_real_source_when_manifest_dir_is_virtual` passed in `evidence/provider-fixed-point-path-normalization-validation.md`.

## Phase 2: Validation

- [x] [serial] Run focused rust-plan/cargo-free tests and Cairn validation/gates. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/provider-fixed-point-path-normalization-validation.md` records `cargo fmt -p mantle --check`, `cargo test -p mantle --bin mantle rust_plan`, `cargo test -p mantle --bin mantle cargo_free`, `cairn validate`, and proposal/design/tasks gates all passing.
- [ ] [serial] Regenerate provider/release/witness replay evidence from the corrected source snapshot. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/provider-fixed-point-path-normalization-proof.md`.
