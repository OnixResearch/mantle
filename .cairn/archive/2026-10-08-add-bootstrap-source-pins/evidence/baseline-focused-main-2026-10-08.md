# Focused BEFORE leg on published main e24bbbc2 (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins-baseline/baseline-20261008T002628Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-baseline.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T00:26:28Z free=1557428170752 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline head=e24bbbc2f803f59872e2e59370bd8ce0829c918e CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins-baseline CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-baseline.sh
phase=running launcher=23890 pgid=23890
phase=child_exited exit=0 elapsed_s=870 free=1330994024448
phase=final exit=0 group_survivors=0 end=2026-10-08T00:40:58Z elapsed_s=870 free=1330978795520
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
head=e24bbbc2f803f59872e2e59370bd8ce0829c918e
+ nickel --version
nickel-lang-cli nickel 1.17.0 (rev 1320a98)
nickel_version_exit=0 nickel_version_wall_s=0
+ nickel export -I lib --format json --output /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/exports/baseline/cmake-3.31.8-gcc10.json bootstrap/cmake-3.31.8-gcc10.ncl
export_cmake_3_31_8_gcc10_exit=0 export_cmake_3_31_8_gcc10_wall_s=1
+ nickel export -I lib --format json --output /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/exports/baseline/picolibc-1.8.12-src.json bootstrap/picolibc-1.8.12-src.ncl
export_picolibc_1_8_12_src_exit=0 export_picolibc_1_8_12_src_wall_s=0
+ nickel export -I lib --format json --output /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/exports/baseline/picolibc-1.8.12-diagnostic.json bootstrap/picolibc-1.8.12-diagnostic.ncl
export_picolibc_1_8_12_diagnostic_exit=0 export_picolibc_1_8_12_diagnostic_wall_s=0
+ cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/cmake-3.31.8-gcc10.ncl bootstrap/picolibc-1.8.12-src.ncl bootstrap/picolibc-1.8.12-diagnostic.ncl
source-pin audit: 3 files, 2 fetch blocks, 0 issues
source_pin_script_recipes_exit=0 source_pin_script_recipes_wall_s=1
+ bash -o pipefail -c cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/*.ncl | tail -5
--- source pin issues ---
FAIL  bootstrap/autoconf-gcc-factory.ncl:10: [bad-hash-format/fetchTarball] fetchTarball hash is not SRI format: 'spec.source_hash'
FAIL  bootstrap/automake-gcc-factory.ncl:7: [bad-hash-format/fetchTarball] fetchTarball hash is not SRI format: 'spec.source_hash'
---
source-pin audit: 185 files, 140 fetch blocks, 2 issues
source_pin_script_all_exit=1 source_pin_script_all_wall_s=0
+ cargo test -p crunch-project-core --lib
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.44s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/deps/crunch_project_core-5f776a7d4a366768)

running 161 tests
test drift::tests::empty_lock_in_sync_with_empty_generated ... ok
test drift::tests::drifted ... ok
test attestation::tests::project_attestation_rejects_missing_locked_patch ... ok
test drift::tests::missing_file ... ok
test drift::tests::in_sync ... ok
test fetch_policy::tests::build_fetch_policy_lowers_expected_hash_without_resolution ... ok
test fetch_policy::tests::build_fetch_policy_rejects_patched_inputs_until_patch_lowering_exists ... ok
test fetch_policy::tests::imported_source_policy_binds_ready_source_state_digest ... ok
test fetch_policy::tests::missing_expected_hash_is_conflicting_for_build_fetch_without_lock ... ok
test fetch_policy::tests::mismatched_source_identity_does_not_satisfy_imported_policy ... ok
test fetch_policy::tests::non_generation_vcs_policy_requires_proven_identity_for_ambiguous_selector ... ok
test fetch_policy::tests::offline_network_policy_reports_network_required_without_source_state ... ok
test fetch_policy::tests::policy_plan_classifies_defaults_over_lock_without_fetching ... ok
test attestation::tests::project_attestation_records_locked_sources_patches_and_roots ... ok
test fetch_policy::tests::stale_source_state_does_not_satisfy_imported_policy ... ok
test fetch_policy::tests::vcs_source_identity_uses_locked_native_identity ... ok
test filegen::tests::apply_plan_detects_plan_drift ... ok
test filegen::tests::symlink_materialization_accepts_matching_managed_symlink ... ok
test filegen::tests::plan_create_update_unchanged_and_stale_generated_files ... ok
test filegen::tests::target_escape_existing_conflict_invalid_contract_and_bad_export_digest_fail_closed ... ok
test filegen::tests::typed_valid_content_records_contract_identity ... ok
test freshness::tests::builtin_observation_normalizes_digest_and_kind ... ok
test freshness::tests::command_probe_validation_accepts_bounded_contract ... ok
test freshness::tests::command_probe_without_argv_is_rejected ... ok
test freshness::tests::diagnostics_are_bounded_deterministically ... ok
test freshness::tests::empty_observed_value_is_rejected ... ok
test attestation::tests::project_attestation_canonicalizes_root_order ... ok
test freshness::tests::failed_observation_classifies_without_lock_mutation_claim ... ok
test freshness::tests::freshness_plan_classifies_selected_stale_and_unchanged ... ok
test freshness::tests::invalid_template_variable_is_rejected ... ok
test freshness::tests::network_required_observation_is_classified_in_offline_mode ... ok
test freshness::tests::observation_for_unknown_input_is_rejected ... ok
test freshness::tests::observed_network_command_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::observed_network_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::oversized_observed_value_is_rejected ... ok
test freshness::tests::rendered_template_bound_is_enforced ... ok
test freshness::tests::template_renders_validated_freshness_value ... ok
test generate::tests::escape_special_chars ... ok
test generate::tests::fingerprint_deterministic ... ok
test generate::tests::fingerprint_differs ... ok
test generate::tests::generate_empty_lock ... ok
test generate::tests::generate_file_entry ... ok
test generate::tests::generate_git_entry ... ok
test generate::tests::generate_hyphenated_name_quoted ... ok
test generate::tests::generate_includes_patch_metadata ... ok
test generate::tests::generate_panics_on_unlocked_patch_reference ... ok
test generate::tests::generate_quoted_name_escapes_embedded_quote ... ok
test generate::tests::generate_quoted_patch_name_escapes_embedded_quote ... ok
test generate::tests::generate_remote_patch_metadata ... ok
test generate::tests::generate_vcs_entries_include_native_lock_metadata ... ok
test generate::tests::generate_with_patches ... ok
test generate::tests::needs_quoting_cases ... ok
test importer::tests::future_adapter_seams_block_recursive_composition_semantics ... ok
test importer::tests::malformed_hash_and_git_identity_fail_closed ... ok
test importer::tests::negative_surfaces_block_without_partial_file_operations ... ok
test lock::tests::empty_lockfile_is_valid ... ok
test importer::tests::existing_file_conflict_blocks_apply_but_keeps_review_plan ... ok
test lock::tests::lock_validation_rejects_unproven_vcs_identity ... ok
test lock::tests::locked_kind_git_serde ... ok
test importer::tests::nixtamal_supported_semantics_map_to_manifest_lock_and_inputs_plan ... ok
test lock::tests::locked_patch_source_variants ... ok
test lock::tests::lockfile_default_version_is_current ... ok
test lock::tests::lockfile_detects_empty_hash ... ok
test lock::tests::lockfile_detects_unlocked_patch ... ok
test lock::tests::locked_vcs_kinds_roundtrip_and_validate ... ok
test filegen::tests::over_limit_inputs_return_blockers_without_planning ... ok
test lock::tests::lockfile_entry_with_all_fields ... ok
test lock::tests::lockfile_json_stability ... ok
test lock::tests::lockfile_validates_clean ... ok
test lock::tests::lockfile_json_roundtrip ... ok
test manifest::tests::build_fetch_policy_with_patches_is_invalid ... ok
test manifest::tests::empty_name_is_invalid ... ok
test manifest::tests::fetch_policy_default_is_generation_material ... ok
test manifest::tests::git_reference_default_is_main ... ok
test manifest::tests::invalid_retention_generation_limit_is_rejected ... ok
test manifest::tests::manifest_accepts_compatible_version ... ok
test manifest::tests::manifest_detects_duplicate_names ... ok
test manifest::tests::manifest_rejects_bad_version ... ok
test manifest::tests::manifest_rejects_incompatible_version ... ok
test manifest::tests::manifest_detects_undefined_patch ... ok
test manifest::tests::manifest_validates_clean ... ok
test manifest::tests::retention_defaults_to_untracked ... ok
test manifest::tests::manifest_serde_roundtrip ... ok
test manifest::tests::schema_version_accessor ... ok
test manifest::tests::unknown_fetch_policy_string_is_rejected ... ok
test manifest::tests::vcs_manifest_validation_rejects_forge_shortcuts_and_empty_selectors ... ok
test merge::tests::clean_merge ... ok
test merge::tests::empty_manifest_and_lock_is_clean ... ok
test manifest::tests::vcs_input_selectors_roundtrip ... ok
test merge::tests::filter_inputs_finds_existing ... ok
test merge::tests::filter_inputs_reports_missing ... ok
test merge::tests::frozen_without_lock ... ok
test merge::tests::inputs_needing_refresh_includes_missing ... ok
test merge::tests::inputs_needing_refresh_skips_frozen ... ok
test merge::tests::kind_mismatch ... ok
test merge::tests::missing_lock_entry ... ok
test merge::tests::orphaned_entries_detected ... ok
test merge::tests::orphaned_lock_entry ... ok
test mirrors::tests::duplicate_rejected ... ok
test mirrors::tests::empty_url_rejected ... ok
test mirrors::tests::unsupported_scheme_rejected ... ok
test mirrors::tests::url_with_mirrors_order ... ok
test mirrors::tests::url_with_no_mirrors ... ok
test mirrors::tests::valid_mirrors ... ok
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::apply_outcomes_rejects_trusted_patch_without_evidence ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::freshness_network_required_decision_fails_closed_for_refresh ... ok
test refresh::tests::freshness_unchanged_decision_skips_resolution_work ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_inputs_attaches_verified_trust_to_updated_lock ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok
test refresh::tests::refresh_inputs_rejects_trusted_input_without_evidence ... ok
test retention::tests::current_policy_plans_root_until_marker_exists ... ok
test retention::tests::input_override_beats_project_default ... ok
test retention::tests::generation_policy_keeps_bounded_newest_history ... ok
test retention::tests::lock_digest_binding_changes_when_hash_changes ... ok
test retention::tests::invalid_generation_limit_and_interrupted_root_are_not_durable ... ok
test retention::tests::missing_root_marker_is_reported_as_absent ... ok
test retention::tests::root_for_unknown_input_is_rejected ... ok
test retention::tests::mismatched_lock_digest_does_not_satisfy_current_policy ... ok
test retention::tests::stale_root_outside_generation_window_is_removed_after_create ... ok
test retention::tests::untracked_input_is_gc_eligible_and_does_not_retain_root ... ok
test soundness::tests::detects_generated_input_missing_and_stale ... ok
test soundness::tests::clean_project_state_is_valid_static_no_network ... ok
test soundness::tests::detects_hash_algorithm_and_expected_hash_mismatches ... ok
test soundness::tests::detects_kind_and_source_mismatches ... ok
test soundness::tests::detects_missing_lock_entry ... ok
test soundness::tests::detects_patch_mirror_and_patch_definition_mismatches_as_warnings ... ok
test soundness::tests::detects_same_kind_source_identity_mismatch ... ok
test soundness::tests::explicit_probe_and_trust_modes_label_possible_side_effects ... ok
test soundness::tests::deterministic_issue_order_does_not_follow_manifest_order ... ok
test soundness::tests::parse_error_report_is_invalid_and_stable ... ok
test soundness::tests::supplemental_policy_trust_retention_and_freshness_facts_are_classified ... ok
test soundness::tests::warning_only_orphaned_lock_entry_keeps_report_valid ... ok
test trust::tests::locked_trust_validation_rejects_overbroad_claim ... ok
test trust::tests::trust_policy_accepts_matching_required_signer ... ok
test trust::tests::locked_trust_validation_rejects_wrong_digest_binding ... ok
test trust::tests::quorum_accepts_independent_trusted_signers ... ok
test trust::tests::trust_policy_rejects_malformed_key_refs_and_unknown_verifier ... ok
test trust::tests::trust_policy_rejects_untrusted_signer_and_bad_quorum ... ok
test trust::tests::trust_policy_requires_declared_evidence_and_bounds_quorum ... ok
test upgrade::tests::upgrade_0_9_0_to_1_0_0 ... ok
test upgrade::tests::upgrade_current_is_noop ... ok
test upgrade::tests::upgrade_future_version_fails ... ok
test upgrade::tests::upgrade_preserves_all_data ... ok
test upgrade::tests::upgrade_unknown_old_version_fails ... ok
test version::tests::compatibility_check ... ok
test version::tests::current_version_parses ... ok
test version::tests::parse_rejects_malformed ... ok
test version::tests::parse_roundtrip ... ok
test version::tests::serde_roundtrip ... ok
test version::tests::upgrade_check ... ok

test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

core_lib_exit=0 core_lib_wall_s=20
+ cargo test -p mantle-portable-client-core --lib
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.27s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/deps/mantle_portable_client_core-6e26e0d8930ed73c)

running 7 tests
test tests::command_inventory_is_complete_and_unique ... ok
test tests::darwin_local_build_is_rejected_before_execution ... ok
test tests::darwin_remote_build_is_admitted_without_local_execution ... ok
test tests::portable_plan_keeps_client_and_target_platforms_separate ... ok
test tests::portable_plan_has_no_fixed_nix_store_assumption ... ok
test tests::portable_plan_requires_remote_capability_and_trust ... ok
test tests::raw_frontend_payload_is_rejected ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

portable_lib_exit=0 portable_lib_wall_s=1
+ ./scripts/check-operator-command-contract.sh
config/operator-command-descriptors.json /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins-baseline/nix-shell.Se6smU/tmp.gX9qXcMsDO/descriptors.json differ: byte 74134, line 3461
operator_contract_exit=1 operator_contract_wall_s=519
+ cargo test -p mantle --test operator_diagnostics
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5m 10s
     Running tests/operator_diagnostics.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/deps/operator_diagnostics-44b0508e7b7f3ff7)

running 18 tests
test operator_contract_generator_runs_positive_and_negative_self_test ... ok
test doctor_json_failure_reports_missing_prerequisite ... ok
test doctor_default_profile_is_build_and_does_not_mutate_paths ... ok
test operator_contract_checked_files_match_clap_and_policy ... FAILED
test operator_contract_exports_sorted_public_clap_paths ... ok
test doctor_json_output_reports_selected_profile ... ok
test doctor_explicit_self_build_profile_checks_nightly_visibility ... ok
test build_json_preflight_failure_omits_saved_log_path ... ok
test build_plan_json_reports_action_schema ... ok
test build_plan_reports_build_for_uncached_root ... ok
test build_human_failure_reports_log_persistence_failure_without_saved_log_path ... ok
test build_json_failure_envelope_includes_saved_log_path ... ok
test doctor_failure_names_missing_prerequisite_and_profile ... ok
test build_without_plan_still_executes_fetchurl ... ok
test build_human_failure_summary_matches_json_facts ... ok
test build_json_success_reports_log_persistence_failure_without_fake_log_file ... ok
test build_plan_reports_preflight_error_for_missing_store_dir ... ok
test build_plan_reports_cached_after_fetchurl_build ... ok

failures:

---- operator_contract_checked_files_match_clap_and_policy stdout ----

thread 'operator_contract_checked_files_match_clap_and_policy' (710514) panicked at /nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5:
Unexpected failure.
code=3
stderr=```"error: operator command contract: flag-drift: source bundle export\n"```
command=`cd "/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline" && "/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/crunch" "__operator-contract" "--mode" "check"`
code=3
stdout=""
stderr="error: operator command contract: flag-drift: source bundle export\n"

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    operator_contract_checked_files_match_clap_and_policy

test result: FAILED. 17 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.05s

error: test failed, to rerun pass `-p mantle --test operator_diagnostics`
operator_diag_exit=101 operator_diag_wall_s=322
```
