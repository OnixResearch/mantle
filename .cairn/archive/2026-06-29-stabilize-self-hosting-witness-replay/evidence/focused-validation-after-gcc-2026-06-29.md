# Focused validation after GCC determinism fix

Date: 2026-06-29


## `nix develop -c cargo fmt --check -p mantle`

```text
```

exit status: `0`

## `nix develop -c cargo test -p mantle --bin mantle self_build::tests::`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 06s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 155 tests
test cargo_free_self_build::tests::child_blocker_accepts_successful_guarded_execution ... ok
test cargo_free_self_build::tests::child_blocker_fails_when_cargo_guard_was_invoked ... ok
test cargo_free_self_build::tests::effective_closure_keeps_absent_non_claim_without_provider ... ok
test cargo_free_self_build::tests::effective_closure_uses_provider_only_when_manifest_absent ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::fixed_point_plan_threads_targets_into_stage_commands ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_provider_supplies_claim ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_explicit_complete_closure_claims ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_requires_source_digest ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_rejects_null_source_digest ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_valid_bounded_bundle ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_bounded_non_claims ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_stage_digest_mismatch ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_enforced_closure ... ok
test cargo_free_self_build::tests::self_build_non_claims_omit_closure_non_claim_when_provider_supplies_claim ... ok
test cargo_free_self_build::tests::safe_path_component_replaces_unsafe_path_bytes ... ok
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test cargo_free_self_build::tests::rustc_wrapper_script_strips_link_self_contained_runtime_args ... ok
test self_build::tests::bwrap_source_bin_dir_uses_parent_for_host_fallback ... ok
test self_build::tests::bwrap_source_display_roundtrip_crunch_built ... ok
test self_build::tests::bwrap_source_display_roundtrip_declared_seed ... ok
test self_build::tests::bootstrap_gcc_ncl_replaces_host_time_touches_with_deterministic_touches ... ok
test self_build::tests::bwrap_source_display_roundtrip_host_fallback ... ok
test self_build::tests::bwrap_source_is_mantle_built_predicate ... ok
test self_build::tests::bwrap_source_parse_rejects_garbage ... ok
test self_build::tests::choose_host_bwrap_path_falls_back_to_path ... ok
test self_build::tests::choose_host_bwrap_path_prefers_wrapper ... ok
test self_build::tests::denied_seccomp_events_do_not_fail_eligibility ... ok
test self_build::tests::enforce_source_resolution_policy_rejects_checkout_fallback_in_strict_later_stage ... ok
test self_build::tests::build_bwrap_path_entries_deduplicates_wrapper ... ok
test self_build::tests::dir_size_basic ... ok
test self_build::tests::find_crunch_busybox_ignores_wrong_suffix ... ok
test self_build::tests::absolutize_path_makes_relative_path_absolute ... ok
test self_build::tests::build_bwrap_path_entries_adds_wrapper_after_source ... ok
test self_build::tests::find_crunch_busybox_finds_executable ... ok
test self_build::tests::bootstrap_gcc_ncl_records_deterministic_policy ... ok
test cargo_free_self_build::tests::effective_closure_prefers_explicit_manifest_over_provider_status ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_linker_for_collect2 ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_member_name_conflict ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_unsafe_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_missing_target_crt ... ok
test self_build::tests::generate_ncl_fails_missing_bootstrap_alias_before_cargo ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test self_build::tests::generate_ncl_has_all_bootstrap_deps ... ok
test self_build::tests::generate_ncl_has_exact_bwrap_and_busybox_paths ... ok
test self_build::tests::generate_ncl_has_build_essentials ... ok
test self_build::tests::generate_ncl_has_source_path ... ok
test self_build::tests::generate_ncl_respects_store_prefix ... ok
test self_build::tests::generate_ncl_keeps_cargo_visible_values_stable_across_tool_store_paths ... ok
test self_build::tests::generate_ncl_source_is_plain_input ... ok
test self_build::tests::generate_ncl_rustc_wrapper_remaps_paths_without_rewriting_build_script_io ... ok
test self_build::tests::generate_ncl_uses_exact_bootstrap_inputs_without_glob_discovery ... ok
test self_build::tests::generate_ncl_uses_nix_store_env_var ... ok
test self_build::tests::generate_ncl_uses_staged_vendor_config ... ok
test self_build::tests::generate_ncl_uses_stable_bootstrap_aliases_for_cargo_visible_paths ... ok
test self_build::tests::host_fallback_events_fail_eligibility ... ok
test self_build::tests::host_gcc_in_seccomp_events_fails_eligibility ... ok
test self_build::tests::find_executable_on_path_returns_none_for_nonexistent ... ok
test self_build::tests::find_crunch_busybox_returns_none_for_empty_store ... ok
test self_build::tests::invalidate_crunch_outputs_noop_on_empty_store ... ok
test self_build::tests::legacy_fetch_provider_fails_stagex_eligibility ... ok
test self_build::tests::legacy_provider_exec_path_fails_eligibility ... ok
test self_build::tests::missing_protected_transition_fails_eligibility ... ok
test self_build::tests::find_crunch_bwrap_ignores_non_executable ... ok
test self_build::tests::missing_stagex_metadata_fails_eligibility ... ok
test self_build::tests::nix_store_in_seccomp_events_fails_eligibility ... ok
test self_build::tests::find_crunch_outputs_empty_store ... ok
test self_build::tests::find_crunch_bwrap_finds_executable_in_store ... ok
test self_build::tests::is_executable_true_for_real_binary ... ok
test self_build::tests::find_crunch_bwrap_returns_none_for_empty_store ... ok
test self_build::tests::find_crunch_bwrap_ignores_wrong_name_suffix ... ok
test self_build::tests::report_format_roundtrip_with_busybox ... ok
test self_build::tests::report_format_roundtrip_with_declared_seed_bwrap ... ok
test self_build::tests::report_format_roundtrip_with_stagex_metadata ... ok
test self_build::tests::report_format_roundtrip_with_protected_transition ... ok
test self_build::tests::report_parse_ignores_non_report_proof_lines ... ok
test self_build::tests::report_format_roundtrip_without_busybox ... ok
test self_build::tests::report_parse_returns_none_for_empty_input ... ok
test self_build::tests::report_parse_returns_none_for_partial_input ... ok
test self_build::tests::find_crunch_outputs_skips_without_binary ... ok
test self_build::tests::invalidate_crunch_outputs_removes_dirs ... ok
test self_build::tests::activate_bwrap_source_prepends_host_parent_dir ... ok
test self_build::tests::find_crunch_outputs_finds_matching_dirs ... ok
test self_build::tests::declared_seed_bootstrap_tools_select_inventory_bwrap_and_shell ... ok
test self_build::tests::is_executable_false_for_plain_file ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_linker_digest_mismatch ... ok
test self_build::tests::declared_seed_bootstrap_tools_reject_shell_digest_drift ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_sysroot_leakage ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_target_prefixed_member_name ... ok
test self_build::tests::required_tools_count_is_exact ... ok
test self_build::tests::required_tools_includes_bwrap_and_busybox ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_pkg_config_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_c_compiler_digest_mismatch ... ok
test self_build::tests::protected_phase_transition_hashes_selected_tools ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_runtime_library_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_ambiguous_target_crt ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test self_build::tests::prepare_self_build_source_rejects_source_outside_store_dir ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test self_build::tests::prepare_self_build_source_reuses_exact_staged_source ... ok
test self_build::tests::resolve_single_root_output_dir_uses_root_labels ... ok
test self_build::tests::resolve_single_root_output_dir_rejects_multiple_matches ... ok
test self_build::tests::self_build_step_count_is_four ... ok
test self_build::tests::source_root_provider_fails_stagex_eligibility ... ok
test self_build::tests::stagex_eligible_report_passes_validation ... ok
test self_build::tests::require_checked_vendor_inputs_accepts_matching_vendor_tree ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_extra_vendored_package ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_registry_package_without_lock_checksum ... ok
test self_build::tests::require_checked_vendor_inputs_accepts_git_package_without_registry_checksum ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_non_vendored_directory_target ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_malformed_lock_quote_without_panic ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_missing_locked_package ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_file_checksum_mismatch ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_missing_vendor_config ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_package_checksum_mismatch ... ok
test self_build::tests::run_cmd_reports_failure ... ok
test self_build::tests::validate_bootstrap_tools_errors_on_missing_file ... ok
test self_build::tests::tree_fingerprint_changes_with_same_size_content ... ok
test self_build::tests::resolve_store_entry_name_uses_exact_store_child ... ok
test self_build::tests::find_executable_on_path_finds_binary_in_tempdir ... ok
test self_build::tests::validate_bootstrap_tools_errors_on_partial ... ok
test self_build::tests::tree_fingerprint_changes_with_mode_bits ... ok
test self_build::tests::tree_fingerprint_deterministic ... ok
test self_build::tests::verify_tools_on_disk_succeeds_with_both ... ok
test self_build::tests::resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling ... ok
test self_build::tests::tree_fingerprint_changes_with_content ... ok
test self_build::tests::validate_staged_source_dir_rejects_missing_lib_dir ... ok
test self_build::tests::validate_staged_source_dir_rejects_tampered_contents ... ok
test self_build::tests::verify_tools_on_disk_errors_without_busybox ... ok
test self_build::tests::validate_bootstrap_tools_passes_when_present ... ok
test self_build::tests::resolve_explicit_bootstrap_busybox_path_ignores_stale_sibling ... ok
test self_build::tests::resolve_explicit_bootstrap_bwrap_source_accepts_exact_store_binary ... ok
test self_build::tests::resolve_bwrap_source_prefers_crunch_built ... ok
test self_build::tests::resolve_explicit_bootstrap_busybox_path_rejects_path_outside_store ... ok
test self_build::tests::find_executable_on_path_skips_non_executable_in_tempdir ... ok
test cargo_free_self_build::tests::compatibility_probe_failure_with_explicit_closure_fails_closed ... ok
test self_build::tests::prepend_to_path_adds_dir_first ... ok
test cargo_free_self_build::tests::compatibility_probe_uses_receipt_bound_path_for_explicit_closure ... ok
test self_build::tests::prepend_to_path_handles_empty_path ... ok
test self_build::tests::resolve_bwrap_source_errors_without_host_bwrap_on_controlled_path ... ok
test cargo_free_self_build::tests::receipt_bound_path_env_omits_ambient_path_entries ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok
test self_build::tests::resolve_bwrap_source_falls_back_to_controlled_host_path ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_materializes_declared_runtime_inputs ... ok
test self_build::tests::resolve_bwrap_source_strict_rejects_host_fallback_once_bootstrap_root_exists ... ok
test self_build::tests::verify_tools_on_disk_error_names_bwrap_ncl ... ok
test self_build::tests::verify_tools_on_disk_errors_on_empty_store ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rewrites_response_file_runtime_inputs ... ok
test self_build::tests::copy_selected_source_tree_copies_only_allowlisted_entries ... ok

test result: ok. 155 passed; 0 failed; 0 ignored; 0 measured; 739 filtered out; finished in 0.02s

```

exit status: `0`

## `nix develop -c cargo test -p mantle --bin mantle witness_rebuild::tests::`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 24 tests
test witness_rebuild::tests::default_witness_scratch_dir_appends_work_suffix ... ok
test witness_rebuild::tests::bootstrap_divergence_diagnostic_reports_convergence ... ok
test witness_rebuild::tests::launched_workflow_command_records_proof_mode_arguments ... ok
test witness_rebuild::tests::resolve_proof_bundle_artifact_path_anchors_relative_paths ... ok
test witness_rebuild::tests::parse_request_relative_path_rejects_parent_components ... ok
test witness_rebuild::tests::resolve_proof_bundle_artifact_path_rejects_parent_escape ... ok
test witness_rebuild::tests::workflow_args_preserve_non_nix_proof_mode_and_reject_unknown_modes ... ok
test witness_rebuild::tests::validate_supported_workflow_identity_rejects_unknown_pair ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_absolute_proof_binary_path ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_escaping_proof_binary_path ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_file_helper_owned_cargo_target_entry ... ok
test witness_rebuild::tests::validate_existing_scratch_root_allows_helper_owned_dirs_only ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_unexpected_entries ... ok
test witness_rebuild::tests::validate_rebuilt_output_digests_rejects_stripped_equivalent_but_different_binary ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlink_root ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlinked_helper_owned_tmp_entry ... ok
test witness_rebuild::tests::failure_audit_reports_gcc_bootstrap_divergence_root ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_invalid_provider_fixed_point_proof ... ok
test witness_rebuild::tests::failure_audit_reports_digest_mismatch_diagnostics ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_missing_expected_digest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_maps_two_expected_outputs_by_digest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_preserves_one_output_legacy_manifest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 870 filtered out; finished in 0.02s

```

exit status: `0`

## `git diff --check`

```text
```

exit status: `0`

## `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}
```

exit status: `0`

## `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal stabilize-self-hosting-witness-replay --root /home/brittonr/git/mantle`

```text
{
  "change": "stabilize-self-hosting-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "60befbee24beedbcec377612c51541b05c44a49f17810fd712655688760b32a0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "dac6ee10abc50ffd99318d17a8e56156a61c0965b6afdafbde6a7496a2c860a3",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

exit status: `0`

## `nix run path:/home/brittonr/git/cairn#cairn -- gate design stabilize-self-hosting-witness-replay --root /home/brittonr/git/mantle`

```text
{
  "change": "stabilize-self-hosting-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "fc3c3d5f9f2ebfa303b13926d1053214b05bb564457adebab70039ad384ef177",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "435e4ccd192d92521f8cd5ca00c8c078be7af10312b3314919b15b59b498a4df",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

exit status: `0`

## `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks stabilize-self-hosting-witness-replay --root /home/brittonr/git/mantle`

```text
{
  "change": "stabilize-self-hosting-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "4cb2269ca33447ab795b738a20de78de783efb089269c54a7d0f5ac7c43b757c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6ef541404837d3027bb8c0c22cbc0350184596ca550cc6db5de5d80fa1981e9d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

exit status: `0`
