# Focused validation for Git source witness replay

Date: 2026-06-29
Change: `git-source-witness-replay`

Environment:

```text
PATH includes nightly rustup, clang-wrapper, mold, pkg-config-wrapper.
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

## cargo fmt --check -p mantle -p crunch-release-core

```text
```

Exit status: 0

## cargo check -p crunch-release-core --target wasm32-unknown-unknown

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
```

Exit status: 0

## cargo test -p crunch-release-core manifest

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling cc v1.2.59
   Compiling serde_json v1.0.149
   Compiling blake3 v1.8.2
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-release-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.75s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 26 tests
test manifest::tests::extract_full_proof_identity_fields_rejects_wrong_schema ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_accepts_matching_binary ... ok
test manifest::tests::canonical_bytes_are_stable_and_compact ... ok
test manifest::tests::extract_full_proof_identity_fields_accepts_valid_manifest ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_missing_provider_kind ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_malformed_digest ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_mismatch ... ok
test manifest::tests::release_manifest_rejects_unknown_selected_provider_kind ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_missing_binaries ... ok
test manifest::tests::validate_accepts_deterministic_proof_artifacts ... ok
test manifest::tests::validate_rejects_absolute_member_path ... ok
test manifest::tests::validate_rejects_credential_bearing_git_source_url ... ok
test manifest::tests::validate_accepts_matching_external_source_acquisition ... ok
test manifest::tests::validate_accepts_git_source_acquisition ... ok
test manifest::tests::validate_accepts_provider_fixed_point_proof_artifact ... ok
test manifest::tests::validate_rejects_deterministic_proof_duplicate_path ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_path ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_git_source_archive_digest_mismatch ... ok
test manifest::tests::validate_rejects_git_source_with_malformed_commit ... ok
test manifest::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_file_kind ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_source_acquisition_digest_mismatch ... ok
test manifest::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test manifest::tests::validate_rejects_unsupported_source_acquisition_url ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p crunch-release-core source_archive

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 6 tests
test source_archive::tests::submodules_are_rejected ... ok
test source_archive::tests::symlink_targets_must_stay_relative ... ok
test source_archive::tests::private_runtime_and_lifecycle_paths_are_excluded ... ok
test manifest::tests::validate_rejects_git_source_archive_digest_mismatch ... ok
test source_archive::tests::unordered_entries_produce_stable_member_order ... ok
test source_archive::tests::unsafe_paths_are_rejected ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_evidence::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 18 tests
test release_evidence::tests::canonical_bytes_are_stable_and_compact ... ok
test release_evidence::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test release_evidence::tests::validate_rejects_absolute_member_path ... ok
test release_evidence::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_accepts_full_proof_bundle ... ok
test release_evidence::tests::create_rejects_conflicting_source_acquisition_modes_before_manifest ... ok
test release_evidence::tests::create_rejects_empty_source_acquisition_url_before_manifest ... ok
test release_evidence::tests::create_rejects_empty_git_source_commit_before_manifest ... ok
test release_evidence::tests::create_rejects_invalid_provider_fixed_point_proof_before_manifest ... ok
test release_evidence::tests::create_rejects_provider_fixed_point_proof_for_different_binary ... ok
test release_evidence::tests::verify_rejects_non_canonical_manifest_json ... ok
test release_evidence::tests::verify_rejects_tampered_binary_artifact ... ok
test release_evidence::tests::create_and_verify_release_bundle_records_git_source_acquisition ... ok
test release_evidence::tests::create_and_verify_release_bundle_records_source_acquisition_url ... ok
test release_evidence::tests::verify_rejects_provider_kind_linkage_mismatch ... ok
test release_evidence::tests::create_and_verify_release_bundle_round_trip ... ok
test release_evidence::tests::create_and_verify_release_bundle_with_provider_fixed_point_proof ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 905 filtered out; finished in 0.02s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_source::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 7 tests
test release_source::tests::source_archive_rejects_git_submodule_entries ... ok
test release_source::tests::source_archive_rejects_symlink_targets_outside_tree ... ok
test release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files ... ok
test release_source::tests::git_source_archive_rejects_wrong_commit_before_archiving ... ok
test release_source::tests::git_source_archive_rejects_missing_ref_policy ... ok
test release_source::tests::git_source_archive_reconstruction_matches_release_source_archive ... ok
test release_source::tests::git_source_archive_rejects_ref_policy_mismatch ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 916 filtered out; finished in 0.03s

```

Exit status: 0

## cargo test -p mantle --bin crunch witness_rebuild::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 35 tests
test witness_rebuild::tests::bootstrap_divergence_diagnostic_reports_convergence ... ok
test witness_rebuild::tests::default_witness_scratch_dir_appends_work_suffix ... ok
test witness_rebuild::tests::launched_workflow_command_records_proof_mode_arguments ... ok
test witness_rebuild::tests::parse_request_relative_path_rejects_parent_components ... ok
test witness_rebuild::tests::resolve_proof_bundle_artifact_path_anchors_relative_paths ... ok
test witness_rebuild::tests::resolve_proof_bundle_artifact_path_rejects_parent_escape ... ok
test witness_rebuild::tests::source_acquisition_for_plan_accepts_metadata_when_flagged ... ok
test witness_rebuild::tests::source_acquisition_for_plan_requires_git_kind_when_flagged ... ok
test witness_rebuild::tests::source_acquisition_for_plan_requires_metadata_when_flagged ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_absolute_proof_binary_path ... ok
test witness_rebuild::tests::validate_source_replay_flags_rejects_conflicting_strict_modes ... ok
test witness_rebuild::tests::workflow_args_preserve_non_nix_proof_mode_and_reject_unknown_modes ... ok
test witness_rebuild::tests::validate_supported_workflow_identity_rejects_unknown_pair ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_escaping_proof_binary_path ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_missing_expected_digest ... ok
test witness_rebuild::tests::git_source_success_audit_records_derivation_details ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_invalid_provider_fixed_point_proof ... ok
test witness_rebuild::tests::failure_audit_reports_digest_mismatch_diagnostics ... ok
test witness_rebuild::tests::failure_audit_reports_gcc_bootstrap_divergence_root ... ok
test witness_rebuild::tests::source_acquisition_prelaunch_failure_audit_records_error ... ok
test witness_rebuild::tests::source_acquisition_success_audit_records_verified_fetch ... ok
test witness_rebuild::tests::validate_existing_scratch_root_allows_helper_owned_dirs_only ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlink_root ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlinked_helper_owned_tmp_entry ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_unexpected_entries ... ok
test witness_rebuild::tests::validate_rebuilt_output_digests_rejects_stripped_equivalent_but_different_binary ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_file_helper_owned_cargo_target_entry ... ok
test witness_rebuild::tests::source_acquisition_rejects_digest_mismatch_before_extraction ... ok
test witness_rebuild::tests::prepare_scratch_fetches_independent_source_archive_when_required ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_preserves_one_output_legacy_manifest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_maps_two_expected_outputs_by_digest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest ... ok
test witness_rebuild::tests::git_source_acquisition_rejects_digest_mismatch_before_extraction ... ok
test witness_rebuild::tests::prepare_scratch_fetches_git_source_archive_when_required ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 888 filtered out; finished in 0.03s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_create_accepts_git_source_flags

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 1 test
test tests::release_create_accepts_git_source_flags ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 922 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_witness_rebuild_accepts_require_git_source_flag

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 1 test
test tests::release_witness_rebuild_accepts_require_git_source_flag ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 922 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_witness_rebuild_rejects_conflicting_source_replay_modes

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 1 test
test tests::release_witness_rebuild_rejects_conflicting_source_replay_modes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 922 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p mantle --test release_cli release_create_records_git_source_metadata

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.59s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 1 test
test release_create_records_git_source_metadata ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.05s

```

Exit status: 0

## cargo test -p mantle --test release_cli release_create_rejects_git_source_url_without_commit

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 1 test
test release_create_rejects_git_source_url_without_commit ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.02s

```

Exit status: 0

## cargo test -p mantle --test release_cli release_create_rejects_conflicting_source_acquisition_flags

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 1 test
test release_create_rejects_conflicting_source_acquisition_flags ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.02s

```

Exit status: 0
