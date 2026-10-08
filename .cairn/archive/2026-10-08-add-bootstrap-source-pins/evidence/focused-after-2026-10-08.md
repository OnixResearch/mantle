# Focused AFTER leg on finish/add-bootstrap-source-pins (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/after-20261008T003929Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-after.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T00:39:29Z free=1371108782080 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=e24bbbc2f803f59872e2e59370bd8ce0829c918e CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-after.sh
phase=running launcher=683373 pgid=683373
phase=child_exited exit=0 elapsed_s=810 free=1134192566272
phase=final exit=0 group_survivors=0 end=2026-10-08T00:52:59Z elapsed_s=810 free=1134191071232
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
head=e24bbbc2f803f59872e2e59370bd8ce0829c918e
+ python3 --version
/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-after.sh: line 7: python3: command not found
python_version_exit=127 python_version_wall_s=0
+ nickel --version
nickel-lang-cli nickel 1.17.0 (rev 1320a98)
nickel_version_exit=0 nickel_version_wall_s=0
+ nickel typecheck config/operator-surfaces.ncl
ncl_typecheck_exit=0 ncl_typecheck_wall_s=0
+ bash -c nickel export --format json config/operator-surfaces.ncl > config/operator-surfaces.json
ncl_export_surfaces_exit=0 ncl_export_surfaces_wall_s=0
+ cargo build -p mantle --bin mantle --bin generate-operator-command-contract
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 00s
build_bins_exit=0 build_bins_wall_s=61
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle __operator-contract --mode raw-descriptors > config/operator-command-descriptors.json
gen_descriptors_exit=0 gen_descriptors_wall_s=1
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle __operator-contract --mode catalog > config/operator-command-catalog.json
gen_catalog_exit=0 gen_catalog_wall_s=0
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle __operator-contract --mode reference > docs/generated/operator-command-reference.md
gen_reference_exit=0 gen_reference_wall_s=0
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle __operator-contract --mode workflow > docs/generated/canonical-operator-workflow.md
gen_workflow_exit=0 gen_workflow_wall_s=1
MM .cairn/changes/add-bootstrap-source-pins/design.md
A  .cairn/changes/add-bootstrap-source-pins/evidence/baseline-2026-09-30.md
A  .cairn/changes/add-bootstrap-source-pins/evidence/cairn-tasks-gate-2026-10-01.txt
A  .cairn/changes/add-bootstrap-source-pins/evidence/implementation-2026-10-01.md
A  .cairn/changes/add-bootstrap-source-pins/evidence/shared-root-pin-tests-2026-10-01.txt
A  .cairn/changes/add-bootstrap-source-pins/evidence/shared-root-pin-tests-current-tree-2026-10-01.txt
M  .cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md
M  .cairn/changes/add-bootstrap-source-pins/tasks.md
 M Cargo.lock
 M Cargo.toml
A  adr/0086-separate-bootstrap-source-pins-from-catalog-update-policy.md
 M adr/README.md
M  bootstrap/cmake-3.31.8-gcc10.ncl
M  bootstrap/picolibc-1.8.12-diagnostic.ncl
M  bootstrap/picolibc-1.8.12-src.ncl
A  bootstrap/pins/cmake.toml
A  bootstrap/pins/generate_readers.py
A  bootstrap/pins/generated/cmake.json
A  bootstrap/pins/generated/picolibc.json
A  bootstrap/pins/picolibc.toml
 M config/operator-command-catalog.json
 M config/operator-command-descriptors.json
 M config/operator-surfaces.json
 M config/operator-surfaces.ncl
A  crates/crunch-project-core/src/bootstrap_pins.rs
M  crates/crunch-project-core/src/lib.rs
 M crates/mantle-portable-client-core/src/lib.rs
MM docs/bootstrap-stage0-inventory.md
 M docs/generated/operator-command-reference.md
D  scripts/check-bootstrap-source-pins.rs
AM src/bootstrap_pin_cmd.rs
 M src/main.rs
+ nickel export -I lib --format json --output /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/exports/after/cmake-3.31.8-gcc10.json bootstrap/cmake-3.31.8-gcc10.ncl
export_cmake_3_31_8_gcc10_exit=0 export_cmake_3_31_8_gcc10_wall_s=0
+ nickel typecheck -I lib bootstrap/cmake-3.31.8-gcc10.ncl
typecheck_cmake_3_31_8_gcc10_exit=0 typecheck_cmake_3_31_8_gcc10_wall_s=0
+ nickel export -I lib --format json --output /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/exports/after/picolibc-1.8.12-src.json bootstrap/picolibc-1.8.12-src.ncl
export_picolibc_1_8_12_src_exit=0 export_picolibc_1_8_12_src_wall_s=1
+ nickel typecheck -I lib bootstrap/picolibc-1.8.12-src.ncl
typecheck_picolibc_1_8_12_src_exit=0 typecheck_picolibc_1_8_12_src_wall_s=0
+ nickel export -I lib --format json --output /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/exports/after/picolibc-1.8.12-diagnostic.json bootstrap/picolibc-1.8.12-diagnostic.ncl
export_picolibc_1_8_12_diagnostic_exit=0 export_picolibc_1_8_12_diagnostic_wall_s=0
+ nickel typecheck -I lib bootstrap/picolibc-1.8.12-diagnostic.ncl
typecheck_picolibc_1_8_12_diagnostic_exit=0 typecheck_picolibc_1_8_12_diagnostic_wall_s=0
+ python3 bootstrap/pins/generate_readers.py --check
/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-after.sh: line 7: python3: command not found
readers_check_exit=127 readers_check_wall_s=0
+ python3 bootstrap/pins/generate_readers.py --check-recipes
/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-after.sh: line 7: python3: command not found
readers_check_recipes_exit=127 readers_check_recipes_wall_s=0
+ cargo test -p crunch-project-core --lib bootstrap_pins
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.54s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/deps/crunch_project_core-5f776a7d4a366768)

running 2 tests
test bootstrap_pins::tests::decides_and_seals_reviewable_plan ... ok
test bootstrap_pins::tests::validates_roundtrip_and_rejects_unknown_missing_and_bad_hash ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.00s

core_pins_exit=0 core_pins_wall_s=20
+ cargo test -p crunch-project-core --lib
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.44s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/deps/crunch_project_core-5f776a7d4a366768)

running 163 tests
test attestation::tests::project_attestation_rejects_missing_locked_patch ... ok
test drift::tests::empty_lock_in_sync_with_empty_generated ... ok
test drift::tests::drifted ... ok
test bootstrap_pins::tests::decides_and_seals_reviewable_plan ... ok
test drift::tests::missing_file ... ok
test fetch_policy::tests::build_fetch_policy_lowers_expected_hash_without_resolution ... ok
test fetch_policy::tests::build_fetch_policy_rejects_patched_inputs_until_patch_lowering_exists ... ok
test bootstrap_pins::tests::validates_roundtrip_and_rejects_unknown_missing_and_bad_hash ... ok
test drift::tests::in_sync ... ok
test fetch_policy::tests::mismatched_source_identity_does_not_satisfy_imported_policy ... ok
test attestation::tests::project_attestation_records_locked_sources_patches_and_roots ... ok
test fetch_policy::tests::missing_expected_hash_is_conflicting_for_build_fetch_without_lock ... ok
test fetch_policy::tests::imported_source_policy_binds_ready_source_state_digest ... ok
test fetch_policy::tests::non_generation_vcs_policy_requires_proven_identity_for_ambiguous_selector ... ok
test fetch_policy::tests::offline_network_policy_reports_network_required_without_source_state ... ok
test fetch_policy::tests::policy_plan_classifies_defaults_over_lock_without_fetching ... ok
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
test attestation::tests::project_attestation_canonicalizes_root_order ... ok
test freshness::tests::diagnostics_are_bounded_deterministically ... ok
test freshness::tests::empty_observed_value_is_rejected ... ok
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
test generate::tests::needs_quoting_cases ... ok
test generate::tests::generate_with_patches ... ok
test generate::tests::generate_vcs_entries_include_native_lock_metadata ... ok
test importer::tests::future_adapter_seams_block_recursive_composition_semantics ... ok
test importer::tests::malformed_hash_and_git_identity_fail_closed ... ok
test lock::tests::empty_lockfile_is_valid ... ok
test importer::tests::negative_surfaces_block_without_partial_file_operations ... ok
test importer::tests::existing_file_conflict_blocks_apply_but_keeps_review_plan ... ok
test lock::tests::lock_validation_rejects_unproven_vcs_identity ... ok
test lock::tests::locked_kind_git_serde ... ok
test lock::tests::locked_patch_source_variants ... ok
test lock::tests::lockfile_default_version_is_current ... ok
test importer::tests::nixtamal_supported_semantics_map_to_manifest_lock_and_inputs_plan ... ok
test lock::tests::lockfile_detects_empty_hash ... ok
test lock::tests::lockfile_detects_unlocked_patch ... ok
test lock::tests::locked_vcs_kinds_roundtrip_and_validate ... ok
test lock::tests::lockfile_json_stability ... ok
test lock::tests::lockfile_entry_with_all_fields ... ok
test lock::tests::lockfile_validates_clean ... ok
test manifest::tests::build_fetch_policy_with_patches_is_invalid ... ok
test lock::tests::lockfile_json_roundtrip ... ok
test manifest::tests::empty_name_is_invalid ... ok
test filegen::tests::over_limit_inputs_return_blockers_without_planning ... ok
test manifest::tests::git_reference_default_is_main ... ok
test manifest::tests::fetch_policy_default_is_generation_material ... ok
test manifest::tests::invalid_retention_generation_limit_is_rejected ... ok
test manifest::tests::manifest_accepts_compatible_version ... ok
test manifest::tests::manifest_detects_duplicate_names ... ok
test manifest::tests::manifest_detects_undefined_patch ... ok
test manifest::tests::manifest_rejects_bad_version ... ok
test manifest::tests::manifest_rejects_incompatible_version ... ok
test manifest::tests::manifest_validates_clean ... ok
test manifest::tests::retention_defaults_to_untracked ... ok
test manifest::tests::schema_version_accessor ... ok
test manifest::tests::unknown_fetch_policy_string_is_rejected ... ok
test manifest::tests::manifest_serde_roundtrip ... ok
test manifest::tests::vcs_manifest_validation_rejects_forge_shortcuts_and_empty_selectors ... ok
test merge::tests::empty_manifest_and_lock_is_clean ... ok
test merge::tests::clean_merge ... ok
test manifest::tests::vcs_input_selectors_roundtrip ... ok
test merge::tests::filter_inputs_finds_existing ... ok
test merge::tests::filter_inputs_reports_missing ... ok
test merge::tests::frozen_without_lock ... ok
test merge::tests::inputs_needing_refresh_includes_missing ... ok
test merge::tests::inputs_needing_refresh_skips_frozen ... ok
test merge::tests::missing_lock_entry ... ok
test merge::tests::orphaned_entries_detected ... ok
test merge::tests::orphaned_lock_entry ... ok
test mirrors::tests::duplicate_rejected ... ok
test mirrors::tests::empty_url_rejected ... ok
test mirrors::tests::unsupported_scheme_rejected ... ok
test merge::tests::kind_mismatch ... ok
test mirrors::tests::url_with_mirrors_order ... ok
test mirrors::tests::url_with_no_mirrors ... ok
test mirrors::tests::valid_mirrors ... ok
test refresh::tests::apply_outcomes_rejects_trusted_patch_without_evidence ... ok
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::freshness_network_required_decision_fails_closed_for_refresh ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
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
test retention::tests::mismatched_lock_digest_does_not_satisfy_current_policy ... ok
test retention::tests::missing_root_marker_is_reported_as_absent ... ok
test retention::tests::root_for_unknown_input_is_rejected ... ok
test retention::tests::stale_root_outside_generation_window_is_removed_after_create ... ok
test retention::tests::untracked_input_is_gc_eligible_and_does_not_retain_root ... ok
test soundness::tests::clean_project_state_is_valid_static_no_network ... ok
test soundness::tests::detects_generated_input_missing_and_stale ... ok
test soundness::tests::detects_hash_algorithm_and_expected_hash_mismatches ... ok
test soundness::tests::detects_missing_lock_entry ... ok
test soundness::tests::detects_kind_and_source_mismatches ... ok
test soundness::tests::detects_patch_mirror_and_patch_definition_mismatches_as_warnings ... ok
test soundness::tests::detects_same_kind_source_identity_mismatch ... ok
test soundness::tests::deterministic_issue_order_does_not_follow_manifest_order ... ok
test soundness::tests::explicit_probe_and_trust_modes_label_possible_side_effects ... ok
test soundness::tests::parse_error_report_is_invalid_and_stable ... ok
test soundness::tests::supplemental_policy_trust_retention_and_freshness_facts_are_classified ... ok
test trust::tests::locked_trust_validation_rejects_overbroad_claim ... ok
test soundness::tests::warning_only_orphaned_lock_entry_keeps_report_valid ... ok
test trust::tests::locked_trust_validation_rejects_wrong_digest_binding ... ok
test trust::tests::trust_policy_accepts_matching_required_signer ... ok
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

test result: ok. 163 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

core_lib_exit=0 core_lib_wall_s=1
+ cargo test -p mantle-portable-client-core --lib
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.10s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/deps/mantle_portable_client_core-6e26e0d8930ed73c)

running 7 tests
test tests::command_inventory_is_complete_and_unique ... ok
test tests::darwin_local_build_is_rejected_before_execution ... ok
test tests::darwin_remote_build_is_admitted_without_local_execution ... ok
test tests::portable_plan_has_no_fixed_nix_store_assumption ... ok
test tests::portable_plan_keeps_client_and_target_platforms_separate ... ok
test tests::portable_plan_requires_remote_capability_and_trust ... ok
test tests::raw_frontend_payload_is_rejected ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

portable_lib_exit=0 portable_lib_wall_s=3
+ cargo test -p mantle --bin mantle bootstrap_pin_cmd::tests:: -- --nocapture
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7m 46s
     Running unittests src/main.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/deps/mantle-dee36004035187be)

running 3 tests
test bootstrap_pin_cmd::tests::host_bounds_use_parsed_https_authority ... ok
test bootstrap_pin_cmd::tests::unresolved_source_blocks_entire_mixed_candidate_plan_without_writes ... ok
test bootstrap_pin_cmd::tests::publication_rolls_back_first_pin_when_second_publication_fails ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2549 filtered out; finished in 0.21s

cli_pins_exit=0 cli_pins_wall_s=477
+ ./scripts/check-operator-command-contract.sh
operator command contract generator self-test: PASS
operator command contract: PASS (commands=179)
operator_contract_exit=0 operator_contract_wall_s=5
+ cargo test -p mantle --test operator_diagnostics
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 39s
     Running tests/operator_diagnostics.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/deps/operator_diagnostics-05ba9de61392a686)

running 18 tests
test operator_contract_generator_runs_positive_and_negative_self_test ... ok
test doctor_json_failure_reports_missing_prerequisite ... ok
test doctor_default_profile_is_build_and_does_not_mutate_paths ... ok
test doctor_explicit_self_build_profile_checks_nightly_visibility ... ok
test doctor_json_output_reports_selected_profile ... ok
test operator_contract_exports_sorted_public_clap_paths ... ok
test operator_contract_checked_files_match_clap_and_policy ... ok
test doctor_failure_names_missing_prerequisite_and_profile ... ok
test build_json_preflight_failure_omits_saved_log_path ... ok
test build_plan_reports_build_for_uncached_root ... ok
test build_plan_json_reports_action_schema ... ok
test build_human_failure_reports_log_persistence_failure_without_saved_log_path ... ok
test build_json_failure_envelope_includes_saved_log_path ... ok
test build_plan_reports_preflight_error_for_missing_store_dir ... ok
test build_json_success_reports_log_persistence_failure_without_fake_log_file ... ok
test build_without_plan_still_executes_fetchurl ... ok
test build_human_failure_summary_matches_json_facts ... ok
test build_plan_reports_cached_after_fetchurl_build ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.87s

operator_diag_exit=0 operator_diag_wall_s=169
done
```
