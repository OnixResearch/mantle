# Implementation validation — global reproducibility claims

Date: 2026-07-02

## baseline release-core lib tests

```text
$ nix develop -c cargo test -p crunch-release-core --lib
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 74 tests
test determinism::tests::deterministic_receipt_blocks_reused_output_store_identity ... ok
test determinism::tests::deterministic_receipt_rejects_digest_drift ... ok
test determinism::tests::deterministic_receipt_rejects_provider_kind_mismatch ... ok
test determinism::tests::deterministic_receipt_rejects_missing_effect_observations ... ok
test determinism::tests::deterministic_receipt_blocks_impure_mode ... ok
test determinism::tests::deterministic_receipt_accepts_strict_matching_runs ... ok
test determinism::tests::deterministic_receipt_rejects_unsupported_workflow_version ... ok
test determinism::tests::deterministic_release_claim_rejects_failed_isolation_evidence ... ok
test determinism::tests::deterministic_receipt_canonical_bytes_are_stable ... ok
test determinism::tests::deterministic_release_claim_rejects_profile_family_mismatch ... ok
test determinism::tests::deterministic_receipt_rejects_undeclared_observed_effects ... ok
test determinism::tests::deterministic_release_claim_rejects_missing_isolation_check ... ok
test determinism::tests::deterministic_receipt_rejects_unsupported_sandbox_profile ... ok
test determinism::tests::deterministic_release_claim_requires_isolation_evidence ... ok
test determinism::tests::deterministic_receipt_rejects_unsupported_effect_policy ... ok
test determinism::tests::deterministic_release_claim_requires_exact_digest_set ... ok
test determinism::tests::deterministic_sandbox_isolation_evidence_canonical_bytes_sort_checks ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_wrong_schema ... ok
test manifest::tests::canonical_bytes_are_stable_and_compact ... ok
test global_reproducibility::tests::release_scoped_witness_evidence_alone_does_not_become_global ... ok
test global_reproducibility::tests::global_reproducibility_report_accepts_tiny_eligible_universe ... ok
test manifest::tests::extract_full_proof_identity_fields_accepts_valid_manifest ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_accepts_matching_binary ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_missing_binaries ... ok
test global_reproducibility::tests::global_reproducibility_report_canonical_bytes_are_stable ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_mismatch ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_malformed_digest ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_missing_provider_kind ... ok
test manifest::tests::release_manifest_rejects_unknown_selected_provider_kind ... ok
test global_reproducibility::tests::global_reproducibility_blocks_required_negative_fixtures ... ok
test manifest::tests::validate_rejects_absolute_member_path ... ok
test manifest::tests::validate_accepts_deterministic_proof_artifacts ... ok
test manifest::tests::validate_accepts_git_source_acquisition ... ok
test manifest::tests::validate_rejects_credential_bearing_git_source_url ... ok
test manifest::tests::validate_accepts_matching_external_source_acquisition ... ok
test manifest::tests::validate_rejects_deterministic_proof_duplicate_path ... ok
test manifest::tests::validate_accepts_provider_fixed_point_proof_artifact ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_path ... ok
test manifest::tests::validate_rejects_git_source_archive_digest_mismatch ... ok
test manifest::tests::validate_rejects_git_source_with_malformed_commit ... ok
test manifest::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_file_kind ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_source_acquisition_digest_mismatch ... ok
test manifest::tests::validate_rejects_unsupported_source_acquisition_url ... ok
test manifest::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test nix_witness::tests::canonicalization_preserves_flag_order_and_sorts_artifact_sets ... ok
test nix_witness::tests::missing_mantle_proof_digest_is_rejected ... ok
test nix_witness::tests::mismatch_receipt_must_not_report_nix_witness_proof_class ... ok
test nix_witness::tests::mismatch_is_fail_closed_to_cross_builder_mismatch_verdict ... ok
test nix_witness::tests::build_flag_order_changes_canonical_digest ... ok
test nix_witness::tests::receipt_digest_must_match_canonical_bytes ... ok
test reproducibility::tests::reproducibility_report_accepts_matching_artifact_names_out_of_order ... ok
test reproducibility::tests::reproducibility_report_accepts_missing_rebuilt_artifact_fixture ... ok
test reproducibility::tests::reproducibility_report_accepts_matching_linkage ... ok
test nix_witness::tests::canonical_receipt_is_compact_stable_and_digest_bound ... ok
test reproducibility::tests::reproducibility_report_rejects_matched_artifact_with_drift ... ok
test reproducibility::tests::reproducibility_report_rejects_impure_rebuild_claim ... ok
test nix_witness::tests::verdict_must_match_digest_set_comparison ... ok
test reproducibility::tests::reproducibility_report_rejects_missing_artifact_with_observed_digest ... ok
test reproducibility::tests::reproducibility_report_rejects_proof_linkage_mismatch_fixture ... ok
test reproducibility::tests::reproducibility_report_rejects_rebuild_class_without_clean_store_identity ... ok
test reproducibility::tests::reproducibility_report_canonical_bytes_are_stable_and_sorted ... ok
test reproducibility::tests::reproducibility_report_rejects_output_name_drift_fixture ... ok
test source_archive::tests::private_runtime_and_lifecycle_paths_are_excluded ... ok
test source_archive::tests::source_paths_with_target_named_components_are_preserved ... ok
test reproducibility::tests::reproducibility_report_digest_is_blake3_of_canonical_bytes ... ok
test source_archive::tests::submodules_are_rejected ... ok
test source_archive::tests::symlink_targets_must_stay_relative ... ok
test source_archive::tests::unsafe_paths_are_rejected ... ok
test source_archive::tests::unordered_entries_produce_stable_member_order ... ok
test reproducibility::tests::reproducibility_report_accepts_mismatched_digest ... ok
test reproducibility::tests::reproducibility_report_rejects_failed_verdict_with_rebuild_claim ... ok

test result: ok. 74 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exit_status=0
```

## format check

```text
$ nix develop -c cargo fmt --check -p mantle -p crunch-release-core

exit_status=0
```

## focused global reproducibility core tests

```text
$ nix develop -c cargo test -p crunch-release-core --lib global_reproducibility
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.77s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 4 tests
test global_reproducibility::tests::release_scoped_witness_evidence_alone_does_not_become_global ... ok
test global_reproducibility::tests::global_reproducibility_report_accepts_tiny_eligible_universe ... ok
test global_reproducibility::tests::global_reproducibility_report_canonical_bytes_are_stable ... ok
test global_reproducibility::tests::global_reproducibility_blocks_required_negative_fixtures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 70 filtered out; finished in 0.00s


exit_status=0
```

## focused global reproducibility CLI shell tests

```text
$ nix develop -c cargo test -p mantle --bin mantle global_reproducibility_cmd
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 2 tests
test global_reproducibility_cmd::tests::shell_allows_missing_evidence_but_report_blocks_global_claim ... ok
test global_reproducibility_cmd::tests::shell_loads_evidence_and_writes_canonical_report ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1091 filtered out; finished in 0.02s


exit_status=0
```

## release command regression tests

```text
$ nix develop -c cargo test -p mantle --bin mantle release_cmd::
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 14 tests
test release_cmd::tests::compute_artifact_set_digest_differs_for_different_sets ... ok
test release_cmd::tests::provider_fixed_point_request_absent_is_optional_by_default ... ok
test release_cmd::tests::compute_artifact_set_digest_is_deterministic ... ok
test release_cmd::tests::provider_fixed_point_binding_records_matching_release_artifact ... ok
test release_cmd::tests::provider_fixed_point_binding_rejects_mismatched_release_artifact ... ok
test release_cmd::tests::stagex_profile_json_never_says_quorum_satisfied ... ok
test release_cmd::tests::provider_fixed_point_request_absent_records_required_blocker ... ok
test release_cmd::tests::stagex_profile_rejects_prerequisite_only_proof ... ok
test release_cmd::tests::stagex_profile_rejects_legacy_fetch_provider ... ok
test release_cmd::tests::stagex_profile_rejects_witness_agreement_without_lineage_proof ... ok
test release_cmd::tests::stagex_profile_rejects_self_proof_only_evidence ... ok
test release_cmd::tests::stagex_profile_rejects_source_root_provider ... ok
test release_cmd::tests::stagex_profile_rejects_missing_reproducibility ... ok
test release_cmd::tests::extract_stagex_proof_block_returns_none_for_missing_summary ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1079 filtered out; finished in 0.01s


exit_status=0
```

## git diff check

```text
$ git diff --check

exit_status=0
```

## cairn validate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}

exit_status=0
```

## cairn gate proposal

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal global-reproducibility-claims --root /home/brittonr/git/mantle
{
  "change": "global-reproducibility-claims",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "80a029e6d8b2af309161613faa167dceacd22d615fe60f4c6ba366237922ed70",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "3cb151b232f2dc4662bef1d3531563920defb519933669723701a60cb8a9c2dc",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design global-reproducibility-claims --root /home/brittonr/git/mantle
{
  "change": "global-reproducibility-claims",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d92a26494bf532d09154281a52f19c8a575e8e726d28f3a58c9dbb88dd66545e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6b4dedd947c3cba13918f4a95e95bb3202ae13f1b8272d24dbc8a9a59c4f3dea",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks global-reproducibility-claims --root /home/brittonr/git/mantle
{
  "change": "global-reproducibility-claims",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "24fa8656bb323f0362ba1739a6b8fbef098c0245f7aa9c5d5c003b596ae393ce",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "2610a7fa995dcfdb922992902dfba7edc3f538fc052025170434e7e84e47666f",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## Archive execution note

Archive execution completed before this transcript append with `CAIRN_ARCHIVE_DATE=2026-07-02 cairn archive global-reproducibility-claims --execute`; the active change directory was moved to `cairn/archive/2026-07-02-global-reproducibility-claims/`.

## post-archive cairn validate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 15,
  "valid": true
}

exit_status=0
```

## post-archive active change list

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- change list --root /home/brittonr/git/mantle
{
  "changes": [],
  "layout": "cairn",
  "root": "/home/brittonr/git/mantle"
}

exit_status=0
```

## Secondary-review follow-up

Added a release-verify JSON regression asserting global reproducibility remains `not-evaluated`/`non-global` on release-scoped verification.

## post-review format check

```text
$ nix develop -c cargo fmt --check -p mantle -p crunch-release-core

exit_status=0
```

## post-review release command regression tests

```text
$ nix develop -c cargo test -p mantle --bin mantle release_cmd::
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.52s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 15 tests
test release_cmd::tests::provider_fixed_point_request_absent_is_optional_by_default ... ok
test release_cmd::tests::compute_artifact_set_digest_differs_for_different_sets ... ok
test release_cmd::tests::provider_fixed_point_binding_records_matching_release_artifact ... ok
test release_cmd::tests::compute_artifact_set_digest_is_deterministic ... ok
test release_cmd::tests::provider_fixed_point_binding_rejects_mismatched_release_artifact ... ok
test release_cmd::tests::provider_fixed_point_request_absent_records_required_blocker ... ok
test release_cmd::tests::release_verify_json_keeps_global_reproducibility_non_global ... ok
test release_cmd::tests::stagex_profile_json_never_says_quorum_satisfied ... ok
test release_cmd::tests::extract_stagex_proof_block_returns_none_for_missing_summary ... ok
test release_cmd::tests::stagex_profile_rejects_prerequisite_only_proof ... ok
test release_cmd::tests::stagex_profile_rejects_self_proof_only_evidence ... ok
test release_cmd::tests::stagex_profile_rejects_missing_reproducibility ... ok
test release_cmd::tests::stagex_profile_rejects_witness_agreement_without_lineage_proof ... ok
test release_cmd::tests::stagex_profile_rejects_legacy_fetch_provider ... ok
test release_cmd::tests::stagex_profile_rejects_source_root_provider ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1079 filtered out; finished in 0.00s


exit_status=0
```

## post-review global reproducibility CLI shell tests

```text
$ nix develop -c cargo test -p mantle --bin mantle global_reproducibility_cmd
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 2 tests
test global_reproducibility_cmd::tests::shell_allows_missing_evidence_but_report_blocks_global_claim ... ok
test global_reproducibility_cmd::tests::shell_loads_evidence_and_writes_canonical_report ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1092 filtered out; finished in 0.00s


exit_status=0
```

## post-review release-core lib tests

```text
$ nix develop -c cargo test -p crunch-release-core --lib
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 74 tests
test determinism::tests::deterministic_receipt_blocks_reused_output_store_identity ... ok
test determinism::tests::deterministic_receipt_rejects_digest_drift ... ok
test determinism::tests::deterministic_receipt_rejects_missing_effect_observations ... ok
test determinism::tests::deterministic_receipt_blocks_impure_mode ... ok
test determinism::tests::deterministic_receipt_accepts_strict_matching_runs ... ok
test determinism::tests::deterministic_receipt_rejects_provider_kind_mismatch ... ok
test determinism::tests::deterministic_receipt_rejects_unsupported_workflow_version ... ok
test determinism::tests::deterministic_release_claim_rejects_failed_isolation_evidence ... ok
test determinism::tests::deterministic_receipt_rejects_undeclared_observed_effects ... ok
test determinism::tests::deterministic_receipt_rejects_unsupported_effect_policy ... ok
test determinism::tests::deterministic_receipt_rejects_unsupported_sandbox_profile ... ok
test determinism::tests::deterministic_receipt_canonical_bytes_are_stable ... ok
test determinism::tests::deterministic_release_claim_rejects_profile_family_mismatch ... ok
test determinism::tests::deterministic_release_claim_rejects_missing_isolation_check ... ok
test determinism::tests::deterministic_release_claim_requires_isolation_evidence ... ok
test determinism::tests::deterministic_release_claim_requires_exact_digest_set ... ok
test determinism::tests::deterministic_sandbox_isolation_evidence_canonical_bytes_sort_checks ... ok
test global_reproducibility::tests::release_scoped_witness_evidence_alone_does_not_become_global ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_wrong_schema ... ok
test global_reproducibility::tests::global_reproducibility_report_accepts_tiny_eligible_universe ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_accepts_matching_binary ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_malformed_digest ... ok
test manifest::tests::extract_full_proof_identity_fields_accepts_valid_manifest ... ok
test manifest::tests::canonical_bytes_are_stable_and_compact ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_mismatch ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_missing_binaries ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_missing_provider_kind ... ok
test manifest::tests::release_manifest_rejects_unknown_selected_provider_kind ... ok
test global_reproducibility::tests::global_reproducibility_report_canonical_bytes_are_stable ... ok
test global_reproducibility::tests::global_reproducibility_blocks_required_negative_fixtures ... ok
test manifest::tests::validate_accepts_deterministic_proof_artifacts ... ok
test manifest::tests::validate_rejects_absolute_member_path ... ok
test manifest::tests::validate_accepts_git_source_acquisition ... ok
test manifest::tests::validate_accepts_matching_external_source_acquisition ... ok
test manifest::tests::validate_rejects_credential_bearing_git_source_url ... ok
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
test nix_witness::tests::mismatch_is_fail_closed_to_cross_builder_mismatch_verdict ... ok
test nix_witness::tests::canonicalization_preserves_flag_order_and_sorts_artifact_sets ... ok
test nix_witness::tests::mismatch_receipt_must_not_report_nix_witness_proof_class ... ok
test nix_witness::tests::missing_mantle_proof_digest_is_rejected ... ok
test nix_witness::tests::build_flag_order_changes_canonical_digest ... ok
test nix_witness::tests::verdict_must_match_digest_set_comparison ... ok
test reproducibility::tests::reproducibility_report_accepts_matching_artifact_names_out_of_order ... ok
test nix_witness::tests::receipt_digest_must_match_canonical_bytes ... ok
test reproducibility::tests::reproducibility_report_accepts_mismatched_digest ... ok
test nix_witness::tests::canonical_receipt_is_compact_stable_and_digest_bound ... ok
test reproducibility::tests::reproducibility_report_accepts_missing_rebuilt_artifact_fixture ... ok
test reproducibility::tests::reproducibility_report_rejects_failed_verdict_with_rebuild_claim ... ok
test reproducibility::tests::reproducibility_report_rejects_rebuild_class_without_clean_store_identity ... ok
test reproducibility::tests::reproducibility_report_rejects_impure_rebuild_claim ... ok
test reproducibility::tests::reproducibility_report_canonical_bytes_are_stable_and_sorted ... ok
test source_archive::tests::private_runtime_and_lifecycle_paths_are_excluded ... ok
test source_archive::tests::submodules_are_rejected ... ok
test reproducibility::tests::reproducibility_report_digest_is_blake3_of_canonical_bytes ... ok
test source_archive::tests::source_paths_with_target_named_components_are_preserved ... ok
test source_archive::tests::symlink_targets_must_stay_relative ... ok
test source_archive::tests::unsafe_paths_are_rejected ... ok
test source_archive::tests::unordered_entries_produce_stable_member_order ... ok
test reproducibility::tests::reproducibility_report_rejects_matched_artifact_with_drift ... ok
test reproducibility::tests::reproducibility_report_accepts_matching_linkage ... ok
test reproducibility::tests::reproducibility_report_rejects_output_name_drift_fixture ... ok
test reproducibility::tests::reproducibility_report_rejects_missing_artifact_with_observed_digest ... ok
test reproducibility::tests::reproducibility_report_rejects_proof_linkage_mismatch_fixture ... ok

test result: ok. 74 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exit_status=0
```

## post-review cairn validate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 15,
  "valid": true
}

exit_status=0
```

## post-review git diff check

```text
$ git diff --check

exit_status=0
```

