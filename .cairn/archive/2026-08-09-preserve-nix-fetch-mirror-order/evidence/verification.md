# Verification

## Producer unit tests

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/fuse-backend-rs)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-castore)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-store)
   Compiling crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta)
   Compiling crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rustc-wrapper)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 55.56s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 24 tests
test foreign_derivation_import::tests::fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts ... ok
test foreign_derivation_import::tests::fixed_output_hash_mode_rejects_unknown_and_conflicting_facts ... ok
test foreign_derivation_import::tests::nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes ... ok
test foreign_derivation_import::tests::nix_closure_selection_filters_unreachable_and_requires_inputs ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_normalize_direct_unstructured_and_structured_forms ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_reject_invalid_ambiguous_and_private_forms ... ok
test foreign_derivation_import::tests::nix_lowering_emits_canonical_candidates_and_keeps_arbitrary_fixed_outputs_non_downloads ... ok
test foreign_derivation_import::tests::canonical_fetch_candidate_validation_rejects_conflicts_and_private_injection ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_non_object_structured_attributes ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_preserves_structured_attributes_as_protocol_json ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_parses_nix_and_guix_paths ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_lowering_emits_guix_graph_facts ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::nix_aterm_derivation_closure_lowering_preserves_graph_facts ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 2305 filtered out; finished in 0.00s

```

## Compiler unit tests

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 19 tests
test foreign_graph_compiler::tests::conflicting_cross_class_path_mapping_is_rejected ... ok
test foreign_graph_compiler::tests::exact_path_map_reuses_identical_entries_and_rejects_conflicts ... ok
test foreign_graph_compiler::tests::unrelated_or_non_token_store_text_is_not_rewritten ... ok
test foreign_graph_compiler::tests::known_store_dir_placeholders_use_the_active_store_prefix ... ok
test foreign_graph_compiler::tests::dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage ... ok
test foreign_graph_compiler::tests::foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution ... ok
test foreign_graph_compiler::tests::fixed_output_seed_outside_the_selected_root_is_not_required ... ok
test foreign_graph_compiler::tests::fixed_output_seed_without_a_producer_binding_is_rejected ... ok
test foreign_graph_compiler::tests::fixed_output_seed_admits_the_recomputed_output_path ... ok
test foreign_graph_compiler::tests::compiler_maps_source_descriptors_and_rewrites_suffixes ... ok
test foreign_graph_compiler::tests::compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::cache_only_compiler_preserves_exact_paths_and_rejects_prefix_drift ... ok
test foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_rewrites_store_paths_inside_structured_attribute_json ... ok
test foreign_graph_compiler::tests::compiler_preserves_non_path_store_placeholders_but_rejects_valid_unknown_paths ... ok
test foreign_graph_compiler::tests::compiler_binds_canonical_nix_candidates_and_order_into_recipe_identity ... ok
test foreign_graph_compiler::tests::compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths ... ok
test foreign_graph_compiler::tests::dependency_compiler_is_exact_deterministic_and_prefix_sensitive ... ok
test foreign_graph_compiler::tests::execution_profile_changes_target_identity_and_reserved_collision_fails ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2310 filtered out; finished in 0.01s

```

## Task-specified producer lib filter

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.87s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/mantle-d895557f628ed146)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 174 filtered out; finished in 0.00s

```

## Task-specified compiler lib filter

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/mantle-d895557f628ed146)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 174 filtered out; finished in 0.00s

```

## Fetch service tests

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/fuse-backend-rs)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-castore)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-build)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-store)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.23s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-793ece271dec2ecd)

running 23 tests
test fetch_build_service::tests::is_fetch_request_false_for_empty_args ... ok
test fetch_build_service::tests::is_fetch_request_false_for_sandbox ... ok
test fetch_build_service::tests::is_fetch_request_true_for_builtin ... ok
test fetch_build_service::tests::parse_git_fetch ... ok
test fetch_build_service::tests::parse_file_fetch ... ok
test fetch_build_service::tests::parse_executable_fetch ... ok
test fetch_build_service::tests::parse_git_missing_rev ... ok
test fetch_build_service::tests::parse_invalid_url ... ok
test fetch_build_service::tests::parse_missing_url ... ok
test fetch_build_service::tests::parse_rejects_non_fetcher ... ok
test fetch_build_service::tests::ordered_foreign_candidates_preserve_declared_order_and_kind ... ok
test fetch_build_service::tests::ordered_foreign_candidates_reject_malformed_duplicate_and_stale_primary ... ok
test fetch_build_service::tests::parse_unpack_takes_priority_over_executable ... ok
test fetch_build_service::tests::parse_tarball_fetch ... ok
test fetch_build_service::tests::source_override_requires_matching_git_revision ... ok
test fetch_build_service::tests::do_build_rejects_missing_url ... ok
test fetch_build_service::tests::do_build_rejects_non_fetch ... ok
test fetch_build_service::tests::required_source_override_rejects_unmatched_fetch_before_network ... ok
test fetch_build_service::tests::do_build_fetches_local_file ... ok
test fetch_build_service::tests::do_build_executable_sets_mode ... ok
test fetch_build_service::tests::ordered_foreign_candidates_select_first_available_source_state ... ok
test fetch_build_service::tests::do_build_uses_matching_source_override_without_network ... ok
test fetch_build_service::tests::do_build_fetches_tarball ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 641 filtered out; finished in 1.37s

```

## Foreign import CLI tests

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 45.15s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 17 tests
test produce_backend_fix_and_host_nix_emit_parity_artifacts ... ignored, requires MANTLE_TEST_FIX_BACKEND_BINARY, MANTLE_TEST_NIX_INSTANTIATE_BINARY, and a reachable Nix daemon
test produce_backend_rejects_unknown_backend ... ok
test foreign_import_cli_produce_aterm_rejects_input_mode_conflicts_without_artifacts ... ok
test produce_backend_rejects_conflicting_target_modes ... ok
test foreign_import_cli_rejects_malformed_drv_without_partial_artifacts ... ok
test foreign_import_cli_rejects_drv_dir_missing_reachable_input_without_partial_artifacts ... ok
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_produce_aterm_rejects_bad_bundles_without_partial_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_dir_without_nix ... ok
test foreign_import_cli_produce_aterm_matches_nix_and_emits_guix_directory_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_files_without_nix ... ok
test foreign_import_cli_produce_aterm_rejects_duplicate_and_bounded_byte_inputs ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok
two-node build_report_blake3="d43e02fbb71558fa67a938df41558be076a2ec53625795704a83fcc304e27a38"
two-node provenance_audit_blake3="e3175cdbe4028d595d3664e831a18b38bbd5c1afbfa8c6bdb190af506c6f264e"
node-limit provenance_audit_blake3="73da9c07ee54f9e329d33f43989f496cc3b6403ac80555fe87442d70b7ac04a7"
guix-no-bin-sh build_report_blake3="72e6088eb8fd6e2e594c695fd303f99d65d8cf013c652309be66656d07b7daf6"
test foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs ... ok
fixed-output-mismatch build_report_blake3="1746553f31f2609a832b839ce99bd98005b425dc447394b0e2af29cb494e77fd"
test foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state ... ok

test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.64s

```

## Final producer rerun

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 08s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 25 tests
test foreign_derivation_import::tests::fixed_output_hash_mode_rejects_unknown_and_conflicting_facts ... ok
test foreign_derivation_import::tests::fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts ... ok
test foreign_derivation_import::tests::nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes ... ok
test foreign_derivation_import::tests::nix_closure_selection_filters_unreachable_and_requires_inputs ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_normalize_direct_unstructured_and_structured_forms ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_reject_invalid_ambiguous_and_private_forms ... ok
test foreign_derivation_import::tests::nix_lowering_emits_canonical_candidates_and_keeps_arbitrary_fixed_outputs_non_downloads ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs ... ok
test foreign_derivation_import::tests::canonical_fetch_candidate_validation_rejects_conflicts_and_private_injection ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_parses_nix_and_guix_paths ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::nix_aterm_derivation_closure_lowering_preserves_graph_facts ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_lowering_emits_guix_graph_facts ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_non_object_structured_attributes ... ok
test foreign_derivation_import::tests::structured_and_unstructured_nix_candidates_lower_to_equal_node_facts ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_preserves_structured_attributes_as_protocol_json ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 2305 filtered out; finished in 0.01s

```

## Final compiler rerun

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 49.92s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 19 tests
test foreign_graph_compiler::tests::conflicting_cross_class_path_mapping_is_rejected ... ok
test foreign_graph_compiler::tests::exact_path_map_reuses_identical_entries_and_rejects_conflicts ... ok
test foreign_graph_compiler::tests::unrelated_or_non_token_store_text_is_not_rewritten ... ok
test foreign_graph_compiler::tests::known_store_dir_placeholders_use_the_active_store_prefix ... ok
test foreign_graph_compiler::tests::dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage ... ok
test foreign_graph_compiler::tests::foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution ... ok
test foreign_graph_compiler::tests::fixed_output_seed_outside_the_selected_root_is_not_required ... ok
test foreign_graph_compiler::tests::fixed_output_seed_without_a_producer_binding_is_rejected ... ok
test foreign_graph_compiler::tests::compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_rewrites_store_paths_inside_structured_attribute_json ... ok
test foreign_graph_compiler::tests::cache_only_compiler_preserves_exact_paths_and_rejects_prefix_drift ... ok
test foreign_graph_compiler::tests::fixed_output_seed_admits_the_recomputed_output_path ... ok
test foreign_graph_compiler::tests::compiler_maps_source_descriptors_and_rewrites_suffixes ... ok
test foreign_graph_compiler::tests::compiler_preserves_non_path_store_placeholders_but_rejects_valid_unknown_paths ... ok
test foreign_graph_compiler::tests::compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths ... ok
test foreign_graph_compiler::tests::compiler_binds_canonical_nix_candidates_and_order_into_recipe_identity ... ok
test foreign_graph_compiler::tests::execution_profile_changes_target_identity_and_reserved_collision_fails ... ok
test foreign_graph_compiler::tests::dependency_compiler_is_exact_deterministic_and_prefix_sensitive ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2313 filtered out; finished in 0.00s

```

## Final fetch-service rerun

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.19s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-793ece271dec2ecd)

running 24 tests
test fetch_build_service::tests::is_fetch_request_false_for_empty_args ... ok
test fetch_build_service::tests::is_fetch_request_false_for_sandbox ... ok
test fetch_build_service::tests::is_fetch_request_true_for_builtin ... ok
test fetch_build_service::tests::parse_git_fetch ... ok
test fetch_build_service::tests::parse_file_fetch ... ok
test fetch_build_service::tests::parse_executable_fetch ... ok
test fetch_build_service::tests::parse_git_missing_rev ... ok
test fetch_build_service::tests::parse_invalid_url ... ok
test fetch_build_service::tests::parse_missing_url ... ok
test fetch_build_service::tests::ordered_foreign_candidates_preserve_declared_order_and_kind ... ok
test fetch_build_service::tests::ordered_foreign_candidates_reject_malformed_duplicate_and_stale_primary ... ok
test fetch_build_service::tests::parse_rejects_non_fetcher ... ok
test fetch_build_service::tests::parse_tarball_fetch ... ok
test fetch_build_service::tests::parse_unpack_takes_priority_over_executable ... ok
test fetch_build_service::tests::source_override_requires_matching_git_revision ... ok
test fetch_build_service::tests::do_build_rejects_missing_url ... ok
test fetch_build_service::tests::do_build_rejects_non_fetch ... ok
test fetch_build_service::tests::ordered_foreign_candidates_report_exhaustion_before_network ... ok
test fetch_build_service::tests::required_source_override_rejects_unmatched_fetch_before_network ... ok
test fetch_build_service::tests::do_build_fetches_local_file ... ok
test fetch_build_service::tests::do_build_executable_sets_mode ... ok
test fetch_build_service::tests::do_build_uses_matching_source_override_without_network ... ok
test fetch_build_service::tests::ordered_foreign_candidates_select_first_available_source_state ... ok
test fetch_build_service::tests::do_build_fetches_tarball ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 641 filtered out; finished in 0.29s

```

## Final CLI rerun

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 45.67s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 17 tests
test produce_backend_fix_and_host_nix_emit_parity_artifacts ... ignored, requires MANTLE_TEST_FIX_BACKEND_BINARY, MANTLE_TEST_NIX_INSTANTIATE_BINARY, and a reachable Nix daemon
test produce_backend_rejects_unknown_backend ... ok
test foreign_import_cli_rejects_malformed_drv_without_partial_artifacts ... ok
test foreign_import_cli_produce_aterm_rejects_input_mode_conflicts_without_artifacts ... ok
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_produce_aterm_matches_nix_and_emits_guix_directory_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_files_without_nix ... ok
test foreign_import_cli_produce_aterm_rejects_bad_bundles_without_partial_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_produce_aterm_rejects_duplicate_and_bounded_byte_inputs ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok
two-node build_report_blake3="d43e02fbb71558fa67a938df41558be076a2ec53625795704a83fcc304e27a38"
two-node provenance_audit_blake3="e3175cdbe4028d595d3664e831a18b38bbd5c1afbfa8c6bdb190af506c6f264e"
node-limit provenance_audit_blake3="73da9c07ee54f9e329d33f43989f496cc3b6403ac80555fe87442d70b7ac04a7"
test produce_backend_rejects_conflicting_target_modes ... ok
test foreign_import_cli_rejects_drv_dir_missing_reachable_input_without_partial_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_dir_without_nix ... ok
fixed-output-mismatch build_report_blake3="1746553f31f2609a832b839ce99bd98005b425dc447394b0e2af29cb494e77fd"
test foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state ... ok
guix-no-bin-sh build_report_blake3="72e6088eb8fd6e2e594c695fd303f99d65d8cf013c652309be66656d07b7daf6"
test foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs ... ok

test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.63s

```

## Stable final rerun after complete edits

### Producer

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 49.13s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 27 tests
test foreign_derivation_import::tests::fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts ... ok
test foreign_derivation_import::tests::fixed_output_hash_mode_rejects_unknown_and_conflicting_facts ... ok
test foreign_derivation_import::tests::nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes ... ok
test foreign_derivation_import::tests::nix_closure_selection_filters_unreachable_and_requires_inputs ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_normalize_direct_unstructured_and_structured_forms ... ok
test foreign_derivation_import::tests::canonical_fetch_graph_model_rejects_stale_legacy_mirrors ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_reject_invalid_ambiguous_and_private_forms ... ok
test foreign_derivation_import::tests::canonical_fetch_candidate_validation_rejects_conflicts_and_private_injection ... ok
test foreign_derivation_import::tests::nix_lowering_emits_canonical_candidates_and_keeps_arbitrary_fixed_outputs_non_downloads ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::structured_and_unstructured_nix_candidates_lower_to_equal_node_facts ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_non_object_structured_attributes ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_lowering_emits_guix_graph_facts ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_parses_nix_and_guix_paths ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_preserves_structured_attributes_as_protocol_json ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::nix_aterm_derivation_closure_lowering_preserves_graph_facts ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::canonical_fetch_graph_model_preserves_legacy_decode_and_rejects_invalid_lists ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 2305 filtered out; finished in 0.00s

```

### Compiler

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 47.78s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 19 tests
test foreign_graph_compiler::tests::exact_path_map_reuses_identical_entries_and_rejects_conflicts ... ok
test foreign_graph_compiler::tests::conflicting_cross_class_path_mapping_is_rejected ... ok
test foreign_graph_compiler::tests::unrelated_or_non_token_store_text_is_not_rewritten ... ok
test foreign_graph_compiler::tests::known_store_dir_placeholders_use_the_active_store_prefix ... ok
test foreign_graph_compiler::tests::dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage ... ok
test foreign_graph_compiler::tests::foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution ... ok
test foreign_graph_compiler::tests::fixed_output_seed_outside_the_selected_root_is_not_required ... ok
test foreign_graph_compiler::tests::fixed_output_seed_without_a_producer_binding_is_rejected ... ok
test foreign_graph_compiler::tests::fixed_output_seed_admits_the_recomputed_output_path ... ok
test foreign_graph_compiler::tests::compiler_maps_source_descriptors_and_rewrites_suffixes ... ok
test foreign_graph_compiler::tests::compiler_rewrites_store_paths_inside_structured_attribute_json ... ok
test foreign_graph_compiler::tests::compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::cache_only_compiler_preserves_exact_paths_and_rejects_prefix_drift ... ok
test foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_preserves_non_path_store_placeholders_but_rejects_valid_unknown_paths ... ok
test foreign_graph_compiler::tests::compiler_binds_canonical_nix_candidates_and_order_into_recipe_identity ... ok
test foreign_graph_compiler::tests::compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths ... ok
test foreign_graph_compiler::tests::execution_profile_changes_target_identity_and_reserved_collision_fails ... ok
test foreign_graph_compiler::tests::dependency_compiler_is_exact_deterministic_and_prefix_sensitive ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2313 filtered out; finished in 0.00s

```

### Fetch service

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-793ece271dec2ecd)

running 24 tests
test fetch_build_service::tests::is_fetch_request_false_for_sandbox ... ok
test fetch_build_service::tests::is_fetch_request_false_for_empty_args ... ok
test fetch_build_service::tests::is_fetch_request_true_for_builtin ... ok
test fetch_build_service::tests::parse_executable_fetch ... ok
test fetch_build_service::tests::parse_git_missing_rev ... ok
test fetch_build_service::tests::parse_git_fetch ... ok
test fetch_build_service::tests::parse_file_fetch ... ok
test fetch_build_service::tests::ordered_foreign_candidates_preserve_declared_order_and_kind ... ok
test fetch_build_service::tests::ordered_foreign_candidates_reject_malformed_duplicate_and_stale_primary ... ok
test fetch_build_service::tests::parse_missing_url ... ok
test fetch_build_service::tests::parse_invalid_url ... ok
test fetch_build_service::tests::parse_rejects_non_fetcher ... ok
test fetch_build_service::tests::parse_tarball_fetch ... ok
test fetch_build_service::tests::parse_unpack_takes_priority_over_executable ... ok
test fetch_build_service::tests::source_override_requires_matching_git_revision ... ok
test fetch_build_service::tests::do_build_rejects_non_fetch ... ok
test fetch_build_service::tests::do_build_executable_sets_mode ... ok
test fetch_build_service::tests::required_source_override_rejects_unmatched_fetch_before_network ... ok
test fetch_build_service::tests::ordered_foreign_candidates_report_exhaustion_before_network ... ok
test fetch_build_service::tests::do_build_fetches_local_file ... ok
test fetch_build_service::tests::do_build_rejects_missing_url ... ok
test fetch_build_service::tests::ordered_foreign_candidates_select_first_available_source_state ... ok
test fetch_build_service::tests::do_build_uses_matching_source_override_without_network ... ok
test fetch_build_service::tests::do_build_fetches_tarball ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 641 filtered out; finished in 0.45s

```

### CLI

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 46.26s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 17 tests
test produce_backend_fix_and_host_nix_emit_parity_artifacts ... ignored, requires MANTLE_TEST_FIX_BACKEND_BINARY, MANTLE_TEST_NIX_INSTANTIATE_BINARY, and a reachable Nix daemon
test produce_backend_rejects_conflicting_target_modes ... ok
test produce_backend_rejects_unknown_backend ... ok
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_rejects_drv_dir_missing_reachable_input_without_partial_artifacts ... ok
test foreign_import_cli_produce_aterm_rejects_input_mode_conflicts_without_artifacts ... ok
test foreign_import_cli_rejects_malformed_drv_without_partial_artifacts ... ok
test foreign_import_cli_produce_aterm_rejects_bad_bundles_without_partial_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_files_without_nix ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_produce_aterm_rejects_duplicate_and_bounded_byte_inputs ... ok
test foreign_import_cli_produce_aterm_matches_nix_and_emits_guix_directory_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_dir_without_nix ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok
two-node build_report_blake3="d43e02fbb71558fa67a938df41558be076a2ec53625795704a83fcc304e27a38"
two-node provenance_audit_blake3="e3175cdbe4028d595d3664e831a18b38bbd5c1afbfa8c6bdb190af506c6f264e"
fixed-output-mismatch build_report_blake3="1746553f31f2609a832b839ce99bd98005b425dc447394b0e2af29cb494e77fd"
test foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state ... ok
node-limit provenance_audit_blake3="73da9c07ee54f9e329d33f43989f496cc3b6403ac80555fe87442d70b7ac04a7"
guix-no-bin-sh build_report_blake3="72e6088eb8fd6e2e594c695fd303f99d65d8cf013c652309be66656d07b7daf6"
test foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs ... ok

test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.66s

```

## Definitive final rerun

### Producer

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 27 tests
test foreign_derivation_import::tests::fixed_output_hash_mode_rejects_unknown_and_conflicting_facts ... ok
test foreign_derivation_import::tests::fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts ... ok
test foreign_derivation_import::tests::nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes ... ok
test foreign_derivation_import::tests::nix_closure_selection_filters_unreachable_and_requires_inputs ... ok
test foreign_derivation_import::tests::canonical_fetch_graph_model_rejects_stale_legacy_mirrors ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_normalize_direct_unstructured_and_structured_forms ... ok
test foreign_derivation_import::tests::nix_lowering_emits_canonical_candidates_and_keeps_arbitrary_fixed_outputs_non_downloads ... ok
test foreign_derivation_import::tests::canonical_fetch_candidate_validation_rejects_conflicts_and_private_injection ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_reject_invalid_ambiguous_and_private_forms ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs ... ok
test foreign_derivation_import::tests::structured_and_unstructured_nix_candidates_lower_to_equal_node_facts ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_non_object_structured_attributes ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_parses_nix_and_guix_paths ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_lowering_emits_guix_graph_facts ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_preserves_structured_attributes_as_protocol_json ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::canonical_fetch_graph_model_preserves_legacy_decode_and_rejects_invalid_lists ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::nix_aterm_derivation_closure_lowering_preserves_graph_facts ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 2305 filtered out; finished in 0.00s

```

### Compiler

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 19 tests
test foreign_graph_compiler::tests::exact_path_map_reuses_identical_entries_and_rejects_conflicts ... ok
test foreign_graph_compiler::tests::conflicting_cross_class_path_mapping_is_rejected ... ok
test foreign_graph_compiler::tests::known_store_dir_placeholders_use_the_active_store_prefix ... ok
test foreign_graph_compiler::tests::unrelated_or_non_token_store_text_is_not_rewritten ... ok
test foreign_graph_compiler::tests::dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage ... ok
test foreign_graph_compiler::tests::foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution ... ok
test foreign_graph_compiler::tests::fixed_output_seed_outside_the_selected_root_is_not_required ... ok
test foreign_graph_compiler::tests::fixed_output_seed_admits_the_recomputed_output_path ... ok
test foreign_graph_compiler::tests::compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_rewrites_store_paths_inside_structured_attribute_json ... ok
test foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::fixed_output_seed_without_a_producer_binding_is_rejected ... ok
test foreign_graph_compiler::tests::compiler_maps_source_descriptors_and_rewrites_suffixes ... ok
test foreign_graph_compiler::tests::cache_only_compiler_preserves_exact_paths_and_rejects_prefix_drift ... ok
test foreign_graph_compiler::tests::compiler_preserves_non_path_store_placeholders_but_rejects_valid_unknown_paths ... ok
test foreign_graph_compiler::tests::compiler_binds_canonical_nix_candidates_and_order_into_recipe_identity ... ok
test foreign_graph_compiler::tests::execution_profile_changes_target_identity_and_reserved_collision_fails ... ok
test foreign_graph_compiler::tests::compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths ... ok
test foreign_graph_compiler::tests::dependency_compiler_is_exact_deterministic_and_prefix_sensitive ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2313 filtered out; finished in 0.01s

```

### Fetch service

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-793ece271dec2ecd)

running 24 tests
test fetch_build_service::tests::is_fetch_request_false_for_empty_args ... ok
test fetch_build_service::tests::is_fetch_request_false_for_sandbox ... ok
test fetch_build_service::tests::is_fetch_request_true_for_builtin ... ok
test fetch_build_service::tests::parse_executable_fetch ... ok
test fetch_build_service::tests::parse_file_fetch ... ok
test fetch_build_service::tests::parse_git_fetch ... ok
test fetch_build_service::tests::ordered_foreign_candidates_preserve_declared_order_and_kind ... ok
test fetch_build_service::tests::parse_git_missing_rev ... ok
test fetch_build_service::tests::ordered_foreign_candidates_reject_malformed_duplicate_and_stale_primary ... ok
test fetch_build_service::tests::parse_invalid_url ... ok
test fetch_build_service::tests::parse_missing_url ... ok
test fetch_build_service::tests::parse_rejects_non_fetcher ... ok
test fetch_build_service::tests::parse_tarball_fetch ... ok
test fetch_build_service::tests::parse_unpack_takes_priority_over_executable ... ok
test fetch_build_service::tests::source_override_requires_matching_git_revision ... ok
test fetch_build_service::tests::do_build_rejects_missing_url ... ok
test fetch_build_service::tests::do_build_rejects_non_fetch ... ok
test fetch_build_service::tests::required_source_override_rejects_unmatched_fetch_before_network ... ok
test fetch_build_service::tests::ordered_foreign_candidates_report_exhaustion_before_network ... ok
test fetch_build_service::tests::do_build_fetches_local_file ... ok
test fetch_build_service::tests::do_build_executable_sets_mode ... ok
test fetch_build_service::tests::do_build_uses_matching_source_override_without_network ... ok
test fetch_build_service::tests::ordered_foreign_candidates_select_first_available_source_state ... ok
test fetch_build_service::tests::do_build_fetches_tarball ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 641 filtered out; finished in 0.03s

```

### CLI

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 17 tests
test produce_backend_fix_and_host_nix_emit_parity_artifacts ... ignored, requires MANTLE_TEST_FIX_BACKEND_BINARY, MANTLE_TEST_NIX_INSTANTIATE_BINARY, and a reachable Nix daemon
test produce_backend_rejects_conflicting_target_modes ... ok
test produce_backend_rejects_unknown_backend ... ok
test foreign_import_cli_produce_aterm_rejects_input_mode_conflicts_without_artifacts ... ok
test foreign_import_cli_rejects_malformed_drv_without_partial_artifacts ... ok
test foreign_import_cli_rejects_drv_dir_missing_reachable_input_without_partial_artifacts ... ok
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_files_without_nix ... ok
test foreign_import_cli_produce_aterm_matches_nix_and_emits_guix_directory_artifacts ... ok
test foreign_import_cli_produce_aterm_rejects_bad_bundles_without_partial_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_dir_without_nix ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_produce_aterm_rejects_duplicate_and_bounded_byte_inputs ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok
fixed-output-mismatch build_report_blake3="1746553f31f2609a832b839ce99bd98005b425dc447394b0e2af29cb494e77fd"
test foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state ... ok
two-node build_report_blake3="d43e02fbb71558fa67a938df41558be076a2ec53625795704a83fcc304e27a38"
two-node provenance_audit_blake3="e3175cdbe4028d595d3664e831a18b38bbd5c1afbfa8c6bdb190af506c6f264e"
node-limit provenance_audit_blake3="73da9c07ee54f9e329d33f43989f496cc3b6403ac80555fe87442d70b7ac04a7"
guix-no-bin-sh build_report_blake3="72e6088eb8fd6e2e594c695fd303f99d65d8cf013c652309be66656d07b7daf6"
test foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs ... ok

test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.57s

```

## Final no-PathInfo CLI proof

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.03s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 1 test
fixed-output-mismatch build_report_blake3="1746553f31f2609a832b839ce99bd98005b425dc447394b0e2af29cb494e77fd"
test foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.15s

```

## Final canonical plan snapshot proof

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.71s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 1 test
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.03s

```

## Credential-bearing candidate rejection proof

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 55.08s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 1 test
test foreign_derivation_import::tests::nix_fetch_candidates_reject_invalid_ambiguous_and_private_forms ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2331 filtered out; finished in 0.00s

```

# Quality transcript

## Task formatting command

```text
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/foreign_derivation_import.rs:3057:
         legacy_source.as_object_mut().expect("source object").remove("fetch_candidates");
         let legacy = serde_json::from_value::<ForeignDerivationGraph>(legacy_value).expect("legacy graph decode");
         validate_graph(&legacy).expect("legacy graph validation");
[31m-        assert!(legacy
(B[m[31m-            .nodes
(B[m[31m-            .iter()
(B[m[31m-            .find(|node| node.original_derivation == NIXPKGS_SOURCE_DRV)
(B[m[31m-            .expect("legacy source")
(B[m[31m-            .fetch_candidates
(B[m[31m-            .is_empty());
(B[m[32m+        assert!(
(B[m[32m+            legacy
(B[m[32m+                .nodes
(B[m[32m+                .iter()
(B[m[32m+                .find(|node| node.original_derivation == NIXPKGS_SOURCE_DRV)
(B[m[32m+                .expect("legacy source")
(B[m[32m+                .fetch_candidates
(B[m[32m+                .is_empty()
(B[m[32m+        );
(B[m 
         let mut duplicate = artifacts.graph.clone();
         let duplicate_source = duplicate
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/foreign_derivation_import.rs:3080:
             .iter_mut()
             .find(|node| node.original_derivation == NIXPKGS_SOURCE_DRV)
             .expect("source node")
[31m-            .fetch_candidates = (0..=MAX_MIRROR_CANDIDATES)
(B[m[31m-            .map(|index| format!("https://mirror{index}.example/source"))
(B[m[31m-            .collect();
(B[m[32m+            .fetch_candidates =
(B[m[32m+            (0..=MAX_MIRROR_CANDIDATES).map(|index| format!("https://mirror{index}.example/source")).collect();
(B[m         assert_error_class(validate_graph(&oversized), "nix-fetch-candidate-count-invalid");
 
         let mut wrong_kind = artifacts.graph;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:18:
 use crate::errors::RunError;
 use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
 use crate::native_toolchain_closure::NativeToolchainClosureOptions;
[31m-use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m use crate::source_built_fixed_point::InitialOutputAuthorityState;
 use crate::source_built_fixed_point::ProofHermeticityMode;
[32m+use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m use crate::source_built_fixed_point::SourceAuthorityInput;
 use crate::source_built_fixed_point::SourceAuthorityRole;
 use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:28:
 use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
 use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
 use crate::source_built_fixed_point::SourceContentKind;
[31m-use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:39:
[32m+use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m use crate::source_built_fixed_point_dev_cache::FastFailDecision;
 use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
 use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:42:
[31m-use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[32m+use crate::source_bundle::SourceRecord;
(B[m use crate::source_bundle::assemble_source_bundle;
 use crate::source_bundle::materialize_source_record_payload;
 use crate::source_bundle::source_built_fixed_point_profile_records;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:47:
[31m-use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[31m-use crate::source_bundle::SourceRecord;
(B[m use crate::stagex_provider::StagexProviderRequest;
 use crate::stagex_transition::StagexTransitionRequest;
 

exit_code=1
```

## First-party Clippy

```text
[clippy] first-party strict gate
running: cargo clippy --workspace --all-targets --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation-core)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat)
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat-derive)
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/fuse-backend-rs)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-castore)
    Checking snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-tracing)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-build)
    Checking crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-action-result-core)
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-overlay-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-gc-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-repair-core)
    Checking crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache-core)
    Checking crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta-core)
    Checking crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-eval)
    Checking crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-wasm-component-core)
    Checking crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-shell-core)
    Checking crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-bootstrap-core)
    Checking mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/mantlepkgs-core)
    Checking crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-release-core)
    Checking mantle-portable-client-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/mantle-portable-client-core)
    Checking crunch-kernelscript-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-kernelscript-core)
    Checking crunch-hardware-simulation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-hardware-simulation-core)
    Checking crunch-spacewasm-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-spacewasm-core)
    Checking crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-shell)
    Checking crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation)
    Checking crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-project-core)
    Checking crunch-spacewasm v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-spacewasm)
    Checking crunch-hardware-simulation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-hardware-simulation)
    Checking crunch-kernelscript-adapter v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-kernelscript-adapter)
    Checking crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-nar)
    Checking crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-glue)
    Checking crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-wasm-component)
    Checking crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-project)
    Checking crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-store)
    Checking crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta)
    Checking crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache)
    Checking crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
    Checking crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rustc-wrapper)
    Checking crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-pipeline)
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 03s

exit_code=0
```

## Foreign import trust model

```text
foreign import trust-model doc check passed

exit_code=0
```

## Foreign import trust model self-test

```text
foreign import trust-model checker self-test passed

exit_code=0
```

## Diff whitespace

```text

exit_code=0
```


# Definitive quality rerun

## Changed-file rustfmt

```text

exit_code=0
```

## Task formatting command final

```text
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:18:
 use crate::errors::RunError;
 use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
 use crate::native_toolchain_closure::NativeToolchainClosureOptions;
[31m-use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m use crate::source_built_fixed_point::InitialOutputAuthorityState;
 use crate::source_built_fixed_point::ProofHermeticityMode;
[32m+use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m use crate::source_built_fixed_point::SourceAuthorityInput;
 use crate::source_built_fixed_point::SourceAuthorityRole;
 use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:28:
 use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
 use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
 use crate::source_built_fixed_point::SourceContentKind;
[31m-use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:39:
[32m+use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m use crate::source_built_fixed_point_dev_cache::FastFailDecision;
 use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
 use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:42:
[31m-use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[32m+use crate::source_bundle::SourceRecord;
(B[m use crate::source_bundle::assemble_source_bundle;
 use crate::source_bundle::materialize_source_record_payload;
 use crate::source_bundle::source_built_fixed_point_profile_records;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:47:
[31m-use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[31m-use crate::source_bundle::SourceRecord;
(B[m use crate::stagex_provider::StagexProviderRequest;
 use crate::stagex_transition::StagexTransitionRequest;
 

exit_code=1
```

## First-party Clippy final

```text
[clippy] first-party strict gate
running: cargo clippy --workspace --all-targets --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.00s

exit_code=0
```

## Foreign import trust model final

```text
foreign import trust-model doc check passed

exit_code=0
```

## Foreign import trust model self-test final

```text
foreign import trust-model checker self-test passed

exit_code=0
```

## Diff whitespace final

```text

exit_code=0
```


## Unchanged excluded formatting blocker

```text
exit_code=0
```
