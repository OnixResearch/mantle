# 0171. Specify and enforce the scheduler–rebuilder contract

Status: Accepted

## Context

Mantle's build execution splits responsibilities across two layers:

- The **scheduler** (`Worker`, `GoalRegistry`, and `scheduling.rs` in `crates/crunch-build`), which manages goal deduplication, `GoalState` readiness, dispatch slots, priority ordering, dynamic consumer dependency attachment, attempt ownership, and watch retraction/teardown.
- The **rebuilder** (`Builder`, `ca_input_resolution.rs`, `action_result.rs` in `crates/crunch-build`, `crunch-action-result-core`, and `crunch-glue`), which manages derivation identity, CA input resolution, shared and local cache admission, sandbox execution, output normalization, and signed admission-evidence publication.

As dynamic derivations, native dynamic plans with static consumer bindings (`ADR 0103`), CA input resolution (`ADR 0098`), shared action-result reuse (`ADR 0024`), watch retraction (`ADR 0093`), deterministic priority scheduling (`ADR 0017`), and causal traces (`ADR 0096`) evolved across separate changes, the cross-boundary invariants between scheduler and rebuilder were distributed across source comments and crate-local tests without a single versioned contract document or end-to-end boundary checks covering cached dynamic producers, bound-plan identity swaps, watch replacement teardown, and scheduling-policy non-interference.

## Decision Drivers

- Specify all six boundary areas required by issue `#39` (readiness/deduplication/`#41` preflight precondition, the five identity roles, reuse and non-authority, dynamic completion ordering, attempt ownership/watch teardown, and bounded explanations/compatibility) in one canonical versioned contract.
- Preserve existing public APIs (`GoalState`, `BuildOutcome`, `FailedGoal`, `EvalMessage`, `crunch-action-result-core` public structs) and preserve exact existing cryptographic hashes (`drv_path`, `resolved_drv_path`, `action_ref`, `object_ref`, `PathInfo`, `result_ref`).
- Enforce boundary invariants with lightweight internal guards and focused regression tests covering every identified runtime seam gap without adding `#[allow(...)]` attributes or relaxing TigerStyle gates.

## Decision

We codify `mantle-scheduler-rebuilder-contract-v1` in `docs/scheduler-rebuilder-contract.md` as the normative specification for Mantle's scheduler–rebuilder boundary, and enforce its runtime seams in `crates/crunch-build`:

1. **Public-API-safe boundary guards in `crates/crunch-build/src/worker.rs`**:
   - Enforce `pending_meta` ownership on the `Err(build_err)` branch of `Worker::process_join_result` (matching `handle_build_completion`) so an unowned failure completion cannot mutate `Goal` state.
   - Add `debug_assert!` guards in `attach_binding_roots` (consumer must be `Waiting` or `Failed` before attaching a non-`Done` root) and `watch_join` (`stale` slice must be sorted before `binary_search`).
2. **Focused boundary regression tests in `crates/crunch-build/src/{orchestrate,worker}.rs`**:
   - Mismatched identities: swapping `bound_plan_outputs` recomputes `resolved_action_ref`, bypasses `check_cache`, and misses prior `SignedActionResultRecord` while leaving `drv_path` unchanged; `admit_resolved_input_realisation` rejects resolved path disagreement with a completed CA parent.
   - Incomplete admission facts: an indexed `ActionResultRecord` missing its castore blob or carrying an untrusted `PathInfo` signature is rejected (`admitted == false`); input-addressed `conflicting-action-results` fail closed (`Error::CacheVerification`); `CausalTraceCollector` enablement/caps never alter `BuildOutcome` or reuse.
   - Premature consumer release: a **cached** (`PrepareResult::Done`, `cached: true`) native dynamic-plan producer executes `scan_native_dynamic_plans` -> `register_accepted_native_dynamic_plan` -> `attach_binding_roots` before `complete_goal` wakes a static consumer (covering both accepted plans where the consumer remains `GoalState::Waiting` until the dynamic root completes, and rejected cached plans failing the consumer with zero `do_build` calls).
   - Stale completion ownership and watch replacement: `process_join_result` rejects completions missing `pending_meta` without mutating goals or admitting `PathInfo`; `Worker::run_watch` root replacement (gen-1 -> gen-2) discards a late `Ok(BuildResult)` from retracted gen-1 during teardown and completes gen-2 cleanly.
   - Scheduling-policy non-interference: executing the same graph (including a CA leaf and an untrusted-signature cached leaf) under distinct `SchedulingPolicy` configurations changes dispatch order while producing identical `BuildOutcome`s, identities, and cache rejections.
3. **Compatibility rule**: Documentation, ADR, and test changes under this decision keep zero hash changes to derivations, actions, objects, `PathInfo`, or action results; any future semantic change to serialized or hashed structures must bump its explicit domain/schema version.

## Alternatives Considered

- **Refactor `GoalState` and public report DTOs around a new unified scheduler trait**: Rejected because downstream callers (`crates/crunch-pipeline`, `src/remote_build/live_state.rs`) embed/exhaustively match `GoalState` and serialize public outcome/report types, and existing boundaries already cleanly enforce separation without breaking downstream callers.
- **Documentation-only specification without runtime seam guards or cached-producer / policy-non-interference tests**: Rejected because `process_join_result`'s `Err` path lacked symmetric `pending_meta` ownership enforcement and cached dynamic-plan producers and end-to-end `SchedulingPolicy` non-interference lacked dedicated runtime tests.

## Consequences

- `docs/scheduler-rebuilder-contract.md` provides a single versioned reference for `#39` and defines the precondition for future `#41` root preflight work.
- All boundary guards and regression tests live inside `crates/crunch-build` (`#[cfg(test)]` + private `worker.rs` invariants), preserving public API compatibility and producing zero hash changes.
- Non-claims: This decision does not implement `#41` root preflight, alter external proof trust (`#14`), or change remote worker-pool placement semantics (`#35`).
