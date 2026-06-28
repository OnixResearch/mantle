# Provider fixed-point path normalization validation

Task-ID: V1
Covers: verification_evidence.provider_fixed_point_path_normalization

## Focused Rust validation

```text
$ cargo fmt -p mantle --check
$ cargo test -p mantle --bin mantle rust_plan -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 174 tests
test rust_plan::tests::aws_lc_memcmp_guard_failure_becomes_stable_compiler_guard_blocker ... ok
test rust_plan::tests::allowed_rust_topology_compile_env_rejects_unrelated_ambient_env ... ok
test rust_plan::tests::bind_build_script_metadata_adds_dep_env_for_linked_dependency ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates ... ok
test rust_plan::tests::build_script_profile_env_child_ignores_ambient_process_env_probe ... ok
test rust_plan::tests::build_script_metadata_producer_index_keeps_same_alias_duplicates_ambiguous ... ok
test rust_plan::tests::build_script_profile_env_derives_dev_and_release_defaults ... ok
test rust_plan::tests::build_script_target_cfg_env_derives_x86_64_linux_values ... ok
test rust_plan::tests::build_script_target_cfg_env_uses_empty_env_for_wasm_unknown ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_target_host_artifact_producer ... ok
test rust_plan::tests::combined_unit_topology_blocks_missing_linked_metadata_producer ... ok
test rust_plan::tests::bind_all_host_artifacts_leaves_unknown_custom_build_alias_placeholder ... ok
test rust_plan::tests::combined_unit_topology_keeps_standalone_host_units ... ok
test rust_plan::tests::bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder ... ok
test rust_plan::tests::combined_unit_topology_orders_host_build_script_before_same_package_proc_macro ... ok
test rust_plan::tests::bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact ... ok
test rust_plan::tests::combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib ... ok
test rust_plan::tests::append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_custom_build_main_alias_placeholder ... ok
test rust_plan::tests::combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit ... ok
test rust_plan::tests::bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates ... ok
test rust_plan::tests::combined_unit_topology_orders_linked_metadata_before_dependent_build_script ... ok
test rust_plan::tests::bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate ... ok
test rust_plan::tests::combined_unit_topology_orders_target_host_and_proc_macro_edges ... ok
test rust_plan::tests::host_dependency_remap_preserves_explicit_non_target_producers ... ok
test rust_plan::tests::build_script_child_env_does_not_forward_compiler_guard_bypass_env ... ok
test rust_plan::tests::host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks ... ok
test rust_plan::tests::build_script_child_env_sets_tool_target_and_manifest_package_name ... ok
test rust_plan::tests::bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata ... ok
test rust_plan::tests::host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets ... ok
test rust_plan::tests::host_dependency_topology_accepts_proc_macro_host_producer ... ok
test rust_plan::tests::build_script_child_env_omits_manifest_dir_without_source_arg ... ok
test rust_plan::tests::native_feature_resolver_exposes_bare_optional_dependency_as_cfg_feature ... ok
test rust_plan::tests::native_feature_resolver_leaves_optional_dependency_unselected_without_feature ... ok
test rust_plan::tests::native_feature_resolver_blocks_malformed_feature_edges ... ok
test rust_plan::tests::bind_dependency_artifacts_uses_selected_unit_variant ... ok
test rust_plan::tests::host_dependency_topology_accepts_target_lib_producer ... ok
test rust_plan::tests::build_script_metadata_success_receipt_records_selected_compiler_route ... ok
test rust_plan::tests::native_cargo_package_env_sets_version_components_and_empty_defaults ... ok
test rust_plan::tests::host_dependency_topology_blocks_missing_producer ... ok
test rust_plan::tests::native_feature_resolver_models_dependency_feature_edges_and_rejects_unknown_entries ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_fails_closed_on_missing_lockfile ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test rust_plan::tests::native_feature_role_resolver_keeps_normal_build_and_host_features_separate ... ok
test rust_plan::tests::native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units ... ok
test rust_plan::tests::native_host_dependencies_use_normal_deps_only_for_proc_macro_units ... ok
test rust_plan::tests::native_host_metadata_dependencies_follow_selected_target_artifacts_only ... ok
test rust_plan::tests::native_host_metadata_dependencies_ignore_unselected_linked_manifest_edges ... ok
test rust_plan::tests::collect_native_manifest_lock_texts_reads_root_member_manifests_and_lockfile ... ok
test rust_plan::tests::native_cargo_package_env_inherits_optional_workspace_metadata ... ok
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_host_derivation_adds_compiler_proc_macro_extern ... ok
test rust_plan::tests::native_host_derivation_caps_lints_for_registry_source ... ok
test rust_plan::tests::native_host_derivation_carries_manifest_package_name_for_build_script_env ... ok
test rust_plan::tests::build_script_package_root_falls_back_to_real_source_when_manifest_dir_is_virtual ... ok
test rust_plan::tests::native_host_unit_derivation_emits_metadata_disambiguator ... ok
test rust_plan::tests::native_host_planning_keeps_selected_same_package_build_script_for_proc_macro ... ok
test rust_plan::tests::native_git_dev_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_host_planning_binds_run_custom_build_alias_to_exact_unique_build_unit ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_accept_supported_subset ... ok
test rust_plan::tests::native_host_planning_links_proc_macro_dependency_proc_macros_without_cargo_edges ... ok
test rust_plan::tests::native_host_planning_does_not_bind_run_custom_build_alias_to_first_duplicate ... ok
test rust_plan::tests::captures_normalized_oracle_receipt ... ok
test rust_plan::tests::native_manifest_lock_unsupported_blockers_reject_patch_replace_target_build_and_unknown_lock_source ... ok
test rust_plan::tests::native_package_oracle_comparison_accepts_supported_fixture ... ok
test rust_plan::tests::native_host_planning_selects_lib_kind_proc_macro_crate_type ... ok
test rust_plan::tests::native_package_oracle_comparison_blocks_identity_mismatch_and_missing_cargo_package ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_blocks_unknown_or_malformed_syntax ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_target_cfg_predicate_scope_evaluates_nested_common_predicates ... ok
test rust_plan::tests::native_manifest_proc_macro_alias_feeds_target_planning ... ok
test rust_plan::tests::native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only ... ok
test rust_plan::tests::dependency_chain_blocks_missing_producer_before_consumer_rustc ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_custom_build_unit_ids ... ok
test rust_plan::tests::cargo_unit_derivation_promotes_proc_macro_crate_type_to_host_unit ... ok
test rust_plan::tests::build_script_child_env_ignores_ambient_profile_env ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_package_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_native_dependency_without_source_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok
test rust_plan::tests::native_unit_graph_follows_native_dependency_facts_without_cargo_unit_edges ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_uses_lib_crate_name_when_package_name_differs ... ok
test rust_plan::tests::native_unit_graph_uses_stable_native_unit_identity ... ok
test rust_plan::tests::native_unit_metadata_disambiguator_ignores_ambient_tool_roots ... ok
test rust_plan::tests::compiler_policy_deny_mode_rejects_adapter_failure ... ok
test rust_plan::tests::compiler_policy_expected_manifest_digest_mismatch_blocks_required_mode ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_driver ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_policy_digest_mismatch ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_standards_artifact ... ok
test rust_plan::tests::non_aws_lc_guard_text_stays_plain_build_script_failure ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_toolchain_mismatch ... ok
test rust_plan::tests::package_only_dependency_artifacts_bind_single_renamed_crate_candidate ... ok
test rust_plan::tests::parse_build_script_metadata_accepts_bounded_link_lib_forms ... ok
test rust_plan::tests::parse_build_script_metadata_captures_link_metadata ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_bad_custom_key ... ok
test rust_plan::tests::parse_build_script_metadata_rejects_unsafe_link_lib_forms ... ok
test rust_plan::tests::package_only_dependency_artifacts_fail_on_ambiguous_unit_variants ... ok
test rust_plan::tests::compiler_policy_required_mode_blocks_missing_lint_library ... ok
test rust_plan::tests::parse_native_lockfile_text_rejects_malformed_package_records ... ok
test rust_plan::tests::parse_native_lockfile_text_extracts_source_identities_revisions_checksums_and_edges ... ok
test rust_plan::tests::proc_macro_crate_type_only_overrides_lib_shaped_targets ... ok
test rust_plan::tests::parse_native_manifest_text_rejects_invalid_inherited_package_version ... ok
test rust_plan::tests::redacted_diagnostic_preserves_non_temp_context ... ok
test rust_plan::tests::redacted_diagnostic_limits_lines_and_redacts_temp_paths ... ok
test rust_plan::tests::parse_native_manifest_text_extracts_core_workspace_package_target_and_dependency_facts ... ok
test rust_plan::tests::registry_dependency_source_rejects_missing_exact_same_name_version ... ok
test rust_plan::tests::registry_dependency_source_uses_default_caret_semver_compatibility ... ok
test rust_plan::tests::registry_dependency_source_uses_exact_version_when_names_repeat ... ok
test rust_plan::tests::native_unit_derivation_leaves_path_sources_uncapped ... ok
test rust_plan::tests::rust_topology_child_env_omits_empty_inherited_path ... ok
test rust_plan::tests::native_unit_derivation_emits_resolved_feature_closure_cfg_args ... ok
test rust_plan::tests::rust_topology_child_env_forwards_allowlisted_compile_env ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_compile_env ... ok
test rust_plan::tests::native_unit_derivation_emits_stable_metadata_disambiguator ... ok
test rust_plan::tests::native_unit_derivation_adds_selected_feature_cfg_args ... ok
test rust_plan::tests::rust_topology_child_env_preserves_non_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_derivation_path ... ok
test rust_plan::tests::native_unit_derivation_caps_lints_for_registry_and_git_sources ... ok
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok
test rust_plan::tests::rust_topology_child_env_rejects_ambient_compiler_guard_bypass_env ... ok
test rust_plan::tests::rust_topology_runtime_args_disable_self_contained_linker_by_default ... ok
test rust_plan::tests::native_host_planning_preserves_duplicate_selected_proc_macro_unit_ids ... ok
test rust_plan::tests::rust_topology_runtime_args_ignore_near_match_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_joined_explicit_linker_mode ... ok
test rust_plan::tests::registry_dependency_source_uses_highest_matching_comparator_range ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_split_explicit_linker_mode ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::native_unit_graph_adds_build_dependency_producer_unit ... ok
test rust_plan::tests::rustc_metadata_disambiguator_distinguishes_same_crate_package_versions ... ok
test rust_plan::tests::path_source_digest_ignores_root_metadata_without_hiding_source_changes ... ok
test rust_plan::tests::deterministic_release_paths_add_remaps_and_provider_placeholder_env ... ok
test rust_plan::tests::native_manifest_package_build_path_and_false_control_targets ... ok
test rust_plan::tests::source_closure_blocks_registry_without_checksum ... ok
test rust_plan::tests::source_built_c_compiler_route_json_validates_receipt_bound_identity ... ok
test rust_plan::tests::source_built_c_compiler_route_json_rejects_non_compiler_route ... ok
test rust_plan::tests::target_dependency_producer_index_uses_selected_unit_variant ... ok
test rust_plan::tests::native_manifest_missing_edition_uses_cargo_default ... ok
test rust_plan::tests::unit_dependency_artifacts_preserve_selected_cargo_unit_id ... ok
test rust_plan::tests::unit_dependency_artifacts_skip_host_producer_edges ... ok
test rust_plan::tests::native_manifest_declared_edition_feeds_host_derivation_args ... ok
test rust_plan::tests::native_manifest_workspace_edition_feeds_target_derivation_args ... ok
test rust_plan::tests::rust_unit_execution_blocks_source_closure_blocker_before_rustc ... ok
test rust_plan::tests::unit_derivation_graph_blocks_doctest_or_non_build_modes ... ok
test rust_plan::tests::unit_graph_failure_fails_closed ... ok
test rust_plan::tests::unit_derivation_graph_blocks_unsupported_target_kinds ... ok
test rust_plan::tests::source_closure_records_registry_git_and_path_identities ... ok
test rust_plan::tests::unit_derivation_graph_represents_build_script_host_units ... ok
test rust_plan::tests::selected_dependency_search_paths_follow_unit_variant_closure_only ... ok
test rust_plan::tests::unit_derivation_graph_emits_binary_with_dependency_artifact ... ok
test rust_plan::tests::unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units ... ok
test rust_plan::tests::native_package_target_fragment_ignores_out_of_scope_oracle_target_kinds ... ok
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok
test rust_plan::tests::native_package_target_fragment_matches_supported_path_workspace ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test tests::build_cli_project_selector_is_not_implicit_rust_plan ... ok
test rust_plan::tests::compiler_policy_audit_mode_records_adapter_identity_and_waivers ... ok
test rust_plan::tests::compiler_policy_json_receipt_contains_identity_and_waiver_summary ... ok
test rust_plan::tests::compiler_policy_required_mode_records_complete_static_identity ... ok
test rust_plan::tests::compiler_policy_required_mode_rejects_raw_rustc_cached_output ... ok
test rust_plan::tests::no_cargo_capture_binds_captured_git_source ... ok
test rust_plan::tests::no_cargo_capture_binds_declared_vendored_registry_source ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_host_artifact_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_declared_output_before_rustc ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_source_before_rustc ... ok
test rust_plan::tests::no_cargo_capture_blocks_missing_vendored_registry_source ... ok
test rust_plan::tests::rust_unit_execution_blocks_missing_dependency_artifact_before_rustc ... ok
test rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph ... ok
test rust_plan::tests::executes_dependency_chain_from_produced_lib_artifact ... ok

test result: ok. 174 passed; 0 failed; 0 ignored; 0 measured; 707 filtered out; finished in 0.04s

$ cargo test -p mantle --bin mantle cargo_free -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 55 tests
test cargo_free_self_build::tests::child_blocker_fails_when_cargo_guard_was_invoked ... ok
test cargo_free_self_build::tests::child_blocker_accepts_successful_guarded_execution ... ok
test cargo_free_self_build::tests::effective_closure_uses_provider_only_when_manifest_absent ... ok
test cargo_free_self_build::tests::effective_closure_keeps_absent_non_claim_without_provider ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_threads_targets_into_stage_commands ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_explicit_complete_closure_claims ... ok
test cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_provider_supplies_claim ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_requires_source_digest ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_rejects_null_source_digest ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_enforced_closure ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_valid_bounded_bundle ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_bounded_non_claims ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_stage_digest_mismatch ... ok
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test cargo_free_self_build::tests::rustc_wrapper_script_strips_link_self_contained_runtime_args ... ok
test cargo_free_self_build::tests::safe_path_component_replaces_unsafe_path_bytes ... ok
test tests::cargo_free_self_build_rejects_legacy_options ... ok
test cargo_free_self_build::tests::self_build_non_claims_omit_closure_non_claim_when_provider_supplies_claim ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_ambiguous_target_crt ... ok
test cargo_free_self_build::tests::effective_closure_prefers_explicit_manifest_over_provider_status ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_member_name_conflict ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_missing_target_crt ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_target_prefixed_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_unsafe_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_linker_for_collect2 ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_out_dir ... ok
test tests::self_build_cli_accepts_cargo_free_target_triple ... ok
test tests::self_build_cli_accepts_cargo_free_fixed_point_out_dir ... ok
test tests::self_build_cli_rejects_toolchain_closure_without_cargo_free ... ok
test tests::self_build_cli_rejects_fixed_point_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_toolchain_closure_manifest ... ok
test tests::cargo_free_fixed_point_rejects_legacy_options ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_linker_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_c_compiler_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_pkg_config_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_sysroot_leakage ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_runtime_library_digest_mismatch ... ok
test cargo_free_self_build::tests::compatibility_probe_failure_with_explicit_closure_fails_closed ... ok
test cargo_free_self_build::tests::compatibility_probe_uses_receipt_bound_path_for_explicit_closure ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_materializes_declared_runtime_inputs ... ok
test cargo_free_self_build::tests::receipt_bound_path_env_omits_ambient_path_entries ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rewrites_response_file_runtime_inputs ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 826 filtered out; finished in 0.02s

```

## Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal provider-fixed-point-path-normalization --root .
{
  "change": "provider-fixed-point-path-normalization",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "9a7b1256915fda67936a1f4b3bb24f24e9ebc01246b3ce1f3cea08f71491167d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "404b0810b5e1c9c27ea1b15f491ddcbe0e37be0820b0887d1c3dfef2c78da0cd",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design provider-fixed-point-path-normalization --root .
{
  "change": "provider-fixed-point-path-normalization",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "230f74825689735ba778a07d63e84970451891d3a128fe16687ce6f3bb811371",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "a0021c99e58b816a7c404d9f65f944dd40dcb8f3dc30a892d0a82948b9ba090e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks provider-fixed-point-path-normalization --root .
{
  "change": "provider-fixed-point-path-normalization",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b0524d99892990e74a2f35d57bad6183207c1a0636edf664da18b27f909b0080",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "670b2ea8ea54ccb60b513ba0cef0206e43ba882d0e30eecffd1a6c1a7abec9cc",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
