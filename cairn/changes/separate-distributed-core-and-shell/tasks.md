## Phase 1: Baseline and inventory

- [x] [serial] Inventory distributed decision types, vendor types, ports, concrete adapters, async orchestration, callers, and compatibility surfaces. r[realization_routing.distributed_core_boundary]
  - Evidence: `evidence/inventory.md` maps each deterministic decision to its pre-change and post-change owner, including vendor-coupled and deferred surfaces.
- [x] [parallel] Record current positive and negative route, resolver, publisher, remote-realizer, and external-batch behavior before migration. r[realization_routing.distributed_boundary_validation] r[external_batch_dispatchers.boundary_validation]
  - Evidence: `evidence/inventory.md` behavioral-baseline section; baseline `cargo test -p crunch-build --lib distributed::` green (130 passed) before and after the boundary work.

## Phase 2: Pure distributed core

- [x] [serial] Define Mantle-owned bounded artifact, route, operation, observation, and typed domain-error values. r[realization_routing.distributed_core_boundary]
  - Evidence: `crates/crunch-build/src/distributed/core.rs` owns `RemoteBuildServiceRequestFacts`, `RequestValidationError`, `FallbackPolicy`, `FallbackDecision`, `GoalAttachmentPlan`, and `AttachmentError`.
- [x] [depends:distributed-domain-values] Move route, reuse, fallback, admission, and outcome decisions into pure functions over explicit facts. r[realization_routing.distributed_core_boundary]
  - Evidence: `core::validate_remote_build_service_request`, `core::classify_failure`, and `core::plan_goal_attachments` own the decisions; the shell (`distributed.rs`) projects vendor values into core facts and maps typed decisions back. Public signatures and error types stay byte-compatible.
- [x] [parallel] Add positive and negative direct tests without async runtimes, stores, providers, processes, or vendor DTOs. r[realization_routing.distributed_boundary_validation]
  - Evidence: `cargo test -p crunch-build --lib distributed::core::` 13 passed, 0 failed (request, classification, attachment, bound, order, determinism).

## Phase 3: Application ports and adapters

- [ ] [depends:distributed-domain-values] Define application-owned ports for resolution, publication, realization, and external batch operations. r[realization_routing.distributed_shell_ownership] r[external_batch_dispatchers.port_error_ownership]
- [ ] [depends:distributed-application-ports] Add Snix, store, direct-process, Slurm, local, remote, and in-memory projections outside the pure boundary. r[realization_routing.distributed_core_boundary]
- [ ] [depends:pure-distributed-decisions] Move resolver order, async calls, retries, time, cancellation, observation recording, and effect execution into the shell. r[realization_routing.distributed_shell_ownership]
- [ ] [depends:distributed-application-ports] Replace raw external batch failures with typed infrastructure failures and explicit application outcomes. r[external_batch_dispatchers.port_error_ownership]
- [ ] [serial] Select concrete distributed and batch adapters only at validated composition roots. r[realization_routing.distributed_shell_ownership]

## Phase 4: Compatibility and validation

- [ ] [serial] Preserve accepted PathInfo semantics, protocol bytes, route decisions, reports, and BLAKE3 identities through explicit compatibility projections. r[realization_routing.distributed_core_boundary]
- [ ] [parallel] Add shell-order tests for denial, unavailable service, timeout, cancellation, ambiguity, stale response, and no silent fallback. r[realization_routing.distributed_boundary_validation]
- [ ] [parallel] Add adapter tests for valid and malformed Snix/store, direct-process, and Slurm values. r[external_batch_dispatchers.boundary_validation]
- [ ] [parallel] Add dependency and source-shape guards for the declared pure distributed boundary and composition roots. r[realization_routing.distributed_core_boundary] r[realization_routing.distributed_shell_ownership]
  - Partial: `scripts/check-distributed-core-boundary.rs` now guards the pure boundary with positive and negative fixtures (`self-test-ok`, `core-boundary-ok`). The composition-root guard leg remains open.
- [ ] [serial] Run focused tests, workspace tests, formatting, Clippy, first-party quality rails, Cairn, Tracey, and relevant Nix checks. r[realization_routing.distributed_boundary_validation] r[external_batch_dispatchers.boundary_validation]
- [ ] [serial] Record evidence, sync accepted specs, archive the change, and rerun validation. r[realization_routing.distributed_boundary_validation]
