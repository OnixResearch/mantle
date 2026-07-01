# Implementation Validation — input-fetch-policy

Date: 2026-07-01

This transcript was generated after implementing the fetch-policy data model,
refresh/generation/source-bundle threading, deterministic source-state identity
matching, and focused CLI coverage.

## rustfmt-check — PASS

Command:

```text
nix develop -c cargo fmt --check
```

Output:

```text

```

## core-lib-tests — PASS

Command:

```text
nix develop -c cargo test -p crunch-project-core --lib
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 109 tests
test drift::tests::empty_lock_in_sync_with_empty_generated ... ok
test drift::tests::drifted ... ok
test drift::tests::in_sync ... ok
test drift::tests::missing_file ... ok
test fetch_policy::tests::build_fetch_policy_lowers_expected_hash_without_resolution ... ok
test attestation::tests::project_attestation_rejects_missing_locked_patch ... ok
test fetch_policy::tests::build_fetch_policy_rejects_patched_inputs_until_patch_lowering_exists ... ok
test fetch_policy::tests::missing_expected_hash_is_conflicting_for_build_fetch_without_lock ... ok
test fetch_policy::tests::mismatched_source_identity_does_not_satisfy_imported_policy ... ok
test fetch_policy::tests::imported_source_policy_binds_ready_source_state_digest ... ok
test fetch_policy::tests::offline_network_policy_reports_network_required_without_source_state ... ok
test fetch_policy::tests::policy_plan_classifies_defaults_over_lock_without_fetching ... ok
test fetch_policy::tests::stale_source_state_does_not_satisfy_imported_policy ... ok
test generate::tests::escape_special_chars ... ok
test generate::tests::fingerprint_deterministic ... ok
test generate::tests::fingerprint_differs ... ok
test generate::tests::generate_empty_lock ... ok
test generate::tests::generate_file_entry ... ok
test generate::tests::generate_git_entry ... ok
test generate::tests::generate_hyphenated_name_quoted ... ok
test generate::tests::generate_includes_patch_metadata ... ok
test attestation::tests::project_attestation_records_locked_sources_patches_and_roots ... ok
test generate::tests::generate_quoted_name_escapes_embedded_quote ... ok
test generate::tests::generate_panics_on_unlocked_patch_reference ... ok
test generate::tests::generate_quoted_patch_name_escapes_embedded_quote ... ok
test generate::tests::generate_remote_patch_metadata ... ok
test generate::tests::generate_with_patches ... ok
test generate::tests::needs_quoting_cases ... ok
test importer::tests::future_adapter_seams_block_recursive_composition_semantics ... ok
test importer::tests::malformed_hash_and_git_identity_fail_closed ... ok
test importer::tests::negative_surfaces_block_without_partial_file_operations ... ok
test lock::tests::empty_lockfile_is_valid ... ok
test lock::tests::lockfile_default_version_is_current ... ok
test lock::tests::locked_patch_source_variants ... ok
test lock::tests::locked_kind_git_serde ... ok
test lock::tests::lockfile_detects_empty_hash ... ok
test importer::tests::existing_file_conflict_blocks_apply_but_keeps_review_plan ... ok
test lock::tests::lockfile_detects_unlocked_patch ... ok
test attestation::tests::project_attestation_canonicalizes_root_order ... ok
test importer::tests::nixtamal_supported_semantics_map_to_manifest_lock_and_inputs_plan ... ok
test lock::tests::lockfile_entry_with_all_fields ... ok
test manifest::tests::git_reference_default_is_main ... ok
test manifest::tests::manifest_accepts_compatible_version ... ok
test lock::tests::lockfile_validates_clean ... ok
test lock::tests::lockfile_json_roundtrip ... ok
test manifest::tests::build_fetch_policy_with_patches_is_invalid ... ok
test manifest::tests::manifest_detects_duplicate_names ... ok
test manifest::tests::manifest_detects_undefined_patch ... ok
test manifest::tests::manifest_rejects_incompatible_version ... ok
test lock::tests::lockfile_json_stability ... ok
test manifest::tests::manifest_validates_clean ... ok
test manifest::tests::manifest_rejects_bad_version ... ok
test manifest::tests::schema_version_accessor ... ok
test merge::tests::empty_manifest_and_lock_is_clean ... ok
test merge::tests::clean_merge ... ok
test manifest::tests::manifest_serde_roundtrip ... ok
test merge::tests::filter_inputs_finds_existing ... ok
test merge::tests::filter_inputs_reports_missing ... ok
test manifest::tests::empty_name_is_invalid ... ok
test merge::tests::frozen_without_lock ... ok
test merge::tests::inputs_needing_refresh_includes_missing ... ok
test merge::tests::inputs_needing_refresh_skips_frozen ... ok
test merge::tests::kind_mismatch ... ok
test manifest::tests::fetch_policy_default_is_generation_material ... ok
test merge::tests::missing_lock_entry ... ok
test merge::tests::orphaned_lock_entry ... ok
test merge::tests::orphaned_entries_detected ... ok
test mirrors::tests::duplicate_rejected ... ok
test mirrors::tests::empty_url_rejected ... ok
test mirrors::tests::unsupported_scheme_rejected ... ok
test mirrors::tests::url_with_mirrors_order ... ok
test mirrors::tests::url_with_no_mirrors ... ok
test mirrors::tests::valid_mirrors ... ok
test manifest::tests::unknown_fetch_policy_string_is_rejected ... ok
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok
test soundness::tests::clean_project_state_is_valid_static_no_network ... ok
test soundness::tests::detects_generated_input_missing_and_stale ... ok
test soundness::tests::detects_missing_lock_entry ... ok
test soundness::tests::detects_hash_algorithm_and_expected_hash_mismatches ... ok
test soundness::tests::detects_kind_and_source_mismatches ... ok
test soundness::tests::explicit_probe_and_trust_modes_label_possible_side_effects ... ok
test soundness::tests::detects_same_kind_source_identity_mismatch ... ok
test soundness::tests::parse_error_report_is_invalid_and_stable ... ok
test soundness::tests::deterministic_issue_order_does_not_follow_manifest_order ... ok
test soundness::tests::detects_patch_mirror_and_patch_definition_mismatches_as_warnings ... ok
test soundness::tests::supplemental_policy_trust_retention_and_freshness_facts_are_classified ... ok
test upgrade::tests::upgrade_0_9_0_to_1_0_0 ... ok
test upgrade::tests::upgrade_current_is_noop ... ok
test soundness::tests::warning_only_orphaned_lock_entry_keeps_report_valid ... ok
test upgrade::tests::upgrade_future_version_fails ... ok
test upgrade::tests::upgrade_preserves_all_data ... ok
test upgrade::tests::upgrade_unknown_old_version_fails ... ok
test version::tests::compatibility_check ... ok
test version::tests::current_version_parses ... ok
test version::tests::parse_rejects_malformed ... ok
test version::tests::parse_roundtrip ... ok
test version::tests::serde_roundtrip ... ok
test version::tests::upgrade_check ... ok

test result: ok. 109 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

## project-lib-tests — PASS

Command:

```text
nix develop -c cargo test -p crunch-project --lib
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project-8d0d664f86eaf824)

running 3 tests
test refresh_adapter::tests::adapter_preserves_failed_resolution_behavior ... ok
test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok
test attestation_adapter::tests::project_attestation_adapter_preserves_borrowed_input_api ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

## project-refresh-cli-tests — PASS

Command:

```text
nix develop -c cargo test -p mantle --test project_refresh_cli
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.99s
     Running tests/project_refresh_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_refresh_cli-448ee49216b77cda)

running 5 tests
test list_stale_reports_stale_and_failed_without_mutating_files ... ok
test refresh_partial_failure_writes_successes_and_exits_nonzero ... ok
test refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes ... ok
test build_fetch_policy_refresh_uses_expected_hash_without_network_resolution ... ok
test refresh_git_input_locks_resolved_rev_and_tree_hash ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s


```

## project-check-policy-cli-test — PASS

Command:

```text
nix develop -c cargo test -p mantle --test project_cli check_static_accepts_build_fetch_policy_without_fetching_remote_url -- --nocapture
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
     Running tests/project_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_cli-25348db11eea788f)

running 1 test
test check_static_accepts_build_fetch_policy_without_fetching_remote_url ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.05s


```

## stdlib-tests — PASS

Command:

```text
nix develop -c cargo test -p mantle --test stdlib_tests
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.94s
     Running tests/stdlib_tests.rs (/home/brittonr/.cargo-target/debug/deps/stdlib_tests-f6cc654b68816c45)

running 22 tests
test contract_catches_missing_name ... ok
test contract_rejects_extra_fields ... ok
test enum_tags_deserialize_via_json ... ok
test store_path_validator_accepts_custom_store_prefix ... ok
test store_path_validator_accepts_nix_store_path ... ok
test full_serde_round_trip ... ok
test enum_tags_deserialize_direct_serde ... ok
test select_produces_correct_structure ... ok
test contract_catches_wrong_type ... ok
test fixed_output_contract ... ok
test store_path_validator_rejects_invalid ... ok
test fetchurl_rejects_empty_url ... ok
test fetchurl_preserves_optional_fetch_policy_env ... ok
test recursive_record_self_reference ... ok
test fetchurl_custom_name ... ok
test i386_linux_system_tag_is_accepted ... ok
test fetch_git_produces_git_env ... ok
test fetch_tarball_produces_recursive_hash ... ok
test fetchurl_invalid_hash_passes_eval ... ok
test fetchurl_produces_fod_record ... ok
test select_round_trip_through_glue ... ok
test mixed_inputs_array_validates ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s


```

## source-bundle-policy-metadata-test — PASS

Command:

```text
nix develop -c cargo test -p mantle --bin mantle source_bundle::tests::source_bundle_preserves_fetch_policy_metadata_from_fetch_derivation -- --nocapture
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.17s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test source_bundle::tests::source_bundle_preserves_fetch_policy_metadata_from_fetch_derivation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1054 filtered out; finished in 0.00s


```

## source-bundle-offline-preflight-cli-test — PASS

Command:

```text
nix develop -c cargo test -p mantle --test source_bundle_cli source_bundle_cli_preflight_reports_network_required_before_build -- --nocapture
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.55s
     Running tests/source_bundle_cli.rs (/home/brittonr/.cargo-target/debug/deps/source_bundle_cli-c881b3f1f9cb0791)

running 1 test
test source_bundle_cli_preflight_reports_network_required_before_build ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.06s


```

## wasm32-core-check — BLOCKED

Command:

```text
nix develop -c cargo check -p crunch-project-core --target wasm32-unknown-unknown
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking arrayref v0.3.9
    Checking arrayvec v0.7.6
    Checking cfg-if v1.0.4
    Checking itoa v1.0.18
    Checking constant_time_eq v0.3.1
    Checking memchr v2.8.0
    Checking data-encoding v2.10.0
    Checking serde_core v1.0.228
    Checking zmij v1.0.21
   Compiling serde_derive v1.0.228
error[E0463]: can't find crate for `core`
  |
  = note: the `wasm32-unknown-unknown` target may not be installed
  = help: consider downloading the target with `rustup target add wasm32-unknown-unknown`
  = help: consider building the standard library from source with `cargo build -Zbuild-std`

For more information about this error, try `rustc --explain E0463`.
error: could not compile `cfg-if` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `arrayvec` (lib) due to 1 previous error
error: could not compile `serde_core` (lib) due to 1 previous error
error: could not compile `zmij` (lib) due to 1 previous error
error: could not compile `constant_time_eq` (lib) due to 1 previous error
error: could not compile `data-encoding` (lib) due to 1 previous error
error: could not compile `arrayref` (lib) due to 1 previous error
error: could not compile `itoa` (lib) due to 1 previous error
error: could not compile `memchr` (lib) due to 1 previous error

```

## cairn-validate — PASS

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```text
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}

```

## cairn-gate-proposal — PASS

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal input-fetch-policy --root .
```

Output:

```text
{
  "change": "input-fetch-policy",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "208a49721c512437658069af566f59d487b49aaa8f9e62c9ff03d4f39971a7ed",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "dce5209eba2ccfc5358243e74f634523bf1ffb0eae3b39b0183d25469a6ce587",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn-gate-design — PASS

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate design input-fetch-policy --root .
```

Output:

```text
{
  "change": "input-fetch-policy",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "7703029c3032e313fcca1df4400c8183df0c959dfb28b6adfa2f5b7a464d7827",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "531f9e31d34040d67c6b66499f5bb09a52d4fa9db80517d483af7466eac86796",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn-gate-tasks — PASS

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks input-fetch-policy --root .
```

Output:

```text
{
  "change": "input-fetch-policy",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3428a976e47b41300d3fbbad91fc759379a4cf24546645fc511a00c76bf424ba",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d01a7446873995adb7988f02e475552c8a7cdc78a09b6c926356cd5a33d32c78",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

## post-archive-cairn-validate — PASS

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

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

```

## post-archive-location-check — PASS

Command:

```text
test -d cairn/archive/2026-07-01-input-fetch-policy
```

Output:

```text
```
