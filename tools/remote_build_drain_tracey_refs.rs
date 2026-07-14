// Tracey bridge for the near-complete remote-build drain cohort.
//
// Requirement sources are the reviewed change specs while changes are active
// and the matching archived specs after completion. Every link below points to
// inspected implementation, positive/negative tests, and the change-owned
// validation transcript. These references do not promote diagnostic, cache,
// workspace, resource, or locality evidence into build correctness, execution
// success, trust, confidentiality, optimal placement, or release eligibility.

// Remote failure debug bundles.
// Core: `crates/crunch-build/src/distributed/remote_failure_debug.rs`.
// Shell/tests: remote bundle store, sandbox capture, coordinator, report, and
// CLI modules named in the change evidence.
// Evidence: `cairn/changes/emit-remote-failure-debug-bundles/evidence/implementation-validation.
// md`. r[impl operator_diagnostics.remote_failure_debug_bundle]
// r[verify operator_diagnostics.remote_failure_debug_bundle]
// r[impl operator_diagnostics.remote_failure_replay]
// r[verify operator_diagnostics.remote_failure_replay]
// r[impl remote_builds.failure_debug_capture]
// r[verify remote_builds.failure_debug_capture]

// Shared action results.
// Core: `crates/crunch-action-result-core`.
// Shell/tests: `crates/crunch-store`, `crates/crunch-build`,
// `crates/crunch-pipeline`, root reports, and typed Nickel fixtures.
// Evidence: `cairn/changes/publish-shared-action-results/evidence/validation.md`.
// r[impl build_correctness.shared_action_result_records]
// r[verify build_correctness.shared_action_result_records]
// r[impl build_correctness.shared_action_result_admission]
// r[verify build_correctness.shared_action_result_admission]
// r[impl cache_substitution.shared_action_result_discovery]
// r[verify cache_substitution.shared_action_result_discovery]

// Stateful tool workspaces.
// Core: `crates/crunch-build/src/workspace.rs`.
// Shell/tests: workspace shell, sandbox transport, remote coordinator, reports,
// typed Nickel policy, and clean-rebuild comparison fixtures.
// Evidence: `cairn/changes/isolate-stateful-tool-workspaces/evidence/validation-transcript.txt`.
// r[impl build_correctness.stateful_workspace_modes]
// r[verify build_correctness.stateful_workspace_modes]
// r[impl build_correctness.mutable_workspace_claim_boundary]
// r[verify build_correctness.mutable_workspace_claim_boundary]
// r[impl remote_builds.stateful_workspace_leases]
// r[verify remote_builds.stateful_workspace_leases]

// Quantified resources and verified locality.
// Core: `crates/crunch-build/src/distributed/remote_resources.rs`.
// Shell/tests: `src/remote_build.rs`, scheduling, remote-farm config, reports,
// and typed Nickel resource/locality fixtures.
// Evidence: `cairn/changes/account-scarce-resources-and-locality/evidence/
// implementation-validation.md`. r[impl build_scheduling.quantified_resource_admission]
// r[verify build_scheduling.quantified_resource_admission]
// r[impl build_scheduling.verified_locality_placement]
// r[verify build_scheduling.verified_locality_placement]
// r[impl remote_builds.fenced_worker_resource_leases]
// r[verify remote_builds.fenced_worker_resource_leases]
