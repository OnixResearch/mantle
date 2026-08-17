# Project freshness probes implementation evidence

Date: 2026-07-01

## Focused tests

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 15 tests
test freshness::tests::command_probe_validation_accepts_bounded_contract ... ok
test freshness::tests::builtin_observation_normalizes_digest_and_kind ... ok
test freshness::tests::command_probe_without_argv_is_rejected ... ok
test freshness::tests::diagnostics_are_bounded_deterministically ... ok
test freshness::tests::empty_observed_value_is_rejected ... ok
test freshness::tests::invalid_template_variable_is_rejected ... ok
test freshness::tests::freshness_plan_classifies_selected_stale_and_unchanged ... ok
test freshness::tests::failed_observation_classifies_without_lock_mutation_claim ... ok
test freshness::tests::observation_for_unknown_input_is_rejected ... ok
test freshness::tests::network_required_observation_is_classified_in_offline_mode ... ok
test freshness::tests::observed_network_command_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::observed_network_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::oversized_observed_value_is_rejected ... ok
test freshness::tests::rendered_template_bound_is_enforced ... ok
test freshness::tests::template_renders_validated_freshness_value ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 111 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 14 tests
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::freshness_unchanged_decision_skips_resolution_work ... ok
test refresh::tests::freshness_network_required_decision_fails_closed_for_refresh ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 112 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project-8d0d664f86eaf824)

running 2 tests
test refresh_adapter::tests::adapter_preserves_failed_resolution_behavior ... ok
test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running tests/project_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_cli-25348db11eea788f)

running 16 tests
test init_creates_project_files ... ok
test check_fails_without_init ... ok
test check_fails_on_conflicting_legacy_project_files ... ok
test upgrade_on_current_version ... ok
test init_fails_if_already_initialized ... ok
test refresh_on_empty_project ... ok
test check_detects_missing_inputs_file ... ok
test check_detects_drift ... ok
test show_on_empty_project ... ok
test check_json_explicit_probe_and_trust_mode_labels_behavior ... ok
test list_stale_on_empty_project ... ok
test check_passes_on_fresh_project ... ok
test check_static_accepts_build_fetch_policy_without_fetching_remote_url ... ok
test check_json_failure_stdout_is_parseable_report ... ok
test check_json_passes_on_fresh_project_with_bounded_non_claims ... ok
test refresh_hashes_local_patch_relative_to_project_root ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running tests/project_refresh_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_refresh_cli-448ee49216b77cda)

running 10 tests
test refresh_partial_failure_writes_successes_and_exits_nonzero ... ok
test list_stale_reports_stale_and_failed_without_mutating_files ... ok
test freshness_no_network_mode_does_not_contact_http_probe ... ok
test freshness_local_directory_probe_updates_lock_digest ... ok
test freshness_http_json_template_refreshes_selected_stale_input ... ok
test refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes ... ok
test build_fetch_policy_refresh_uses_expected_hash_without_network_resolution ... ok
test freshness_git_ref_probe_reports_stale_without_mutating_lock ... ok
test refresh_git_input_locks_resolved_rev_and_tree_hash ... ok
test freshness_command_missing_timeout_and_output_limit_fail_deterministically ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s


[test-status=0]
```

## Formatting

```text

[fmt-status=0]
```


## Cairn validation and gates

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

[validate-status=0]

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

[proposal-gate-status=0]

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

[design-gate-status=0]

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

[tasks-gate-status=0]
```

## Post-task-check validation

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

[validate-status=0]

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
  "input_hash": "8d5229ada9d6cc64f2c76119ea5462f7920bcbcee8b15fcfae0d34bbebe75888",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "eefd6b8d4eb41f799fa6701a863c04fdd6ce00f5465c2311e26222aa68c750c6",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

[tasks-gate-status=0]
```

## Archive execution

```text
{
  "actions": [
    {
      "description": "move active change to archive: project-freshness-probes",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-07-01-project-freshness-probes"
    }
  ],
  "blocked": false,
  "change": "project-freshness-probes",
  "dry_run": false,
  "input_hash": "00cecce8d8391d982bfdd4935a839c2d3bc4d923d0606db1687d6a99c8b3bde0",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "c9ed3af023ca853061e543029a070c0c4fc591bd75b7db13284990e74aa8cf6d",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/design.md"
        },
        {
          "content_hash": "e989320b84d5c9f4d46c8c79e4a1ef027c7af5b9e55e0cf964454d0572b4d7ac",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/evidence/current-blocker.md"
        },
        {
          "content_hash": "f7917d25c4a2012e57da03c1cc8c5ebce8d8dee652fd3628e9c3c7a9e92d8aee",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/evidence/implementation-transcript.md"
        },
        {
          "content_hash": "f259e47e8c67a2a2e4847fb87d736b82097d42219815840810df78fe87eec1cd",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/evidence/pure-core-validation-2026-07-01.md"
        },
        {
          "content_hash": "35f2787f210e0f487cc8e2208d47abed16933f964d5879fde05a91b918e9dc0f",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/evidence/scaffold-validation-2026-07-01.md"
        },
        {
          "content_hash": "7b4f2c0841a519f995f1954702f651e98b30662c9f3b0b060cad65a6bfd99ccc",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/proposal.md"
        },
        {
          "content_hash": "5e1fc6ff8bf421cef3b5678e20130c73242071cfebd19400b9c860df4a78a444",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/specs/project-workflows/spec.md"
        },
        {
          "content_hash": "ba35801dc3f6f07e30dd993555ee4874897faa8b93a3fa2c228ed074ae137225",
          "exists": true,
          "path": "./cairn/archive/2026-07-01-project-freshness-probes/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/project-freshness-probes"
        }
      ],
      "manifest_hash": "9ab7543fe103b41b77b1887213f99ee216d795aafe9b6a9fd64b09bacae473aa"
    },
    "before": {
      "entries": [
        {
          "content_hash": "c9ed3af023ca853061e543029a070c0c4fc591bd75b7db13284990e74aa8cf6d",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/design.md"
        },
        {
          "content_hash": "e989320b84d5c9f4d46c8c79e4a1ef027c7af5b9e55e0cf964454d0572b4d7ac",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/evidence/current-blocker.md"
        },
        {
          "content_hash": "f7917d25c4a2012e57da03c1cc8c5ebce8d8dee652fd3628e9c3c7a9e92d8aee",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/evidence/implementation-transcript.md"
        },
        {
          "content_hash": "f259e47e8c67a2a2e4847fb87d736b82097d42219815840810df78fe87eec1cd",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/evidence/pure-core-validation-2026-07-01.md"
        },
        {
          "content_hash": "35f2787f210e0f487cc8e2208d47abed16933f964d5879fde05a91b918e9dc0f",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/evidence/scaffold-validation-2026-07-01.md"
        },
        {
          "content_hash": "7b4f2c0841a519f995f1954702f651e98b30662c9f3b0b060cad65a6bfd99ccc",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/proposal.md"
        },
        {
          "content_hash": "5e1fc6ff8bf421cef3b5678e20130c73242071cfebd19400b9c860df4a78a444",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/specs/project-workflows/spec.md"
        },
        {
          "content_hash": "ba35801dc3f6f07e30dd993555ee4874897faa8b93a3fa2c228ed074ae137225",
          "exists": true,
          "path": "./cairn/changes/project-freshness-probes/tasks.md"
        }
      ],
      "manifest_hash": "d1b1406e89eda73f8add109a87a7c8cf678a02c5fab0ca44f2db9f9574800333"
    },
    "kind": "archive",
    "manifest_hash": "099b52bb87ac83ed6974ffb5ccbd34b659d1ddaceb74664e4aea8ef965fed6c9"
  },
  "plan_hash": "814ccf77a8a69e3b146286a449c13c74ead913a3cc398782d6aeb69a5ae72346",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "d95d59c84949088d21dce16f17926bc54a4918d300e6f8632c7d3bfd428de9eb"
}

[archive-status=0]
```

## Post-archive accepted-spec validation

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}

[validate-status=0]
```

## Post-archive tracey coverage

```text
{
  "dangling": [
    "build_correctness.action_spec",
    "build_correctness.cas_object_store",
    "build_correctness.hermetic_execution_policy",
    "build_correctness.nickel_eval_source_closure",
    "build_correctness.output_reference_scanning",
    "build_correctness.reuse_and_substitution",
    "build_tool_boundary.correctness_primitives_frontend_neutral",
    "project_workflows.input_fetch_policy",
    "project_workflows.input_fetch_policy_preflight",
    "project_workflows.nixtamal_importer",
    "project_workflows.project_lock_importers",
    "verification_evidence.build_correctness_receipts"
  ],
  "missing": [
    "rust_package_planning.bundle_deterministic_release_proof",
    "rust_package_planning.bundle_provider_fixed_point_release_evidence",
    "rust_package_planning.cargo_free_fixed_point_command",
    "rust_package_planning.cargo_free_fixed_point_proof",
    "rust_package_planning.cargo_free_self_build_command",
    "rust_package_planning.cargo_free_self_build_proof",
    "rust_package_planning.cargo_free_topology_proof",
    "rust_package_planning.cargo_oracle_parity",
    "rust_package_planning.cargo_oracle_parity.capture",
    "rust_package_planning.cargo_oracle_parity.compare",
    "rust_package_planning.compiler_policy_adapter",
    "rust_package_planning.compiler_policy_adapter.cache_identity",
    "rust_package_planning.compiler_policy_adapter.cache_identity.raw_rustc_rejected",
    "rust_package_planning.compiler_policy_adapter.cli",
    "rust_package_planning.compiler_policy_adapter.fail_closed",
    "rust_package_planning.compiler_policy_adapter.fail_closed.missing_material",
    "rust_package_planning.compiler_policy_adapter.invocation",
    "rust_package_planning.compiler_policy_adapter.non_claims",
    "rust_package_planning.compiler_policy_adapter.non_claims.scope",
    "rust_package_planning.compiler_policy_adapter.octet",
    "rust_package_planning.compiler_policy_adapter.receipts",
    "rust_package_planning.compiler_policy_adapter.standards_gate",
    "rust_package_planning.fail_closed_boundaries",
    "rust_package_planning.fail_closed_boundaries.non_claims",
    "rust_package_planning.fail_closed_boundaries.unsupported",
    "rust_package_planning.host_target_split",
    "rust_package_planning.host_target_split.build_scripts",
    "rust_package_planning.host_target_split.proc_macros",
    "rust_package_planning.native_build_oracle_target_scope",
    "rust_package_planning.native_build_oracle_target_scope.normal_build",
    "rust_package_planning.native_build_script_execution_env",
    "rust_package_planning.native_build_script_package_metadata_env",
    "rust_package_planning.native_build_script_package_name_env",
    "rust_package_planning.native_build_script_profile_env",
    "rust_package_planning.native_build_script_runtime",
    "rust_package_planning.native_build_script_target_cfg_env",
    "rust_package_planning.native_build_topology_dev_dependency_scope",
    "rust_package_planning.native_build_topology_dev_dependency_scope.dev_test_explicit",
    "rust_package_planning.native_build_topology_dev_dependency_scope.normal_build",
    "rust_package_planning.native_compile_env_allowlist",
    "rust_package_planning.native_crate_disambiguators",
    "rust_package_planning.native_custom_build_alias_binding",
    "rust_package_planning.native_dependency_cap_lints",
    "rust_package_planning.native_dependency_feature_edge_scope",
    "rust_package_planning.native_executor_hardening",
    "rust_package_planning.native_feature_resolution",
    "rust_package_planning.native_git_dependency_source_scope",
    "rust_package_planning.native_host_artifact_topology_execution",
    "rust_package_planning.native_host_artifact_topology_execution.binds",
    "rust_package_planning.native_host_artifact_topology_execution.blockers",
    "rust_package_planning.native_host_artifact_topology_execution.bounded_claim",
    "rust_package_planning.native_host_artifact_topology_execution.executes",
    "rust_package_planning.native_host_artifact_topology_execution.metadata",
    "rust_package_planning.native_host_dependency_producer_coverage",
    "rust_package_planning.native_host_real_unit_identity",
    "rust_package_planning.native_host_unit_graph_planning",
    "rust_package_planning.native_host_unit_graph_planning.blockers",
    "rust_package_planning.native_host_unit_graph_planning.compare",
    "rust_package_planning.native_host_unit_graph_planning.consumes_native",
    "rust_package_planning.native_host_unit_graph_planning.receipts",
    "rust_package_planning.native_host_unit_graph_planning.tests",
    "rust_package_planning.native_host_unit_graph_planning.verify",
    "rust_package_planning.native_link_lib_metadata",
    "rust_package_planning.native_linked_build_script_metadata_env",
    "rust_package_planning.native_manifest_edition_derivations",
    "rust_package_planning.native_manifest_links_env",
    "rust_package_planning.native_manifest_lock_planner",
    "rust_package_planning.native_proc_macro_crate_type_classification",
    "rust_package_planning.native_proc_macro_host_dependency_binding",
    "rust_package_planning.native_proc_macro_target_name_normalization",
    "rust_package_planning.native_real_unit_identity",
    "rust_package_planning.native_registry_dependency_version_resolution",
    "rust_package_planning.native_registry_host_artifact_topology_execution",
    "rust_package_planning.native_registry_host_artifact_topology_execution.blockers",
    "rust_package_planning.native_registry_host_artifact_topology_execution.executes",
    "rust_package_planning.native_registry_host_artifact_topology_execution.receipts",
    "rust_package_planning.native_registry_host_artifact_topology_execution.source_and_host_facts",
    "rust_package_planning.native_registry_host_artifact_topology_execution.tests",
    "rust_package_planning.native_registry_host_artifact_topology_execution.verify",
    "rust_package_planning.native_registry_source",
    "rust_package_planning.native_registry_source.blockers",
    "rust_package_planning.native_registry_source.lockfile_identity",
    "rust_package_planning.native_registry_source.oracle_compare",
    "rust_package_planning.native_registry_source.receipts",
    "rust_package_planning.native_registry_source.tests",
    "rust_package_planning.native_registry_source.vendor_digest",
    "rust_package_planning.native_registry_source.verify",
    "rust_package_planning.native_registry_topology_execution",
    "rust_package_planning.native_registry_topology_execution.blockers",
    "rust_package_planning.native_registry_topology_execution.executes",
    "rust_package_planning.native_registry_topology_execution.receipts",
    "rust_package_planning.native_registry_topology_execution.source_facts",
    "rust_package_planning.native_registry_topology_execution.tests",
    "rust_package_planning.native_registry_topology_execution.verify",
    "rust_package_planning.native_registry_transitive_producer_coverage",
    "rust_package_planning.native_registry_unified_host_topology_execution",
    "rust_package_planning.native_registry_unified_host_topology_execution.blockers",
    "rust_package_planning.native_registry_unified_host_topology_execution.executes",
    "rust_package_planning.native_registry_unified_host_topology_execution.receipts",
    "rust_package_planning.native_registry_unified_host_topology_execution.source_and_host_facts",
    "rust_package_planning.native_registry_unified_host_topology_execution.tests",
    "rust_package_planning.native_registry_unified_host_topology_execution.verify",
    "rust_package_planning.native_selected_host_units",
    "rust_package_planning.native_self_package_lib_binding",
    "rust_package_planning.native_self_target_cfg_predicate_scope",
    "rust_package_planning.native_target_cfg_optional_dependency_scope",
    "rust_package_planning.native_target_proc_macro_host_extern_binding",
    "rust_package_planning.native_transitive_search_paths",
    "rust_package_planning.native_unified_topology_execution",
    "rust_package_planning.native_unified_topology_execution.binds",
    "rust_package_planning.native_unified_topology_execution.blockers",
    "rust_package_planning.native_unified_topology_execution.executes",
    "rust_package_planning.native_unified_topology_execution.receipts",
    "rust_package_planning.native_unified_topology_execution.tests",
    "rust_package_planning.native_unified_topology_execution.verify",
    "rust_package_planning.native_unit_graph",
    "rust_package_planning.native_unit_graph_planning",
    "rust_package_planning.native_unit_graph_planning.blockers",
    "rust_package_planning.native_unit_graph_planning.compare",
    "rust_package_planning.native_unit_graph_planning.consumes_native",
    "rust_package_planning.native_unit_graph_planning.receipts",
    "rust_package_planning.native_unit_graph_planning.tests",
    "rust_package_planning.native_unit_graph_planning.verify",
    "rust_package_planning.native_unit_variant_artifacts",
    "rust_package_planning.no_cargo_oracle_cli",
    "rust_package_planning.provider_fixed_point_release_artifact_binding",
    "rust_package_planning.provider_fixed_point_release_verifier",
    "rust_package_planning.rust_topology_external_linker",
    "rust_package_planning.rust_topology_tool_path_env",
    "rust_package_planning.source_built_rust_seed_closure",
    "rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard",
    "rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary",
    "rust_package_planning.source_built_toolchain_closure.explicit_native_promotion",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_libgcc_eh_unwind",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_linker_wrapper_path",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_prefix_runtime",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_rustc_probe_runtime",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_pie_normalization",
    "rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier",
    "rust_package_planning.source_built_toolchain_closure.native_materialization",
    "rust_package_planning.source_built_toolchain_closure.native_static_pie_crt",
    "rust_package_planning.source_built_toolchain_closure.provider_contract_independence",
    "rust_package_planning.source_built_toolchain_closure.provider_fixed_point",
    "rust_package_planning.source_built_toolchain_closure.provider_status",
    "rust_package_planning.source_built_toolchain_closure.selectable_rust_source_route",
    "rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env",
    "rust_package_planning.source_built_toolchain_closure.source_root_musl_rust_provider_tools",
    "rust_package_planning.source_built_toolchain_closure.target_aliases",
    "rust_package_planning.source_closure",
    "rust_package_planning.source_closure.identities",
    "rust_package_planning.source_closure.offline",
    "rust_package_planning.unit_derivation_graph",
    "rust_package_planning.unit_derivation_graph.cache_identity",
    "rust_package_planning.unit_derivation_graph.units",
    "rust_package_planning.unit_execution",
    "rust_package_planning.unit_execution.bounded_claim",
    "rust_package_planning.unit_execution.build_script_metadata",
    "rust_package_planning.unit_execution.build_script_metadata.binds",
    "rust_package_planning.unit_execution.build_script_metadata.blockers",
    "rust_package_planning.unit_execution.build_script_metadata.captures",
    "rust_package_planning.unit_execution.build_script_metadata.executes",
    "rust_package_planning.unit_execution.build_script_metadata.link_binding",
    "rust_package_planning.unit_execution.build_script_metadata.link_lib_binding",
    "rust_package_planning.unit_execution.build_script_metadata.link_metadata_blockers",
    "rust_package_planning.unit_execution.build_script_metadata.link_search_binding",
    "rust_package_planning.unit_execution.dependency_chain",
    "rust_package_planning.unit_execution.dependency_chain.cli",
    "rust_package_planning.unit_execution.dependency_chain.cli.executes",
    "rust_package_planning.unit_execution.dependency_chain.produced_artifact",
    "rust_package_planning.unit_execution.host_artifacts",
    "rust_package_planning.unit_execution.host_artifacts.binds",
    "rust_package_planning.unit_execution.host_artifacts.blockers",
    "rust_package_planning.unit_execution.host_artifacts.blockers.missing_material",
    "rust_package_planning.unit_execution.host_artifacts.executes",
    "rust_package_planning.unit_execution.supported_unit",
    "rust_package_planning.unit_execution.target_topology",
    "rust_package_planning.unit_execution.target_topology.blockers",
    "rust_package_planning.unit_execution.target_topology.blockers.unsupported",
    "rust_package_planning.unit_execution.target_topology.executes",
    "rust_package_planning.unit_execution.topology",
    "rust_package_planning.unit_execution.topology.binds_all_artifacts",
    "rust_package_planning.unit_execution.topology.blockers",
    "rust_package_planning.unit_execution.topology.mixed_order",
    "rust_package_planning.unit_execution.topology.output_reuse",
    "rust_package_planning.unit_execution.topology.output_reuse.repeated",
    "rust_package_planning.unit_execution.topology.output_reuse_blockers",
    "rust_package_planning.unit_execution.topology.output_reuse_blockers.stale",
    "rust_package_planning.unit_execution.topology.unified_cli",
    "rust_package_planning.unit_execution_blockers",
    "rust_package_planning.unit_execution_blockers.dependency_chain",
    "rust_package_planning.unit_execution_blockers.dependency_chain.missing_material",
    "rust_package_planning.unit_execution_blockers.dependency_chain.unsupported_shape",
    "rust_package_planning.unit_execution_blockers.missing_material",
    "rust_package_planning.unit_execution_blockers.unsupported_boundary",
    "rust_package_planning.unit_execution_receipts",
    "rust_package_planning.unit_execution_receipts.build_script_metadata",
    "rust_package_planning.unit_execution_receipts.build_script_metadata.cli",
    "rust_package_planning.unit_execution_receipts.dependency_chain",
    "rust_package_planning.unit_execution_receipts.dependency_chain.cli",
    "rust_package_planning.unit_execution_receipts.dependency_chain.cli.ordered_units",
    "rust_package_planning.unit_execution_receipts.dependency_chain.ordered_units",
    "rust_package_planning.unit_execution_receipts.failure",
    "rust_package_planning.unit_execution_receipts.host_artifacts",
    "rust_package_planning.unit_execution_receipts.host_artifacts.cli",
    "rust_package_planning.unit_execution_receipts.host_artifacts.cli.json",
    "rust_package_planning.unit_execution_receipts.host_artifacts.ordered",
    "rust_package_planning.unit_execution_receipts.output_identity",
    "rust_package_planning.unit_execution_receipts.target_topology",
    "rust_package_planning.unit_execution_receipts.target_topology.cli",
    "rust_package_planning.unit_execution_receipts.target_topology.cli.json",
    "rust_package_planning.unit_execution_receipts.target_topology.ordered",
    "verification_evidence.portable_release_verification_replay",
    "verification_evidence.proof_before_claim",
    "verification_evidence.provider_bound_release_evidence_refresh_transcripts",
    "verification_evidence.provider_bound_release_evidence_transcripts",
    "verification_evidence.release_reproducibility_transcripts"
  ],
  "referenced": 35,
  "requirements": 254,
  "valid": false
}
error: tracey coverage failed

[tracey-status=1]
```

## Post-review focused regression

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.19s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 15 tests
test freshness::tests::builtin_observation_normalizes_digest_and_kind ... ok
test freshness::tests::command_probe_without_argv_is_rejected ... ok
test freshness::tests::diagnostics_are_bounded_deterministically ... ok
test freshness::tests::command_probe_validation_accepts_bounded_contract ... ok
test freshness::tests::empty_observed_value_is_rejected ... ok
test freshness::tests::invalid_template_variable_is_rejected ... ok
test freshness::tests::freshness_plan_classifies_selected_stale_and_unchanged ... ok
test freshness::tests::failed_observation_classifies_without_lock_mutation_claim ... ok
test freshness::tests::network_required_observation_is_classified_in_offline_mode ... ok
test freshness::tests::observation_for_unknown_input_is_rejected ... ok
test freshness::tests::observed_network_command_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::oversized_observed_value_is_rejected ... ok
test freshness::tests::observed_network_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::rendered_template_bound_is_enforced ... ok
test freshness::tests::template_renders_validated_freshness_value ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 111 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 14 tests
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::freshness_unchanged_decision_skips_resolution_work ... ok
test refresh::tests::freshness_network_required_decision_fails_closed_for_refresh ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 112 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project-core)
   Compiling crunch-project v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.23s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project-8d0d664f86eaf824)

running 2 tests
test refresh_adapter::tests::adapter_preserves_failed_resolution_behavior ... ok
test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.74s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test project_resolve::tests::oversized_shell_observation_becomes_failed_observation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1055 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 26.89s
     Running tests/project_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_cli-25348db11eea788f)

running 16 tests
test init_creates_project_files ... ok
test check_fails_without_init ... ok
test check_fails_on_conflicting_legacy_project_files ... ok
test upgrade_on_current_version ... ok
test init_fails_if_already_initialized ... ok
test check_detects_missing_inputs_file ... ok
test check_passes_on_fresh_project ... ok
test check_detects_drift ... ok
test check_json_failure_stdout_is_parseable_report ... ok
test show_on_empty_project ... ok
test refresh_hashes_local_patch_relative_to_project_root ... ok
test refresh_on_empty_project ... ok
test check_json_passes_on_fresh_project_with_bounded_non_claims ... ok
test list_stale_on_empty_project ... ok
test check_json_explicit_probe_and_trust_mode_labels_behavior ... ok
test check_static_accepts_build_fetch_policy_without_fetching_remote_url ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.39s
     Running tests/project_refresh_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_refresh_cli-448ee49216b77cda)

running 10 tests
test refresh_partial_failure_writes_successes_and_exits_nonzero ... ok
test freshness_local_directory_probe_updates_lock_digest ... ok
test list_stale_reports_stale_and_failed_without_mutating_files ... ok
test freshness_no_network_mode_does_not_contact_http_probe ... ok
test freshness_http_json_template_refreshes_selected_stale_input ... ok
test refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes ... ok
test build_fetch_policy_refresh_uses_expected_hash_without_network_resolution ... ok
test freshness_git_ref_probe_reports_stale_without_mutating_lock ... ok
test refresh_git_input_locks_resolved_rev_and_tree_hash ... ok
test freshness_command_missing_timeout_and_output_limit_fail_deterministically ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s


[test-status=0]
```

## Post-review formatting

```text

[fmt-status=0]
```

## Final Cairn validation

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}

[validate-status=0]
```

## Final focused regression after CLI output tweak

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 15 tests
test freshness::tests::diagnostics_are_bounded_deterministically ... ok
test freshness::tests::empty_observed_value_is_rejected ... ok
test freshness::tests::builtin_observation_normalizes_digest_and_kind ... ok
test freshness::tests::command_probe_without_argv_is_rejected ... ok
test freshness::tests::command_probe_validation_accepts_bounded_contract ... ok
test freshness::tests::invalid_template_variable_is_rejected ... ok
test freshness::tests::failed_observation_classifies_without_lock_mutation_claim ... ok
test freshness::tests::network_required_observation_is_classified_in_offline_mode ... ok
test freshness::tests::freshness_plan_classifies_selected_stale_and_unchanged ... ok
test freshness::tests::observation_for_unknown_input_is_rejected ... ok
test freshness::tests::observed_network_command_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::observed_network_probe_in_offline_mode_is_rejected ... ok
test freshness::tests::oversized_observed_value_is_rejected ... ok
test freshness::tests::rendered_template_bound_is_enforced ... ok
test freshness::tests::template_renders_validated_freshness_value ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 111 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project_core-eec7a0d866e95b9e)

running 14 tests
test refresh::tests::freshness_unchanged_decision_skips_resolution_work ... ok
test refresh::tests::freshness_network_required_decision_fails_closed_for_refresh ... ok
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 112 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_project-8d0d664f86eaf824)

running 2 tests
test refresh_adapter::tests::adapter_preserves_failed_resolution_behavior ... ok
test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.79s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test project_resolve::tests::oversized_shell_observation_becomes_failed_observation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1055 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.66s
     Running tests/project_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_cli-25348db11eea788f)

running 16 tests
test init_creates_project_files ... ok
test check_fails_without_init ... ok
test upgrade_on_current_version ... ok
test check_fails_on_conflicting_legacy_project_files ... ok
test init_fails_if_already_initialized ... ok
test list_stale_on_empty_project ... ok
test check_static_accepts_build_fetch_policy_without_fetching_remote_url ... ok
test check_detects_drift ... ok
test check_json_failure_stdout_is_parseable_report ... ok
test check_json_passes_on_fresh_project_with_bounded_non_claims ... ok
test check_passes_on_fresh_project ... ok
test check_detects_missing_inputs_file ... ok
test check_json_explicit_probe_and_trust_mode_labels_behavior ... ok
test show_on_empty_project ... ok
test refresh_hashes_local_patch_relative_to_project_root ... ok
test refresh_on_empty_project ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 34.16s
     Running tests/project_refresh_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_refresh_cli-448ee49216b77cda)

running 10 tests
test list_stale_reports_stale_and_failed_without_mutating_files ... ok
test refresh_partial_failure_writes_successes_and_exits_nonzero ... ok
test freshness_no_network_mode_does_not_contact_http_probe ... ok
test freshness_local_directory_probe_updates_lock_digest ... ok
test freshness_http_json_template_refreshes_selected_stale_input ... ok
test build_fetch_policy_refresh_uses_expected_hash_without_network_resolution ... ok
test freshness_git_ref_probe_reports_stale_without_mutating_lock ... ok
test refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes ... ok
test refresh_git_input_locks_resolved_rev_and_tree_hash ... ok
test freshness_command_missing_timeout_and_output_limit_fail_deterministically ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s


[test-status=0]
```

## Final formatting check

```text

[fmt-status=0]
```

## Final post-transcript Cairn validation

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}

[validate-status=0]
```
