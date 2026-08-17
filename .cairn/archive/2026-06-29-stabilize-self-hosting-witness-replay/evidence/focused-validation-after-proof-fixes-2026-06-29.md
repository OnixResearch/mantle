# Focused validation after proof fixes

Date: 2026-06-29T18:56:06Z

## nix develop -c cargo fmt --check -p mantle -p crunch-eval

~~~text
~~~

exit status: 0

## nix develop -c cargo test -p crunch-eval stdlib::tests::

~~~text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
    Blocking waiting for file lock on artifact directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.22s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_eval-b822061078b82761)

running 10 tests
test stdlib::tests::source_stdlib_candidates_empty_without_inputs ... ok
test stdlib::tests::write_stdlib_content_matches_embedded ... ok
test stdlib::tests::write_stdlib_creates_all_files ... ok
test stdlib::tests::source_stdlib_candidates_include_cwd_and_exe_ancestors ... ok
test stdlib::tests::embedded_stdlib_matches_repo_lib_directory ... ok
test stdlib::tests::stdlib_import_path_returns_dir_with_lib_ncl ... ok
test stdlib::tests::stdlib_import_path_can_force_embedded_copy ... ok
test stdlib::tests::write_stdlib_skips_rewrite_when_unchanged ... ok
test stdlib::tests::stdlib_is_importable ... ok
test stdlib::tests::written_embedded_stdlib_is_importable ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.06s

~~~

exit status: 0

## nix develop -c cargo test -p mantle --bin mantle bootstrap::tests::

~~~text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.36s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 16 tests
test bootstrap::tests::fetch_seed_metadata_logical_path_appends_provider_json ... ok
test bootstrap::tests::generate_seed_empty_packages ... ok
test bootstrap::tests::resolve_success ... ok
test bootstrap::tests::resolve_failure_reports_package ... ok
test bootstrap::tests::resolve_empty_path_rejected ... ok
test bootstrap::tests::generate_seed_sanitizes_hyphens ... ok
test bootstrap::tests::source_tree_file_candidates_empty_without_current_dir ... ok
test bootstrap::tests::source_tree_file_candidates_walk_up_from_current_dir ... ok
test bootstrap::tests::generate_seed_valid_structure ... ok
test bootstrap::tests::generate_source_root_seed_ncl_structure ... ok
test bootstrap::tests::fetch_seed_provider_loads_shared_seed_module ... ok
test bootstrap::tests::fetch_seed_provider_records_reduction_surface ... ok
test bootstrap::tests::generate_fetch_seed_ncl_structure ... ok
test bootstrap::tests::fetch_seed_provider_loads_with_embedded_stdlib ... ok
test bootstrap::tests::write_provider_metadata_records_shared_schema ... ok
test bootstrap::tests::stage_reduced_seed_provider_materializes_kernel_headers_in_target_include ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 880 filtered out; finished in 0.11s

~~~

exit status: 0

## nix develop -c cargo test -p mantle --bin mantle self_build::tests::

~~~text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 155 tests
test cargo_free_self_build::tests::child_blocker_fails_when_cargo_guard_was_invoked ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::child_blocker_accepts_successful_guarded_execution ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::effective_closure_uses_provider_only_when_manifest_absent ... ok
test cargo_free_self_build::tests::effective_closure_keeps_absent_non_claim_without_provider ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_rejects_null_source_digest ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_requires_source_digest ... ok
test cargo_free_self_build::tests::rustc_wrapper_script_strips_link_self_contained_runtime_args ... ok
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test cargo_free_self_build::tests::safe_path_component_replaces_unsafe_path_bytes ... ok
test cargo_free_self_build::tests::fixed_point_plan_threads_targets_into_stage_commands ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_stage_digest_mismatch ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_enforced_closure ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_valid_bounded_bundle ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_bounded_non_claims ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_explicit_complete_closure_claims ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_provider_supplies_claim ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test cargo_free_self_build::tests::self_build_non_claims_omit_closure_non_claim_when_provider_supplies_claim ... ok
test self_build::tests::bootstrap_gcc_ncl_replaces_host_time_touches_with_deterministic_touches ... ok
test self_build::tests::absolutize_path_makes_relative_path_absolute ... ok
test self_build::tests::bootstrap_gcc_ncl_records_deterministic_policy ... ok
test self_build::tests::bwrap_source_bin_dir_uses_parent_for_host_fallback ... ok
test self_build::tests::build_bwrap_path_entries_adds_wrapper_after_source ... ok
test self_build::tests::bwrap_source_display_roundtrip_crunch_built ... ok
test self_build::tests::build_bwrap_path_entries_deduplicates_wrapper ... ok
test self_build::tests::bwrap_source_display_roundtrip_host_fallback ... ok
test self_build::tests::bwrap_source_is_mantle_built_predicate ... ok
test self_build::tests::bwrap_source_parse_rejects_garbage ... ok
test self_build::tests::bwrap_source_display_roundtrip_declared_seed ... ok
test self_build::tests::choose_host_bwrap_path_prefers_wrapper ... ok
test self_build::tests::choose_host_bwrap_path_falls_back_to_path ... ok
test self_build::tests::dir_size_basic ... ok
test self_build::tests::denied_seccomp_events_do_not_fail_eligibility ... ok
test self_build::tests::enforce_source_resolution_policy_rejects_checkout_fallback_in_strict_later_stage ... ok
test self_build::tests::find_crunch_busybox_returns_none_for_empty_store ... ok
test self_build::tests::find_crunch_bwrap_returns_none_for_empty_store ... ok
test self_build::tests::find_crunch_bwrap_ignores_non_executable ... ok
test self_build::tests::find_crunch_busybox_ignores_wrong_suffix ... ok
test self_build::tests::find_crunch_busybox_finds_executable ... ok
test self_build::tests::find_crunch_bwrap_finds_executable_in_store ... ok
test self_build::tests::find_crunch_bwrap_ignores_wrong_name_suffix ... ok
test self_build::tests::find_crunch_outputs_empty_store ... ok
test self_build::tests::find_crunch_outputs_skips_without_binary ... ok
test self_build::tests::find_executable_on_path_returns_none_for_nonexistent ... ok
test self_build::tests::generate_ncl_has_all_bootstrap_deps ... ok
test self_build::tests::generate_ncl_fails_missing_bootstrap_alias_before_cargo ... ok
test self_build::tests::generate_ncl_has_exact_bwrap_and_busybox_paths ... ok
test self_build::tests::activate_bwrap_source_prepends_host_parent_dir ... ok
test self_build::tests::generate_ncl_has_build_essentials ... ok
test self_build::tests::generate_ncl_has_source_path ... ok
test self_build::tests::generate_ncl_respects_store_prefix ... ok
test self_build::tests::find_executable_on_path_finds_binary_in_tempdir ... ok
test self_build::tests::find_crunch_outputs_finds_matching_dirs ... ok
test self_build::tests::generate_ncl_rustc_wrapper_remaps_paths_without_rewriting_build_script_io ... ok
test self_build::tests::generate_ncl_source_is_plain_input ... ok
test self_build::tests::generate_ncl_uses_nix_store_env_var ... ok
test self_build::tests::find_executable_on_path_skips_non_executable_in_tempdir ... ok
test self_build::tests::generate_ncl_uses_exact_bootstrap_inputs_without_glob_discovery ... ok
test self_build::tests::generate_ncl_keeps_cargo_visible_values_stable_across_tool_store_paths ... ok
test self_build::tests::host_fallback_events_fail_eligibility ... ok
test self_build::tests::host_gcc_in_seccomp_events_fails_eligibility ... ok
test self_build::tests::invalidate_crunch_outputs_noop_on_empty_store ... ok
test self_build::tests::generate_ncl_uses_stable_bootstrap_aliases_for_cargo_visible_paths ... ok
test self_build::tests::is_executable_false_for_plain_file ... ok
test self_build::tests::legacy_fetch_provider_fails_stagex_eligibility ... ok
test self_build::tests::generate_ncl_uses_staged_vendor_config ... ok
test self_build::tests::legacy_provider_exec_path_fails_eligibility ... ok
test self_build::tests::missing_protected_transition_fails_eligibility ... ok
test self_build::tests::invalidate_crunch_outputs_removes_dirs ... ok
test self_build::tests::missing_stagex_metadata_fails_eligibility ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_unsafe_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_member_name_conflict ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_ambiguous_target_crt ... ok
test self_build::tests::nix_store_in_seccomp_events_fails_eligibility ... ok
test self_build::tests::prepend_to_path_adds_dir_first ... ok
test self_build::tests::prepend_to_path_handles_empty_path ... ok
test self_build::tests::report_format_roundtrip_with_declared_seed_bwrap ... ok
test self_build::tests::report_format_roundtrip_without_busybox ... ok
test self_build::tests::report_format_roundtrip_with_busybox ... ok
test self_build::tests::report_parse_ignores_non_report_proof_lines ... ok
test self_build::tests::report_parse_returns_none_for_empty_input ... ok
test self_build::tests::report_parse_returns_none_for_partial_input ... ok
test self_build::tests::report_format_roundtrip_with_stagex_metadata ... ok
test self_build::tests::protected_phase_transition_hashes_selected_tools ... ok
test self_build::tests::report_format_roundtrip_with_protected_transition ... ok
test self_build::tests::is_executable_true_for_real_binary ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_linker_for_collect2 ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_missing_target_crt ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_target_prefixed_member_name ... ok
test cargo_free_self_build::tests::effective_closure_prefers_explicit_manifest_over_provider_status ... ok
test self_build::tests::required_tools_count_is_exact ... ok
test self_build::tests::required_tools_includes_bwrap_and_busybox ... ok
test self_build::tests::declared_seed_bootstrap_tools_reject_shell_digest_drift ... ok
test self_build::tests::declared_seed_bootstrap_tools_select_inventory_bwrap_and_shell ... ok
test self_build::tests::resolve_bwrap_source_prefers_crunch_built ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_malformed_lock_quote_without_panic ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_non_vendored_directory_target ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_registry_package_without_lock_checksum ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_missing_locked_package ... ok
test self_build::tests::prepare_self_build_source_rejects_source_outside_store_dir ... ok
test self_build::tests::self_build_step_count_is_four ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_missing_vendor_config ... ok
test self_build::tests::source_root_provider_fails_stagex_eligibility ... ok
test self_build::tests::stagex_eligible_report_passes_validation ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_package_checksum_mismatch ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_file_checksum_mismatch ... ok
test self_build::tests::require_checked_vendor_inputs_accepts_git_package_without_registry_checksum ... ok
test self_build::tests::require_checked_vendor_inputs_rejects_extra_vendored_package ... ok
test self_build::tests::resolve_single_root_output_dir_rejects_multiple_matches ... ok
test self_build::tests::resolve_single_root_output_dir_uses_root_labels ... ok
test self_build::tests::run_cmd_reports_failure ... ok
test self_build::tests::resolve_explicit_bootstrap_bwrap_source_accepts_exact_store_binary ... ok
test self_build::tests::resolve_store_entry_name_uses_exact_store_child ... ok
test self_build::tests::tree_fingerprint_changes_with_content ... ok
test self_build::tests::tree_fingerprint_changes_with_mode_bits ... ok
test self_build::tests::tree_fingerprint_changes_with_same_size_content ... ok
test self_build::tests::validate_bootstrap_tools_errors_on_missing_file ... ok
test self_build::tests::resolve_explicit_bootstrap_busybox_path_ignores_stale_sibling ... ok
test self_build::tests::resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling ... ok
test self_build::tests::validate_bootstrap_tools_errors_on_partial ... ok
test self_build::tests::tree_fingerprint_deterministic ... ok
test self_build::tests::validate_bootstrap_tools_passes_when_present ... ok
test cargo_free_self_build::tests::receipt_bound_path_env_omits_ambient_path_entries ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_linker_digest_mismatch ... ok
test self_build::tests::resolve_explicit_bootstrap_busybox_path_rejects_path_outside_store ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_pkg_config_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_sysroot_leakage ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok
test self_build::tests::validate_staged_source_dir_rejects_missing_lib_dir ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_runtime_library_digest_mismatch ... ok
test self_build::tests::verify_tools_on_disk_errors_without_busybox ... ok
test self_build::tests::validate_staged_source_dir_rejects_tampered_contents ... ok
test self_build::tests::verify_tools_on_disk_succeeds_with_both ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_c_compiler_digest_mismatch ... ok
test self_build::tests::resolve_bwrap_source_errors_without_host_bwrap_on_controlled_path ... ok
test self_build::tests::resolve_bwrap_source_strict_rejects_host_fallback_once_bootstrap_root_exists ... ok
test self_build::tests::resolve_bwrap_source_falls_back_to_controlled_host_path ... ok
test self_build::tests::verify_tools_on_disk_error_names_bwrap_ncl ... ok
test self_build::tests::verify_tools_on_disk_errors_on_empty_store ... ok
test cargo_free_self_build::tests::compatibility_probe_uses_receipt_bound_path_for_explicit_closure ... ok
test cargo_free_self_build::tests::compatibility_probe_failure_with_explicit_closure_fails_closed ... ok
test self_build::tests::prepare_self_build_source_reuses_exact_staged_source ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test self_build::tests::require_checked_vendor_inputs_accepts_matching_vendor_tree ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rewrites_response_file_runtime_inputs ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_materializes_declared_runtime_inputs ... ok
test self_build::tests::copy_selected_source_tree_copies_only_allowlisted_entries ... ok

test result: ok. 155 passed; 0 failed; 0 ignored; 0 measured; 741 filtered out; finished in 0.04s

~~~

exit status: 0

## nix develop -c cargo test -p mantle --test self_hosting expected_store_prefix

~~~text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running tests/self_hosting.rs (/home/brittonr/.cargo-target/debug/deps/self_hosting-662dbc81a90cc4d2)

running 2 tests
test expected_store_prefix_matches_nix_compat_precedence ... ok
test expected_store_prefix_defaults_to_crunch_store ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.00s

~~~

exit status: 0

## nix develop -c cargo test -p mantle --test self_hosting -- --list

~~~text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/self_hosting.rs (/home/brittonr/.cargo-target/debug/deps/self_hosting-662dbc81a90cc4d2)
append_no_host_tools_stage0_args_preserves_base_command_and_adds_inventory: test
audit_hashing_rejects_excessively_deep_trees: test
collect_embedded_store_paths_filters_relevant_entries: test
create_stage0_scrubbed_path_blocks_nix_binaries: test
diff_binary_bytes_reports_first_mismatch: test
expected_store_prefix_defaults_to_crunch_store: test
expected_store_prefix_matches_nix_compat_precedence: test
filtered_proof_tool_names_removes_blocked_host_tools_for_no_host_mode: test
proof_mode_defaults_to_fixed_point: test
prove_self_hosting_script_accepts_exact_scratch_threshold: test
prove_self_hosting_script_anchors_default_scratch_root_to_repo_root_from_non_repo_cwd: test
prove_self_hosting_script_anchors_relative_bundle_dir_and_updates_latest: test
prove_self_hosting_script_anchors_relative_sandbox_shell_to_repo_root: test
prove_self_hosting_script_anchors_relative_scratch_override_to_repo_root_from_non_repo_cwd: test
prove_self_hosting_script_creates_selected_scratch_root_when_missing: test
prove_self_hosting_script_discovers_repo_local_default_sandbox_shell_when_env_is_bin_sh: test
prove_self_hosting_script_exports_no_host_tools_inventory_and_blocks_host_tools: test
prove_self_hosting_script_exports_non_nix_host_mode_and_inventory_doc: test
prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default: test
prove_self_hosting_script_fails_fast_when_default_sandbox_shell_missing_without_nix_build_fallback: test
prove_self_hosting_script_generates_no_host_tools_inventory_from_explicit_seeds: test
prove_self_hosting_script_no_host_tools_requires_inventory_before_launch: test
prove_self_hosting_script_preserves_absolute_bundle_dir_and_updates_latest: test
prove_self_hosting_script_preserves_absolute_scratch_override_path: test
prove_self_hosting_script_rejects_below_threshold_override_scratch_before_proof_work: test
prove_self_hosting_script_rejects_below_threshold_scratch_before_proof_work: test
prove_self_hosting_script_rejects_create_failure_default_scratch_before_proof_work: test
prove_self_hosting_script_rejects_create_failure_scratch_override_without_fallback: test
prove_self_hosting_script_rejects_option_like_bundle_dir_value: test
prove_self_hosting_script_rejects_unusable_default_scratch_before_proof_work: test
prove_self_hosting_script_rejects_unusable_scratch_override_without_fallback: test
prove_self_hosting_script_rejects_unwritable_default_scratch_before_proof_work: test
prove_self_hosting_script_rejects_unwritable_scratch_override_without_fallback: test
prove_self_hosting_script_reports_default_scratch_policy_in_check_mode: test
prove_self_hosting_script_reports_scratch_override_in_check_mode: test
prove_self_hosting_script_routes_spawned_tmpdir_and_cargo_target_under_default_scratch_root: test
prove_self_hosting_script_uses_bundle_dir_env_when_cli_arg_absent: test
record_stage_evidence_with_populated_store_avoids_audit_limit: test
record_stage_evidence_writes_audit_and_diagnostics_paths: test
remove_crunch_outputs_ignores_non_mantle_entries: test
remove_crunch_outputs_removes_read_only_crunch_output: test
render_stage_diagnostics_includes_proof_lines_and_paths: test
resolve_proof_bundle_dir_anchors_relative_env_to_repo_root: test
resolve_proof_bundle_dir_preserves_absolute_env_path: test
run_command_live_writes_stage_stream_files: test
self_hosting_controlled_failure_reports_breadcrumbs: test
self_hosting_controlled_failure_reports_breadcrumbs_trigger: test
self_hosting_stage0_stage1_stage2: test
stage_context_reads_saved_snapshot_from_disk: test
write_proof_bundle_copies_stage_artifacts_and_manifest: test

50 tests, 0 benchmarks
~~~

exit status: 0

## git diff --check

~~~text
~~~

exit status: 0

