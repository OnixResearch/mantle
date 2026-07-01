# Pure core validation — project-freshness-probes — 2026-07-01

This transcript validates the completed pure-core freshness slice. It does not claim shell adapter or refresh/list-stale integration completion.

## cargo fmt --check -p crunch-project-core -p crunch-project
```text
```

## cargo test -p crunch-project-core
```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-cb773069b4944fda)

running 124 tests
test drift::tests::empty_lock_in_sync_with_empty_generated ... ok
test drift::tests::drifted ... ok
test drift::tests::missing_file ... ok
test fetch_policy::tests::build_fetch_policy_lowers_expected_hash_without_resolution ... ok
test drift::tests::in_sync ... ok
test fetch_policy::tests::build_fetch_policy_rejects_patched_inputs_until_patch_lowering_exists ... ok
test fetch_policy::tests::imported_source_policy_binds_ready_source_state_digest ... ok
test fetch_policy::tests::missing_expected_hash_is_conflicting_for_build_fetch_without_lock ... ok
test fetch_policy::tests::mismatched_source_identity_does_not_satisfy_imported_policy ... ok
test attestation::tests::project_attestation_rejects_missing_locked_patch ... ok
test fetch_policy::tests::offline_network_policy_reports_network_required_without_source_state ... ok
test fetch_policy::tests::policy_plan_classifies_defaults_over_lock_without_fetching ... ok
test fetch_policy::tests::stale_source_state_does_not_satisfy_imported_policy ... ok
test freshness::tests::builtin_observation_normalizes_digest_and_kind ... ok
test freshness::tests::command_probe_validation_accepts_bounded_contract ... ok
test freshness::tests::command_probe_without_argv_is_rejected ... ok
test freshness::tests::empty_observed_value_is_rejected ... ok
test freshness::tests::diagnostics_are_bounded_deterministically ... ok
test freshness::tests::failed_observation_classifies_without_lock_mutation_claim ... ok
test freshness::tests::invalid_template_variable_is_rejected ... ok
test freshness::tests::network_required_observation_is_classified_in_offline_mode ... ok
test freshness::tests::freshness_plan_classifies_selected_stale_and_unchanged ... ok
test freshness::tests::observation_for_unknown_input_is_rejected ... ok
test freshness::tests::observed_network_command_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::observed_network_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::oversized_observed_value_is_rejected ... ok
test freshness::tests::rendered_template_bound_is_enforced ... ok
test generate::tests::escape_special_chars ... ok
test freshness::tests::template_renders_validated_freshness_value ... ok
test generate::tests::fingerprint_deterministic ... ok
test generate::tests::fingerprint_differs ... ok
test generate::tests::generate_empty_lock ... ok
test generate::tests::generate_file_entry ... ok
test generate::tests::generate_git_entry ... ok
test generate::tests::generate_hyphenated_name_quoted ... ok
test generate::tests::generate_includes_patch_metadata ... ok
test generate::tests::generate_quoted_name_escapes_embedded_quote ... ok
test generate::tests::generate_quoted_patch_name_escapes_embedded_quote ... ok
test attestation::tests::project_attestation_records_locked_sources_patches_and_roots ... ok
test generate::tests::generate_remote_patch_metadata ... ok
test generate::tests::generate_panics_on_unlocked_patch_reference ... ok
test generate::tests::needs_quoting_cases ... ok
test generate::tests::generate_with_patches ... ok
test importer::tests::future_adapter_seams_block_recursive_composition_semantics ... ok
test importer::tests::malformed_hash_and_git_identity_fail_closed ... ok
test importer::tests::negative_surfaces_block_without_partial_file_operations ... ok
test lock::tests::empty_lockfile_is_valid ... ok
test lock::tests::lockfile_default_version_is_current ... ok
test lock::tests::locked_kind_git_serde ... ok
test lock::tests::lockfile_detects_empty_hash ... ok
test lock::tests::locked_patch_source_variants ... ok
test lock::tests::lockfile_detects_unlocked_patch ... ok
test lock::tests::lockfile_json_stability ... ok
test lock::tests::lockfile_validates_clean ... ok
test lock::tests::lockfile_entry_with_all_fields ... ok
test manifest::tests::build_fetch_policy_with_patches_is_invalid ... ok
test manifest::tests::empty_name_is_invalid ... ok
test importer::tests::nixtamal_supported_semantics_map_to_manifest_lock_and_inputs_plan ... ok
test manifest::tests::git_reference_default_is_main ... ok
test manifest::tests::fetch_policy_default_is_generation_material ... ok
test manifest::tests::manifest_accepts_compatible_version ... ok
test lock::tests::lockfile_json_roundtrip ... ok
test manifest::tests::manifest_detects_duplicate_names ... ok
test manifest::tests::manifest_detects_undefined_patch ... ok
test importer::tests::existing_file_conflict_blocks_apply_but_keeps_review_plan ... ok
test manifest::tests::manifest_rejects_bad_version ... ok
test manifest::tests::manifest_rejects_incompatible_version ... ok
test manifest::tests::schema_version_accessor ... ok
test manifest::tests::manifest_validates_clean ... ok
test manifest::tests::unknown_fetch_policy_string_is_rejected ... ok
test merge::tests::clean_merge ... ok
test merge::tests::empty_manifest_and_lock_is_clean ... ok
test merge::tests::filter_inputs_finds_existing ... ok
test merge::tests::filter_inputs_reports_missing ... ok
test merge::tests::frozen_without_lock ... ok
test merge::tests::inputs_needing_refresh_includes_missing ... ok
test manifest::tests::manifest_serde_roundtrip ... ok
test merge::tests::inputs_needing_refresh_skips_frozen ... ok
test merge::tests::kind_mismatch ... ok
test merge::tests::orphaned_entries_detected ... ok
test merge::tests::missing_lock_entry ... ok
test attestation::tests::project_attestation_canonicalizes_root_order ... ok
test merge::tests::orphaned_lock_entry ... ok
test mirrors::tests::duplicate_rejected ... ok
test mirrors::tests::empty_url_rejected ... ok
test mirrors::tests::unsupported_scheme_rejected ... ok
test mirrors::tests::url_with_mirrors_order ... ok
test mirrors::tests::url_with_no_mirrors ... ok
test mirrors::tests::valid_mirrors ... ok
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok
test soundness::tests::detects_missing_lock_entry ... ok
test soundness::tests::clean_project_state_is_valid_static_no_network ... ok
test soundness::tests::detects_generated_input_missing_and_stale ... ok
test soundness::tests::detects_hash_algorithm_and_expected_hash_mismatches ... ok
test soundness::tests::detects_kind_and_source_mismatches ... ok
test soundness::tests::explicit_probe_and_trust_modes_label_possible_side_effects ... ok
test soundness::tests::parse_error_report_is_invalid_and_stable ... ok
test upgrade::tests::upgrade_0_9_0_to_1_0_0 ... ok
test soundness::tests::detects_same_kind_source_identity_mismatch ... ok
test soundness::tests::deterministic_issue_order_does_not_follow_manifest_order ... ok
test upgrade::tests::upgrade_future_version_fails ... ok
test soundness::tests::supplemental_policy_trust_retention_and_freshness_facts_are_classified ... ok
test upgrade::tests::upgrade_preserves_all_data ... ok
test version::tests::compatibility_check ... ok
test upgrade::tests::upgrade_unknown_old_version_fails ... ok
test version::tests::current_version_parses ... ok
test soundness::tests::detects_patch_mirror_and_patch_definition_mismatches_as_warnings ... ok
test upgrade::tests::upgrade_current_is_noop ... ok
test soundness::tests::warning_only_orphaned_lock_entry_keeps_report_valid ... ok
test version::tests::parse_roundtrip ... ok
test version::tests::upgrade_check ... ok
test version::tests::parse_rejects_malformed ... ok
test version::tests::serde_roundtrip ... ok

test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_project_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## cargo check -p crunch-project-core --target wasm32-unknown-unknown
```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
```

## cargo test -p crunch-project --test integration_nickel
```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running tests/integration_nickel.rs (/home/brittonr/.cargo-target/debug/deps/integration_nickel-e0fbe62c8ede6969)

running 6 tests
test drift_detection_matches_generation ... ok
test malformed_manifest_loading_via_nickel_eval_fails ... ok
test generated_inputs_importable_from_package_code ... ok
test generated_inputs_is_valid_nickel ... ok
test manifest_loading_via_nickel_eval ... ok
test generated_inputs_field_values_match_lock ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

```

## cairn validate and gates
```text
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 17,
  "valid": true
}
{
  "change": "project-freshness-probes",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e8020f9445af31bbfd905bf0eac5e843cb7604658f0088c86aa882d95ba9ec17",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cab51ff06b9d0913fd3dd1fae7189b0d769b8ef2bf7d957308980efc30f7fae2",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "project-freshness-probes",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "7e5f6326a632ea7bc87243f57a69dc41d48012a251ab63e7ae91f7819d622e84",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "5c5c70ffaf6ccc7f60767af54f5fc9eba2f4d929031924018e79b26a1e39feb8",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "project-freshness-probes",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2ddfd98d63ac675011f100041f795907772059c14b6c2b4eafa1d04b2da48184",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4137a9d895588f05e17cd621d1a5c0b5a78ef350c83c532801b2abf979470df5",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## no-std boundary rail note

A separate same-session run of `./scripts/check-no-std-core.sh` reached the API-shape stage and still failed on pre-existing API-shape findings in crunch-attestation-core, crunch-project-core fetch_policy/importer/soundness, and crunch-release-core. After changing `validate_freshness_probe` to return `FreshnessProbeValidation`, `crates/crunch-project-core/src/freshness.rs` was no longer listed in those API-shape findings.
