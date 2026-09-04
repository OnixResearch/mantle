# Tasks

## 1. Baseline and data contracts

- [x] [serial] 1.1 Run existing scheduler, resource declaration, lease, report, result-discovery, and accounting tests before core changes. r[remote_builds.resource_observations]
- [x] [serial] 1.2 Define versioned resource-observation and action-family identity schemas. r[remote_builds.resource_observations]
- [x] [serial] 1.3 Define named sample, age, margin, retry, class, charge, time, and measurement bounds. r[remote_builds.replayable_resource_selection] r[remote_builds.positive_oom_retry]
- [x] [serial] 1.4 Add positive observations and negative missing, stale, incompatible, oversized, and untrusted fixtures. r[remote_builds.resource_observations]

## 2. Build the pure selection core

- [x] [serial] 2.1 Add typed policy inputs, eligible machine classes, quota facts, observations, decisions, and reason codes. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.2 Enforce declared minima and required architecture, platform, KVM, trust, and isolation features. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.3 Add deterministic tie-breaking and canonical replay serialization. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.4 Add positive selection tests and negative no-class, stale-history, quota-denial, and feature-mismatch tests. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.5 Add property tests for determinism, monotonic minima, stable ordering, and bounded output. r[remote_builds.replayable_resource_selection]

## 3. Add bounded OOM recovery

- [x] [serial] 3.1 Define trusted positive OOM evidence for each supported worker platform. r[remote_builds.positive_oom_retry]
- [x] [serial] 3.2 Implement the pure escalation planner with named retry, class, charge, and wall-time limits. r[remote_builds.positive_oom_retry]
- [x] [serial] 3.3 Create each retry as a new fenced attempt with predecessor linkage. r[remote_builds.positive_oom_retry]
- [x] [serial] 3.4 Add positive OOM escalation tests and negative arbitrary-failure, ambiguous-exit, exhausted-limit, unavailable-class, and quota-denial tests. r[remote_builds.positive_oom_retry]

## 4. Add accounting and quotas

- [x] [serial] 4.1 Add explicit project and account reservation and reconciliation records. r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 4.2 Make ledger mutations idempotent by attempt identity. r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 4.3 Add bounded API summaries with public units and reason codes. r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 4.4 Add positive charge tests and negative duplicate, cancellation, worker-loss, partial-observation, overflow, and store-failure tests. r[remote_builds.usage_reservation_and_reconciliation]

## 5. Add authorized result sharing

- [x] [serial] 5.1 Add private, project, and explicitly named sharing-scope policy. r[remote_builds.authorized_result_sharing]
- [x] [serial] 5.2 Require request identity, producer signature, policy and platform compatibility, authorized scopes, output identity, and CAS availability. r[remote_builds.authorized_result_sharing]
- [x] [serial] 5.3 Record producer-to-consumer evidence links. r[remote_builds.authorized_result_sharing]
- [x] [serial] 5.4 Add positive same-project and explicit-sharing fixtures plus negative signature, scope, platform, policy, identity, and CAS fixtures. r[remote_builds.authorized_result_sharing]

## 6. Build benchmarks and fault evidence

- [x] [serial] 6.1 Review `nixbench` workload patterns and licenses before adapting any fixture. r[remote_builds.resource_benchmark_evidence]
- [x] [serial] 6.2 Add fixed-input workloads with public source, license, platform, outcome, and measurement metadata. r[remote_builds.resource_benchmark_evidence]
- [x] [serial] 6.3 Compare observe-only policy decisions with the declared static baseline. r[remote_builds.resource_benchmark_evidence] r[remote_builds.resource_policy_rollout]
- [x] [serial] 6.4 Add ChaosControl campaigns for OOM, worker loss, duplicate events, accounting interruption, and CAS failure. r[remote_builds.positive_oom_retry] r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 6.5 Export observations and non-claims through Valence and gate them through Cairn. r[remote_builds.resource_benchmark_evidence]

## 7. Roll out and validate

- [x] [serial] 7.1 Add observe-only mode and project opt-in controls. r[remote_builds.resource_policy_rollout]
- [x] [serial] 7.2 Add independent rollback controls for selection, retry, quota enforcement, and sharing. r[remote_builds.resource_policy_rollout]
- [x] [serial] 7.3 Run `cargo fmt --all -- --check`. r[remote_builds.replayable_resource_selection]
  - Evidence: the committed focused run passed formatting with no diff.
- [x] [serial] 7.4 Run focused positive, negative, replay, property, benchmark, and fault tests. r[remote_builds.resource_benchmark_evidence]
  - Evidence: 15 core, 6 application, 3 shell-adapter, 1 coordinator, and 6 integration tests passed.
- [x] [serial] 7.5 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`. r[remote_builds.replayable_resource_selection]
  - Evidence: the broad test reached one missing-`bwrap` environment failure, and its exact rerun passed after restoring the documented path. Broad Clippy stopped in vendored `fuse-backend-rs` and `nix-compat`; strict first-party root and new-crate Clippy passed.
- [x] [serial] 7.6 Run the relevant Nix flake checks. r[remote_builds.resource_benchmark_evidence]
  - Evidence: the architecture, core, core-WASM, and integration checks passed.
- [x] [serial] 7.7 Run Cairn validation, requirement coverage, design gate, and tasks gate. r[remote_builds.resource_policy_rollout]
  - Evidence: lifecycle transcripts are in `evidence/committed-validation-2026-09-04/` and the later sync/archive evidence directories.
