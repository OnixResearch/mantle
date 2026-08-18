## `nix develop -c cargo test -p crunch-eval-budget-core`

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-eval-budget-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval-budget-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.31s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_eval_budget_core-bfa06e2488cb0cdd)

running 9 tests
test tests::diagnostics_truncate_on_item_byte_and_utf8_boundaries ... ok
test tests::framing_rejects_oversize_incomplete_and_trailing_data ... ok
test tests::terminal_classification_gives_cancellation_and_teardown_precedence ... ok
test tests::bounded_protocol_and_identity_properties_hold_across_fixture_matrix ... ok
test tests::policy_and_request_identity_are_deterministic ... ok
test tests::strict_support_and_policy_limits_fail_closed ... ok
test tests::request_rejects_oversize_duplicates_and_identity_drift ... ok
test tests::worker_response_validation_accepts_bound_facts_and_rejects_drift ... ok
test tests::reports_keep_metric_roles_and_non_claims_explicit ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_eval_budget_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exit_status=0
```

## `nix develop -c cargo test -p crunch-eval --lib`

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_eval-759bed47e32d6e82)

running 80 tests
test session::tests::effective_parallel_workers_clamps_requested_concurrency ... ok
test session::tests::clamp_parallel_workers_preserves_minimum_chunk_size ... ok
test session::tests::isolated_worker_input_is_send ... ok
test session::tests::normalize_concurrency_cap_clamps_to_requested_roots ... ok
test session::tests::normalize_concurrency_cap_defaults_zero_to_one ... ok
test session::tests::resolve_execution_backend_uses_inline_for_single_worker_requests ... ok
test session::tests::small_all_root_sets_reuse_the_session_even_when_threading_is_allowed ... ok
test stdlib::tests::source_stdlib_candidates_empty_without_inputs ... ok
test session::tests::select_worker_labels_keeps_requested_subset_and_indexes ... ok
test stdlib::tests::stdlib_import_path_returns_dir_with_lib_ncl ... ok
test stdlib::tests::source_stdlib_candidates_include_cwd_and_exe_ancestors ... ok
test tests::eval_file_nonexistent_returns_io_error ... ok
test session::tests::u32_from_usize_rejects_overflow_without_a_sentinel ... ok
test stdlib::tests::embedded_stdlib_matches_repo_lib_directory ... ok
test stdlib::tests::stdlib_import_path_can_force_embedded_copy ... ok
test stdlib::tests::write_stdlib_creates_all_files ... ok
test stdlib::tests::write_stdlib_content_matches_embedded ... ok
test session::tests::discover_static_record_labels_iteratively_unwraps_nested_lets ... ok
test session::tests::discover_static_record_labels_rejects_single_derivation_shape ... ok
test session::tests::discover_static_record_labels_extracts_let_wrapped_record_fields ... ok
test session::tests::force_selected_roots_bounded_reports_failed_label ... ok
test stdlib::tests::write_stdlib_skips_rewrite_when_unchanged ... ok
test tests::eval_enum_tags ... ok
test tests::eval_not_exported_stripped ... ok
test tests::eval_simple_record ... ok
test session::tests::discover_record_labels_returns_boundary_error_for_stale_shape_without_panic ... ok
test tests::eval_file_simple_record ... ok
test tests::eval_defaults_applied ... ok
test tests::eval_file_resolves_import_from_parent_dir ... ok
test tests::eval_file_extra_import_path ... ok
test tests::eval_str_and_deserialize_enum_tags ... ok
test tests::eval_contract_violation ... ok
test session::tests::force_selected_roots_bounded_preserves_requested_order ... ok
test tests::eval_merge ... ok
test tests::eval_str_and_deserialize_nested_records ... ok
test tests::eval_str_and_deserialize_flat_record ... ok
test tests::eval_recursive_record ... ok
test tests::eval_serde_deserialize ... ok
test session::tests::force_root_returns_boundary_error_for_inconsistent_cached_shape_without_panic ... ok
test tests::eval_str_and_deserialize_type_mismatch_returns_serde_error ... ok
test tests::eval_file_to_json_returns_valid_json ... ok
test tests::eval_and_deserialize_non_record_returns_serde ... ok
test tests::eval_error_display_has_context ... ok
test tests::session_missing_root_returns_error ... ok
test tests::eval_str_and_extract_named_roots_array_missing_name_returns_boundary_error ... ok
test tests::session_record_discovers_labels_from_field_names ... ok
test tests::derivation_contract_defaults_dynamic_plan_outputs ... ok
test tests::derivation_deserialization_rejects_undeclared_dynamic_plan_output ... ok
test tests::derivation_contract_accepts_declared_dynamic_plan_outputs ... ok
test tests::session_type_lives_in_crunch_eval ... ok
test stdlib::tests::stdlib_is_importable ... ok
test stdlib::tests::written_embedded_stdlib_is_importable ... ok
test session::tests::prefer_threaded_falls_back_to_inline_when_threaded_executor_is_unavailable ... ok
test session::tests::force_root_isolated_matches_same_session_force_root ... ok
test session::tests::inline_executor_executes_assignments_without_threaded_backend_help ... ok
test tests::eval_str_and_extract_named_roots_rejects_non_root_shape_with_boundary_error ... ok
test session::tests::force_selected_roots_policy_inline_matches_prefer_threaded ... ok
test session::tests::force_all_roots_bounded_matches_serial_path ... ok
test tests::eval_str_and_extract_named_roots_single_derivation ... ok
test tests::eval_str_and_extract_named_roots_nested_derivation_enum_tags ... ok
test tests::eval_str_and_extract_named_roots_source_input_uses_manual_input_deserializer ... ok
test tests::eval_str_typecheck_failure_returns_eval ... ok
test tests::session_invalid_top_level_shape_returns_error ... ok
test tests::session_file_with_imports ... ok
test tests::eval_str_syntax_error_returns_eval ... ok
test tests::session_does_not_affect_eval_to_json ... ok
test tests::eval_str_and_extract_named_roots_package_set_preserves_keys ... ok
test tests::eval_str_and_extract_named_roots_array_uses_derivation_names ... ok
test tests::eval_str_and_extract_named_roots_output_selection_uses_manual_input_deserializer ... ok
test tests::session_nonselected_force_count_is_zero_for_single_root ... ok
test tests::session_file_based_evaluation ... ok
test tests::session_array_discovers_labels_from_derivation_names ... ok
test tests::session_single_derivation_discovers_label_from_name ... ok
test tests::session_force_root_matches_eager_path ... ok
test tests::session_force_all_roots_matches_eager_path ... ok
test tests::session_recursive_record_refs_match_eager ... ok
test tests::session_nested_derivation_inputs_match_eager ... ok
test tests::session_array_force_matches_eager ... ok
test tests::session_import_heavy_fixture_matches_eager ... ok
test tests::session_single_derivation_force_matches_eager ... ok

test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s


exit_status=0
```

## `nix develop -c cargo test -p mantle --test benchmark_harness --test evaluator_budget_cli`

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-eval-budget-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval-budget-core)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 06s
     Running tests/benchmark_harness.rs (/home/brittonr/.cargo-target/debug/deps/benchmark_harness-d0a7283660cf7890)

running 25 tests
test benchmark_support::tests::compare_bundles_report_sparse_metric_omissions_per_workload ... ok
test benchmark_support::tests::compare_bundles_matches_workloads_and_metrics ... ok
test benchmark_support::tests::compare_bundles_do_not_fabricate_missing_phase_metrics ... ok
test benchmark_support::tests::compare_rejects_threshold_application_across_mismatched_resource_cohorts ... ok
test benchmark_support::tests::build_result_allows_workloads_without_phase_metrics ... ok
test benchmark_support::tests::compare_bundles_highlight_largest_regressions_and_wins ... ok
test benchmark_support::tests::compatible_cohorts_apply_named_memory_thresholds_and_report_missing_values ... ok
test benchmark_support::tests::default_benchmark_paths_stay_under_target ... ok
test benchmark_support::tests::record_root_count_rejects_drift ... ok
test benchmark_support::tests::sum_samples_ns_adds_all_samples ... ok
test benchmark_support::tests::render_bundle_json_keeps_schema ... ok
test toml_section_keeps_dotted_subsections ... ok
test benchmark_support::tests::compare_benchmark_files_reads_fixed_fixtures ... ok
test benchmark_entry_points_are_checked_in_examples ... ok
test benchmark_docs_cover_all_workloads_and_entry_points ... ok
test benchmark_runtime_boundary_stays_out_of_library_path ... ok
test benchmark_support::tests::suite_workload_descriptors_cover_required_kinds ... ok
test eval_smoke_benchmark_writes_machine_readable_bundle ... ok
test benchmark_support::tests::suite_workload_setup_is_deterministic ... ok
test suite_workload_descriptors_stay_deterministic ... ok
test suite_has_checked_in_multi_phase_workflow_result ... ok
test store_metrics_only_appear_on_store_aware_workload ... ok
test lazy_selected_root_nonselected_force_count_is_zero ... ok
test suite_phase_metrics_cover_phase_two_scope ... ok
test suite_benchmark_writes_full_workload_matrix_bundle ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s

     Running tests/evaluator_budget_cli.rs (/home/brittonr/.cargo-target/debug/deps/evaluator_budget_cli-f0e913c94db95466)

running 14 tests
test hidden_worker_is_not_listed_in_public_help ... ok
test deadline_is_terminal_and_worker_is_reaped ... ok
test discovered_root_limit_fails_with_its_stable_class ... ok
test cancellation_is_terminal_and_reaps_a_late_worker ... ok
test observe_only_keeps_in_process_path_and_marks_operation_scoped_metrics_unavailable ... ok
test invalid_policy_fixtures_fail_before_report_publication ... ok
test simulated_reap_failure_blocks_a_clean_terminal_claim ... ok
test strict_worker_matches_existing_eval_and_reports_supported_resources ... ok
test strict_worker_confines_imports_to_admitted_roots ... ok
test process_fixtures_classify_crash_signal_protocol_overflow_and_memory ... ok
test worker_preserves_declared_imports_and_bounded_evaluator_errors ... ok
test selected_and_all_root_requests_report_honest_force_counts ... ok
test process_fixtures_bound_stderr_and_make_late_results_terminal ... ok
test cpu_exhaustion_reaches_the_enforced_cpu_terminal_class ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.87s


exit_status=0
```

## `nix develop -c cargo clippy -p crunch-eval-budget-core --all-targets --no-deps -- -D warnings`

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking crunch-eval-budget-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval-budget-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s

exit_status=0
```

## `nix develop -c cargo clippy -p mantle --examples --no-deps -- -D warnings`

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking crunch-eval-budget-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval-budget-core)
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.67s

exit_status=0
```

## `git diff --check`

```text

exit_status=0
```
