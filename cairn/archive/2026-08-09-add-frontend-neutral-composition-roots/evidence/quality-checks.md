# Quality checks

Run on 2026-08-09.

## Focused checks

```text
operator command contract generator self-test: PASS
operator command contract: PASS (commands=172)
FOCUSED_QUALITY_STATUS=PASS
```

## Broad formatting visibility

Command: `nix develop -c cargo fmt --check -p mantle -v`

```text
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_compare.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_eval_backends.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_eval_smoke.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_lazy_eval.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_scheduler_priority.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_suite.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/hardware_simulation_plan.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/picolibc_compare.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/projects/delta-substitution/demo.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/projects/release-witness-handoff/demo.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/projects/shared-action-result-roundtrip/publish.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/lib.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/main.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/attest_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/audit_support.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/benchmark_harness.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/bootstrap_eval.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/bootstrap_parity_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/bootstrap_validate_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/cargo_free_self_build_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/cargo_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/composition_root_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/example_projects.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_build.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_eval.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_inventory.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_workflow_gallery.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/foreign_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/freshness_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/identity_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/integration.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/integration_build.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/kernel_bundle_oci_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/kernel_bundle_oci_registry_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/kernelscript_experiment.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/lock_importer_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/machine_schema_contracts.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/nix_free_demo_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/offline_build_runbook_docs.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/offline_cargo_project.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/operator_diagnostics.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/pin_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/project_build_smoke.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/project_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/project_refresh_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/release_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_credentials_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_rail_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_stdio_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_transfer_production.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/removed_system_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/retention_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/rust_compatibility_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/rust_plan_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/scheduling_policy.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/self_hosting.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/smoke.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/source_bundle_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/source_bundle_hydration_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/stdlib_tests.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/store_archive_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/store_gc_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/transcript_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/trust_policy_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/wasm_component_cli.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tools/generate_operator_command_contract.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tools/generate_portable_platform_profiles.rs"
rustfmt --edition 2024 --check /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_compare.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_eval_backends.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_eval_smoke.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_lazy_eval.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_scheduler_priority.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/benchmark_suite.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/hardware_simulation_plan.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/picolibc_compare.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/projects/delta-substitution/demo.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/projects/release-witness-handoff/demo.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/examples/projects/shared-action-result-roundtrip/publish.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/lib.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/main.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/attest_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/audit_support.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/benchmark_harness.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/bootstrap_eval.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/bootstrap_parity_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/bootstrap_validate_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/cargo_free_self_build_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/cargo_import_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/composition_root_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/example_projects.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_build.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_eval.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_inventory.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/examples_workflow_gallery.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/foreign_import_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/freshness_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/identity_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/integration.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/integration_build.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/kernel_bundle_oci_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/kernel_bundle_oci_registry_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/kernelscript_experiment.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/lock_importer_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/machine_schema_contracts.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/nix_free_demo_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/offline_build_runbook_docs.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/offline_cargo_project.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/operator_diagnostics.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/pin_import_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/project_build_smoke.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/project_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/project_refresh_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/release_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_credentials_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_rail_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_stdio_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/remote_transfer_production.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/removed_system_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/retention_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/rust_compatibility_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/rust_plan_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/scheduling_policy.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/self_hosting.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/smoke.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/source_bundle_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/source_bundle_hydration_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/stdlib_tests.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/store_archive_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/store_gc_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/transcript_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/trust_policy_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tests/wasm_component_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tools/generate_operator_command_contract.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/tools/generate_portable_platform_profiles.rs
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/source_built_fixed_point_shell.rs:18:
 use crate::errors::RunError;
 use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
 use crate::native_toolchain_closure::NativeToolchainClosureOptions;
[31m-use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m use crate::source_built_fixed_point::InitialOutputAuthorityState;
 use crate::source_built_fixed_point::ProofHermeticityMode;
[32m+use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m use crate::source_built_fixed_point::SourceAuthorityInput;
 use crate::source_built_fixed_point::SourceAuthorityRole;
 use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/source_built_fixed_point_shell.rs:28:
 use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
 use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
 use crate::source_built_fixed_point::SourceContentKind;
[31m-use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/source_built_fixed_point_shell.rs:39:
[32m+use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m use crate::source_built_fixed_point_dev_cache::FastFailDecision;
 use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
 use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/source_built_fixed_point_shell.rs:42:
[31m-use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[32m+use crate::source_bundle::SourceRecord;
(B[m use crate::source_bundle::assemble_source_bundle;
 use crate::source_bundle::materialize_source_record_payload;
 use crate::source_bundle::source_built_fixed_point_profile_records;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/source_built_fixed_point_shell.rs:47:
[31m-use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[31m-use crate::source_bundle::SourceRecord;
(B[m use crate::stagex_provider::StagexProviderRequest;
 use crate::stagex_transition::StagexTransitionRequest;
 
BROAD_FORMAT_STATUS=KNOWN_PRE_EXISTING_FAILURE
```

## Whole-tree Tiger Style visibility

Command: `CARGO_TARGET_DIR=/tmp/mantle-composition-tiger-target nix run .#tigerstyle -- check -- -p crunch-composition-core --all-targets`

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking bytes v1.11.1
    Checking bstr v1.12.1
    Checking aws-lc-rs v1.16.2
    Checking crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/crates/crunch-attestation-core)
    Checking url v2.5.8
    Checking zstd v0.13.3
    Checking chrono v0.4.44
    Checking serde_urlencoded v0.7.1
    Checking quick-xml v0.40.1
    Checking postcard v1.1.3
    Checking serde_qs v0.12.0
    Checking serde_tagged v0.3.0
    Checking petgraph v0.6.5
    Checking artifact-auth-core v0.1.0 (ssh://git@github.com/OnixResearch/onix-artifact.git?rev=c932138d880ddf4c2967f4c024b489b5c0022bf1#c932138d)
    Checking oci-spec v0.7.1
    Checking crunch-composition-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/crates/crunch-composition-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/crates/crunch-gc-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/crates/crunch-repair-core)
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/crates/crunch-overlay-core)
   Compiling zerocopy v0.8.48
   Compiling regex-automata v0.4.14
error: function `revalidate_overlay` has 0 assertion(s) in 22 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:222:103
    |
222 |   pub fn revalidate_overlay(expected: &OverlayPlan, observed: &OverlayPlan) -> Result<(), OverlayError> {
    |  _______________________________________________________________________________________________________^
223 | |     if expected.policy_id != observed.policy_id || expected.logical_prefix != observed.logical_prefix {
224 | |         return Err(OverlayError::CompositionDrift);
...   |
242 | |     Ok(())
243 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density
    = note: `-D tigerstyle::assertion-density` implied by `-D assertion-density`
    = help: to override `-D assertion-density` add `#[allow(tigerstyle::assertion_density)]`

error: function `validate_policy` has 0 assertion(s) in 25 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:245:72
    |
245 |   fn validate_policy(policy: &OverlayPolicy) -> Result<(), OverlayError> {
    |  ________________________________________________________________________^
246 | |     if policy.policy_id.is_empty() {
247 | |         return Err(OverlayError::EmptyPolicyId);
...   |
268 | |     Ok(())
269 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_observation` has 0 assertion(s) in 41 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:308:31
    |
308 |   ) -> Result<(), OverlayError> {
    |  _______________________________^
309 | |     if observation.declaration_index != expected_index {
310 | |         return Err(OverlayError::InvalidDeclarationIndex {
311 | |             expected: expected_index,
...   |
347 | |     validate_generation_members(policy, observation)
348 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_generation_members` has 0 assertion(s) in 41 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:350:115
    |
350 |   fn validate_generation_members(policy: &OverlayPolicy, observation: &BaseObservation) -> Result<(), OverlayError> {
    |  ___________________________________________________________________________________________________________________^
351 | |     if observation.members.is_empty() {
352 | |         return Err(OverlayError::EmptyGeneration {
353 | |             declaration_index: observation.declaration_index,
...   |
389 | |     Ok(())
390 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `descriptor_for_observation` has 0 assertion(s) in 22 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:403:105
    |
403 |   fn descriptor_for_observation(mut observation: BaseObservation) -> Result<BaseDescriptor, OverlayError> {
    |  _________________________________________________________________________________________________________^
404 | |     observation.members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
405 | |     let observed_bytes = observation.members.iter().try_fold(0_u64, |total, member| {
406 | |         total.checked_add(member.bytes).ok_or(OverlayError::GenerationBytesOverflow {
...   |
423 | |     })
424 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: `report` looks like a quantity but has no unit suffix
   --> crates/crunch-gc-core/src/retention.rs:333:13
    |
333 |     let mut report = UsageReport {
    |             ^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
    = note: `-D tigerstyle::numeric-units` implied by `-D numeric-units`
    = help: to override `-D numeric-units` add `#[allow(tigerstyle::numeric_units)]`
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
333 |     let mut report_<unit> = UsageReport {
    |                   +++++++

error: function `validate_policy` has 0 assertion(s) in 25 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:351:76
    |
351 |   fn validate_policy(policy: &RetentionPolicy) -> Result<(), RetentionError> {
    |  ____________________________________________________________________________^
352 | |     if policy.policy_id.is_empty() {
353 | |         return Err(RetentionError::EmptyPolicyId);
...   |
374 | |     Ok(())
375 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density
    = note: `-D tigerstyle::assertion-density` implied by `-D assertion-density`
    = help: to override `-D assertion-density` add `#[allow(tigerstyle::assertion_density)]`

error: function `normalize_roots` has 0 assertion(s) in 37 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:380:49
    |
380 |   ) -> Result<Vec<RetentionRoot>, RetentionError> {
    |  _________________________________________________^
381 | |     for root in &roots {
382 | |         validate_path_id(&root.path_id)?;
383 | |         if root.owner_scope.is_empty() {
...   |
415 | |     Ok(roots)
416 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `generation_ranks` has 0 assertion(s) in 38 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:418:97
    |
418 |   fn generation_ranks(roots: &[RetentionRoot]) -> Result<BTreeMap<String, usize>, RetentionError> {
    |  _________________________________________________________________________________________________^
419 | |     let mut groups = BTreeMap::<(RootClass, String, String, String), Vec<(u64, String)>>::new();
420 | |     for root in roots {
421 | |         if !matches!(root.class, RootClass::ProjectOutputGeneration | RootClass::ProjectSourceGeneration) {
...   |
455 | |     Ok(ranks)
456 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `decide_root` has 0 assertion(s) in 35 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:463:48
    |
463 |   ) -> Result<RetentionDecision, RetentionError> {
    |  ________________________________________________^
464 | |     let (mut disposition, mut reason) = match root.class {
465 | |         RootClass::ExplicitPin => (RetentionDisposition::Keep, RetentionReason::ExplicitPin),
466 | |         RootClass::ProjectOutputGeneration => generation_decision(
...   |
496 | |     })
497 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `lease_decision` has 0 assertion(s) in 31 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:522:70
    |
522 |   ) -> Result<(RetentionDisposition, RetentionReason), RetentionError> {
    |  ______________________________________________________________________^
523 | |     let Some(lease) = &root.lease else {
524 | |         return Err(RetentionError::MissingLease {
525 | |             path_id: root.path_id.clone(),
...   |
552 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: `lease_duration_is_unsafe` looks like a quantity but has no unit suffix
   --> crates/crunch-gc-core/src/retention.rs:535:9
    |
535 |     let lease_duration_is_unsafe = lease
    |         ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
535 |     let lease_duration_is_unsafe_<unit> = lease
    |                                 +++++++

error: function `retention_plan_id` has 0 assertion(s) in 37 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:558:46
    |
558 |   ) -> Result<RetentionPlanId, RetentionError> {
    |  ______________________________________________^
559 | |     let mut hasher = blake3::Hasher::new();
560 | |     hasher.update(RETENTION_PLAN_DOMAIN);
561 | |     hash_string(&mut hasher, &policy.policy_id)?;
...   |
593 | |     Ok(RetentionPlanId(*hasher.finalize().as_bytes()))
594 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `hash_root` has 0 assertion(s) in 26 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:596:95
    |
596 |   fn hash_root(hasher: &mut blake3::Hasher, root: &RetentionRoot) -> Result<(), RetentionError> {
    |  _______________________________________________________________________________________________^
597 | |     hash_string(hasher, &root.path_id)?;
598 | |     hash_string(hasher, root.class.as_str())?;
599 | |     hash_string(hasher, &root.owner_scope)?;
...   |
620 | |     Ok(())
621 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_objects` has 0 assertion(s) in 37 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:693:29
    |
693 |   ) -> Result<(), UsageError> {
    |  _____________________________^
694 | |     for pair in objects.windows(2) {
695 | |         if pair[0].object_id == pair[1].object_id {
696 | |             return Err(UsageError::DuplicateObject {
...   |
728 | |     Ok(())
729 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `accumulate_usage_object` has 0 assertion(s) in 35 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:736:29
    |
736 |   ) -> Result<(), UsageError> {
    |  _____________________________^
737 | |     let Some(bytes) = object.bytes else {
738 | |         report.unknown_object_count = report.unknown_object_count.checked_add(1).ok_or(UsageError::ByteOverflow)?;
739 | |         for root_id in &object.retaining_root_ids {
...   |
769 | |     Ok(())
770 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `usage_class` has 0 assertion(s) in 23 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:780:119
    |
780 |   fn usage_class(decisions: &BTreeMap<&str, &RetentionDecision>, root_ids: &[String]) -> Result<UsageClass, UsageError> {
    |  _______________________________________________________________________________________________________________________^
781 | |     if root_ids.is_empty() {
782 | |         return Ok(UsageClass::Reclaimable);
...   |
801 | |     Ok(UsageClass::Reclaimable)
802 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_links` has 0 assertion(s) in 27 lines (need 2+)
   --> crates/crunch-gc-core/src/lib.rs:371:30
    |
371 |   ) -> Result<(), GcPlanError> {
    |  ______________________________^
372 | |     for root in roots {
373 | |         if !entries_by_id.contains_key(root.as_str()) {
374 | |             return Err(GcPlanError::MissingRoot { path_id: root.clone() });
...   |
396 | |     Ok(())
397 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `compute_retaining_roots` has 0 assertion(s) in 27 lines (need 2+)
   --> crates/crunch-gc-core/src/lib.rs:424:49
    |
424 |   ) -> Result<Vec<GcRetainingRoots>, GcPlanError> {
    |  _________________________________________________^
425 | |     let mut roots_by_path = BTreeMap::<String, BTreeSet<String>>::new();
426 | |     for root in roots {
427 | |         let mut visited = BTreeSet::new();
...   |
449 | |         .collect())
450 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `prepare_composition` has 0 assertion(s) in 44 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:267:107
    |
267 |   pub fn prepare_composition(request: &CompositionRequest) -> Result<PreparedComposition, CompositionError> {
    |  ___________________________________________________________________________________________________________^
268 | |     validate_policy(&request.realization_policy)?;
269 | |     if request.plan.schema != PLAN_SCHEMA {
270 | |         return Err(CompositionError::InvalidPlanSchema);
...   |
312 | |     })
313 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density
    = note: `-D tigerstyle::assertion-density` implied by `-D assertion-density`
    = help: to override `-D assertion-density` add `#[allow(tigerstyle::assertion_density)]`

error: function `plan_composition` has 0 assertion(s) in 42 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:318:51
    |
318 |   ) -> Result<CompositionOutcome, CompositionError> {
    |  ___________________________________________________^
319 | |     let snapshots_by_root = index_snapshots(prepared, snapshots)?;
320 | |     let mut usage = LimitUsage {
321 | |         bindings: count_u32(prepared.bindings.len(), "bindings")?,
...   |
359 | |     })
360 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `finalize_receipt` has 0 assertion(s) in 28 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:366:51
    |
366 |   ) -> Result<RealizationReceipt, CompositionError> {
    |  ___________________________________________________^
367 | |     validate_root_ref(&resulting_root)?;
368 | |     let mut input_roots = prepared.bindings.iter().map(|binding| binding.root.clone()).collect::<Vec<_>>();
369 | |     input_roots.sort();
...   |
392 | |     })
393 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: `unsupported_metadata_classes` looks like a quantity but has no unit suffix
   --> crates/crunch-composition-core/src/lib.rs:371:9
    |
371 |     let unsupported_metadata_classes = vec_of_strings(&[
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
    = note: `-D tigerstyle::numeric-units` implied by `-D numeric-units`
    = help: to override `-D numeric-units` add `#[allow(tigerstyle::numeric_units)]`
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
371 |     let unsupported_metadata_classes_<unit> = vec_of_strings(&[
    |                                     +++++++

error: function `validate_policy` has 0 assertion(s) in 42 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:395:80
    |
395 |   fn validate_policy(policy: &RealizationPolicy) -> Result<(), CompositionError> {
    |  ________________________________________________________________________________^
396 | |     if policy.schema != POLICY_SCHEMA {
397 | |         return Err(CompositionError::InvalidPolicySchema);
...   |
435 | |     Ok(())
436 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: `limits` looks like a quantity but has no unit suffix
   --> crates/crunch-composition-core/src/lib.rs:399:9
    |
399 |     let limits = [
    |         ^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
399 |     let limits_<unit> = [
    |               +++++++

error: `bounded_limits` looks like a quantity but has no unit suffix
   --> crates/crunch-composition-core/src/lib.rs:417:9
    |
417 |     let bounded_limits = [
    |         ^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
417 |     let bounded_limits_<unit> = [
    |                       +++++++

error: function `normalize_path` has 0 assertion(s) in 24 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:471:76
    |
471 |   fn normalize_path(path: &str, is_root_allowed: bool) -> Result<String, ()> {
    |  ____________________________________________________________________________^
472 | |     if path.is_empty() {
473 | |         return if is_root_allowed { Ok(String::new()) } else { Err(()) };
...   |
493 | |     Ok(normalized.join("/"))
494 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: condition has 4 boolean operators
   --> crates/crunch-composition-core/src/lib.rs:475:8
    |
475 |       if path.starts_with(PATH_SEPARATOR)
    |  ________^
476 | |         || path.starts_with(WINDOWS_SEPARATOR)
477 | |         || path.ends_with(PATH_SEPARATOR)
478 | |         || path.contains(WINDOWS_SEPARATOR)
479 | |         || path.contains("//")
    | |______________________________^
    |
    = help: Tiger Style (Condition Decomposition): break multi-part conditions into named predicates or nested guards Example: if is_leader { if has_quorum { ... } }
    = note: see Tiger Style guide: Condition Decomposition
    = note: `-D tigerstyle::compound-condition` implied by `-D compound-condition`
    = help: to override `-D compound-condition` add `#[allow(tigerstyle::compound_condition)]`

error: function `index_snapshots` has 0 assertion(s) in 20 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:526:71
    |
526 |   ) -> Result<BTreeMap<CastoreRootRef, RootSnapshot>, CompositionError> {
    |  _______________________________________________________________________^
527 | |     let expected = prepared.bindings.iter().map(|binding| binding.root.clone()).collect::<BTreeSet<_>>();
528 | |     let mut indexed = BTreeMap::new();
529 | |     for snapshot in snapshots {
...   |
544 | |     Ok(indexed)
545 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_snapshot` has 0 assertion(s) in 38 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:553:35
    |
553 |   ) -> Result<(), CompositionError> {
    |  ___________________________________^
554 | |     usage.depth = usage.depth.max(depth);
555 | |     if usage.depth > policy.max_depth {
556 | |         return Err(CompositionError::LimitExceeded("depth"));
...   |
590 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `merge_directories` has 0 assertion(s) in 32 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:665:45
    |
665 |   ) -> Result<SnapshotNode, CompositionError> {
    |  _____________________________________________^
666 | |     let contributor_count = contributions.len();
667 | |     let mut children = BTreeMap::<String, Vec<Contribution>>::new();
668 | |     for contribution in contributions {
...   |
695 | |     Ok(SnapshotNode::Directory { entries })
696 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `resolve_collision` has 0 assertion(s) in 22 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:704:45
    |
704 |   ) -> Result<SnapshotNode, CompositionError> {
    |  _____________________________________________^
705 | |     let winner_ref = decisions.get(path).ok_or_else(|| CompositionError::UnresolvedCollision(path.to_string()))?;
706 | |     let winner = contributions
707 | |         .iter()
...   |
724 | |     Ok(winner.node.clone())
725 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `hash_receipt` has 0 assertion(s) in 29 lines (need 2+)
   --> crates/crunch-composition-core/src/lib.rs:777:39
    |
777 |   ) -> Result<String, CompositionError> {
    |  _______________________________________^
778 | |     let mut hasher = blake3::Hasher::new();
779 | |     hash_bytes(&mut hasher, RECEIPT_DOMAIN)?;
780 | |     hash_bytes(&mut hasher, prepared.plan_ref.as_bytes())?;
...   |
804 | |     Ok(format!("{RECEIPT_REF_PREFIX}{}", hasher.finalize().to_hex()))
805 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

   Compiling phf_shared v0.11.3
error: boolean binding `bounds_are_valid` should have a predicate prefix
   --> crates/crunch-overlay-core/src/lib.rs:252:9
    |
252 |     let bounds_are_valid = policy.max_base_layers > 0
    |         ^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
    = note: `-D tigerstyle::bool-naming` implied by `-D bool-naming`
    = help: to override `-D bool-naming` add `#[allow(tigerstyle::bool_naming)]`
help: rename binding to `is_bounds_are_valid`
    |
252 |     let is_bounds_are_valid = policy.max_base_layers > 0
    |         +++

error: boolean binding `valid` should have a predicate prefix
   --> crates/crunch-overlay-core/src/lib.rs:272:9
    |
272 |     let valid = logical_prefix.starts_with('/')
    |         ^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
help: rename binding to `is_valid`
    |
272 |     let is_valid = logical_prefix.starts_with('/')
    |         +++

   Compiling digest v0.10.7
error: could not compile `crunch-overlay-core` (lib) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
error: boolean binding `owner_is_eligible` should have a predicate prefix
   --> crates/crunch-gc-core/src/retention.rs:389:13
    |
389 |         let owner_is_eligible = policy
    |             ^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
    = note: `-D tigerstyle::bool-naming` implied by `-D bool-naming`
    = help: to override `-D bool-naming` add `#[allow(tigerstyle::bool_naming)]`
help: rename binding to `is_owner_is_eligible`
    |
389 |         let is_owner_is_eligible = policy
    |             +++

error: collection grows in loop without a prior reservation or explicit local bound
   --> crates/crunch-gc-core/src/retention.rs:451:13
    |
451 |             ranks.insert(path_id.clone(), current_rank);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Bounded Resource Growth): reserve collection capacity or add an explicit local length bound before growing it in a loop Example: let mut items = Vec::with_capacity(limit);
    = note: see Tiger Style guide: Bounded Resource Growth
    = note: `-D tigerstyle::unbounded-collection-growth` implied by `-D unbounded-collection-growth`
    = help: to override `-D unbounded-collection-growth` add `#[allow(tigerstyle::unbounded_collection_growth)]`

error: boolean binding `lease_duration_is_unsafe` should have a predicate prefix
   --> crates/crunch-gc-core/src/retention.rs:535:9
    |
535 |     let lease_duration_is_unsafe = lease
    |         ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
help: rename binding to `is_lease_duration_is_unsafe`
    |
535 |     let is_lease_duration_is_unsafe = lease
    |         +++

error: 2 consecutive `u64` parameters are easy to swap by accident
   --> crates/crunch-gc-core/src/retention.rs:804:1
    |
804 | / fn checked_add(left: u64, right: u64) -> Result<u64, UsageError> {
805 | |     left.checked_add(right).ok_or(UsageError::ByteOverflow)
806 | | }
    | |_^
    |
    = help: Tiger Style (Explicit Interfaces): replace same-type parameter clusters with named fields or an options struct Example: fn connect(opts: ConnectOptions) -> ...
    = note: see Tiger Style guide: Explicit Interfaces
    = note: `-D tigerstyle::ambiguous-params` implied by `-D ambiguous-params`
    = help: to override `-D ambiguous-params` add `#[allow(tigerstyle::ambiguous_params)]`

error: could not compile `crunch-gc-core` (lib) due to 18 previous errors
error: implicit default on external type — construct with explicit values
  --> crates/crunch-composition-core/src/lib.rs:60:5
   |
60 | /     #[serde(default, skip_serializing_if = "Option::is_none")]
61 | |     pub label: Option<String>,
   | |_____________________________^
   |
   = help: Tiger Style (Explicit Defaults): construct foreign-crate values explicitly instead of relying on `Default` Example: let client = reqwest::ClientBuilder::new();
   = note: see Tiger Style guide: Explicit Defaults
   = note: `-D tigerstyle::explicit-defaults` implied by `-D explicit-defaults`
   = help: to override `-D explicit-defaults` add `#[allow(tigerstyle::explicit_defaults)]`

error: implicit default on external type — construct with explicit values
  --> crates/crunch-composition-core/src/lib.rs:77:5
   |
77 | /     #[serde(default)]
78 | |     pub collision_decisions: Vec<CollisionDecision>,
   | |___________________________________________________^
   |
   = help: Tiger Style (Explicit Defaults): construct foreign-crate values explicitly instead of relying on `Default` Example: let client = reqwest::ClientBuilder::new();
   = note: see Tiger Style guide: Explicit Defaults

error: implicit default on external type — construct with explicit values
   --> crates/crunch-composition-core/src/lib.rs:108:5
    |
108 | /     #[serde(default, skip_serializing_if = "Option::is_none")]
109 | |     pub label: Option<String>,
    | |_____________________________^
    |
    = help: Tiger Style (Explicit Defaults): construct foreign-crate values explicitly instead of relying on `Default` Example: let client = reqwest::ClientBuilder::new();
    = note: see Tiger Style guide: Explicit Defaults

error: implicit default on external type — construct with explicit values
   --> crates/crunch-composition-core/src/lib.rs:163:5
    |
163 | /     #[serde(default, skip_serializing_if = "Option::is_none")]
164 | |     pub winner_binding_ref: Option<String>,
    | |__________________________________________^
    |
    = help: Tiger Style (Explicit Defaults): construct foreign-crate values explicitly instead of relying on `Default` Example: let client = reqwest::ClientBuilder::new();
    = note: see Tiger Style guide: Explicit Defaults

error: implicit default on external type — construct with explicit values
   --> crates/crunch-composition-core/src/lib.rs:165:5
    |
165 | /     #[serde(default)]
166 | |     pub displaced_binding_refs: Vec<String>,
    | |___________________________________________^
    |
    = help: Tiger Style (Explicit Defaults): construct foreign-crate values explicitly instead of relying on `Default` Example: let client = reqwest::ClientBuilder::new();
    = note: see Tiger Style guide: Explicit Defaults

error: collection grows in loop without a prior reservation or explicit local bound
   --> crates/crunch-composition-core/src/lib.rs:491:9
    |
491 |         normalized.push(component);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Bounded Resource Growth): reserve collection capacity or add an explicit local length bound before growing it in a loop Example: let mut items = Vec::with_capacity(limit);
    = note: see Tiger Style guide: Bounded Resource Growth
    = note: `-D tigerstyle::unbounded-collection-growth` implied by `-D unbounded-collection-growth`
    = help: to override `-D unbounded-collection-growth` add `#[allow(tigerstyle::unbounded_collection_growth)]`

error: 2 consecutive `&str` parameters are easy to swap by accident
   --> crates/crunch-composition-core/src/lib.rs:870:1
    |
870 | / fn is_ref(value: &str, prefix: &str) -> bool {
871 | |     value.strip_prefix(prefix).is_some_and(|digest| {
872 | |         digest.len() == BLAKE3_HEX_CHARS
873 | |             && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
874 | |     })
875 | | }
    | |_^
    |
    = help: Tiger Style (Explicit Interfaces): replace same-type parameter clusters with named fields or an options struct Example: fn connect(opts: ConnectOptions) -> ...
    = note: see Tiger Style guide: Explicit Interfaces
    = note: `-D tigerstyle::ambiguous-params` implied by `-D ambiguous-params`
    = help: to override `-D ambiguous-params` add `#[allow(tigerstyle::ambiguous_params)]`

error: 2 consecutive `&str` parameters are easy to swap by accident
   --> crates/crunch-composition-core/src/lib.rs:877:1
    |
877 | / fn join_path(parent: &str, child: &str) -> String {
878 | |     if parent.is_empty() {
879 | |         child.to_string()
880 | |     } else {
...   |
883 | | }
    | |_^
    |
    = help: Tiger Style (Explicit Interfaces): replace same-type parameter clusters with named fields or an options struct Example: fn connect(opts: ConnectOptions) -> ...
    = note: see Tiger Style guide: Explicit Interfaces

error: could not compile `crunch-composition-core` (lib) due to 22 previous errors
TIGERSTYLE_STATUS=KNOWN_PRE_EXISTING_FAILURE
```

The Tiger Style runner checks the workspace despite the package arguments. It stops on existing findings in `crates/crunch-gc-core` and `crates/crunch-overlay-core`. This change does not modify those crates.

## Post-review label-erasure quality rerun

```text

thread 'rustc' (556372) panicked at /rustc-dev/c756124775121dea0e640652c5ee3c89e3dd0eb4/compiler/rustc_middle/src/query/on_disk_cache.rs:663:9:
cannot decode `AttrId` with `CacheDecoder`
stack backtrace:
   0:     0x7ffff49bf8bb - <<std[e6f27331f023ae41]::sys::backtrace::BacktraceLock>::print::DisplayBacktrace as core[109e66eb34fb8c82]::fmt::Display>::fmt
   1:     0x7ffff501dd88 - core[109e66eb34fb8c82]::fmt::write
   2:     0x7ffff49d6b86 - <std[e6f27331f023ae41]::sys::stdio::unix::Stderr as std[e6f27331f023ae41]::io::Write>::write_fmt
   3:     0x7ffff4995bb8 - std[e6f27331f023ae41]::panicking::default_hook::{closure#0}
   4:     0x7ffff49b2f23 - std[e6f27331f023ae41]::panicking::default_hook
   5:     0x7ffff39b5c2c - std[e6f27331f023ae41]::panicking::update_hook::<alloc[eb388bbecdd9eb12]::boxed::Box<rustc_driver_impl[dc7f9df789ba0473]::install_ice_hook::{closure#1}>>::{closure#0}
   6:     0x7ffff49b3202 - std[e6f27331f023ae41]::panicking::panic_with_hook
   7:     0x7ffff4995caa - std[e6f27331f023ae41]::panicking::panic_handler::{closure#0}
   8:     0x7ffff498c9b9 - std[e6f27331f023ae41]::sys::backtrace::__rust_end_short_backtrace::<std[e6f27331f023ae41]::panicking::panic_handler::{closure#0}, !>
   9:     0x7ffff49976dd - __rustc[35c91ab2531d24f9]::rust_begin_unwind
  10:     0x7ffff1beba1c - core[109e66eb34fb8c82]::panicking::panic_fmt
  11:     0x7ffff64f606d - <rustc_middle[892f238719cdef02]::query::on_disk_cache::OnDiskCache>::load_side_effect
  12:     0x7ffff64f75cd - <rustc_query_impl[7f414e36c042cbbb]::dep_kind_vtables::non_query::SideEffect::{closure#0} as core[109e66eb34fb8c82]::ops::function::FnOnce<(rustc_middle[892f238719cdef02]::ty::context::TyCtxt, rustc_middle[892f238719cdef02]::dep_graph::dep_node::DepNode, rustc_middle[892f238719cdef02]::dep_graph::serialized::SerializedDepNodeIndex)>>::call_once
  13:     0x7ffff50a4c34 - <rustc_middle[892f238719cdef02]::dep_graph::graph::DepGraphData>::try_mark_previous_green
  14:     0x7ffff50a1de9 - <rustc_middle[892f238719cdef02]::dep_graph::graph::DepGraph>::try_mark_green
  15:     0x7ffff50a1c2e - rustc_query_impl[7f414e36c042cbbb]::execution::ensure_can_skip_execution::<rustc_middle[892f238719cdef02]::query::caches::DefaultCache<rustc_span[dd82765e8bfac38d]::def_id::LocalModDefId, rustc_middle[892f238719cdef02]::query::erase::ErasedData<[u8; 0usize]>>>
  16:     0x7ffff5fa9974 - rustc_query_impl[7f414e36c042cbbb]::query_impl::check_mod_deathness::execute_query_incr::__rust_end_short_backtrace
  17:     0x7ffff5fa9423 - rustc_interface[4db99434440230c0]::passes::analysis::{closure#0}::{closure#0}::{closure#1}
  18:     0x7ffff5fa7de2 - rustc_data_structures[b3c083c117c274b3]::sync::parallel::par_fns
  19:     0x7ffff5fa7d7c - rustc_interface[4db99434440230c0]::passes::analysis::{closure#0}::{closure#0}
  20:     0x7ffff5fa7de2 - rustc_data_structures[b3c083c117c274b3]::sync::parallel::par_fns
  21:     0x7ffff50a5c23 - rustc_interface[4db99434440230c0]::passes::analysis
  22:     0x7ffff657857e - rustc_query_impl[7f414e36c042cbbb]::execution::try_execute_query::<rustc_middle[892f238719cdef02]::query::caches::SingleCache<rustc_middle[892f238719cdef02]::query::erase::ErasedData<[u8; 0usize]>>, true>
  23:     0x7ffff6577eea - rustc_query_impl[7f414e36c042cbbb]::query_impl::analysis::execute_query_incr::__rust_end_short_backtrace
  24:     0x7ffff624081e - rustc_interface[4db99434440230c0]::interface::run_compiler::<(), rustc_driver_impl[dc7f9df789ba0473]::run_compiler::{closure#0}>::{closure#1}
  25:     0x7ffff61f24be - std[e6f27331f023ae41]::sys::backtrace::__rust_begin_short_backtrace::<rustc_interface[4db99434440230c0]::util::run_in_thread_with_globals<rustc_interface[4db99434440230c0]::util::run_in_thread_pool_with_globals<rustc_interface[4db99434440230c0]::interface::run_compiler<(), rustc_driver_impl[dc7f9df789ba0473]::run_compiler::{closure#0}>::{closure#1}, ()>::{closure#0}, ()>::{closure#0}::{closure#0}, ()>
  26:     0x7ffff61f2d60 - <std[e6f27331f023ae41]::thread::lifecycle::spawn_unchecked<rustc_interface[4db99434440230c0]::util::run_in_thread_with_globals<rustc_interface[4db99434440230c0]::util::run_in_thread_pool_with_globals<rustc_interface[4db99434440230c0]::interface::run_compiler<(), rustc_driver_impl[dc7f9df789ba0473]::run_compiler::{closure#0}>::{closure#1}, ()>::{closure#0}, ()>::{closure#0}::{closure#0}, ()>::{closure#1} as core[109e66eb34fb8c82]::ops::function::FnOnce<()>>::call_once::{shim:vtable#0}
  27:     0x7ffff61f3c6c - <std[e6f27331f023ae41]::sys::thread::unix::Thread>::new::thread_start
  28:     0x7fffefc92d53 - start_thread
  29:     0x7fffefd1a63c - __clone3
  30:                0x0 - <unknown>

error: the compiler unexpectedly panicked. This is a bug

note: we would appreciate a bug report: https://github.com/rust-lang/rust-clippy/issues/new?template=ice.yml

note: please make sure that you have updated to the latest nightly

note: please attach the file at `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-add-frontend-neutral-composition-roots-20260809/rustc-ice-2026-08-09T20_46_15-556351.txt` to your bug report

note: rustc 1.96.0-nightly (c75612477 2026-04-07) running on x86_64-unknown-linux-gnu

note: compiler flags: --crate-type lib -C embed-bitcode=no -C debuginfo=2 -C linker=clang -C incremental=[REDACTED] -C link-arg=-fuse-ld=mold -C link-arg=-fuse-ld=mold -C link-arg=-Wl,--allow-multiple-definition

note: some of the compiler flags provided by cargo are hidden

query stack during panic:
#0 [analysis] running analysis passes on crate `crunch_delta`
end of query stack
note: Clippy version: clippy 0.1.96 (c756124775 2026-04-07)

there was a panic while trying to force a dep node
try_mark_green dep node stack:
#0 check_mod_deathness(crunch_delta[faf7]::fixtures)
end of try_mark_green dep node stack
error: could not compile `crunch-delta` (lib)

Caused by:
  process didn't exit successfully: `/nix/store/p17m0phb203vw5ica88j5fm4gg8a87s5-cargo-rustc-kache-wrapper/bin/cargo-rustc-kache-wrapper /nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08/bin/clippy-driver rustc --crate-name crunch_delta --edition=2024 crates/crunch-delta/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("experimental-vectorcdc"))' -C metadata=9ee6ee472fbe5b89 -C extra-filename=-cbe116cc3303e62c --out-dir /tmp/mantle-composition-clippy-quality-20260809/debug/deps -C linker=clang -C incremental=/tmp/mantle-composition-clippy-quality-20260809/debug/incremental -L dependency=/tmp/mantle-composition-clippy-quality-20260809/debug/deps --extern async_trait=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libasync_trait-095d8dd496e21c59.so --extern blake3=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libblake3-456b0b659b336e84.rmeta --extern bytes=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libbytes-f445e9dbad7689fa.rmeta --extern crunch_delta_core=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libcrunch_delta_core-b67279f24544dcc0.rmeta --extern crunch_store=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libcrunch_store-b149830c7f5e7c7d.rmeta --extern ed25519_dalek=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libed25519_dalek-1e03b1175c6a8fe0.rmeta --extern futures=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libfutures-1faeeaee0954697d.rmeta --extern nix_compat=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libnix_compat-fa911d86cba56498.rmeta --extern serde=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libserde-a5246cf1bae3e5ee.rmeta --extern serde_json=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libserde_json-eb8ccb5a9de5af35.rmeta --extern snix_castore=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libsnix_castore-048efe0935e53f2e.rmeta --extern snix_store=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libsnix_store-131877f86162d050.rmeta --extern thiserror=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libthiserror-086697014b12f1e6.rmeta --extern tokio=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libtokio-3637c77fc87573d8.rmeta --extern zstd=/tmp/mantle-composition-clippy-quality-20260809/debug/deps/libzstd-cb1f9665f89880ec.rmeta -C link-arg=-fuse-ld=mold -C link-arg=-fuse-ld=mold -C link-arg=-Wl,--allow-multiple-definition -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/blake3-cb2c5a8904d829e6/out -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/blake3-cb2c5a8904d829e6/out -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/bzip2-sys-44cdd1d79070d045/out/lib -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/lzma-sys-b57c3dd6d864b26b/out -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/zstd-sys-f90c322e7d91975d/out -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/libmimalloc-sys-da69bf522e3d8cdb/out -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/aws-lc-sys-ce12bf0d110edff2/out -L native=/tmp/mantle-composition-clippy-quality-20260809/debug/build/ring-9d67a9ec6be42b1f/out` (exit status: 101)

## Post-review Clippy retry with a fresh target

The prior retry hit a nightly incremental-cache ICE in unchanged `crunch-delta`. This retry uses a new target directory.

```text
POST_REVIEW_QUALITY_STATUS=PASS
```
