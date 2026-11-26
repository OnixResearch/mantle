# Scheduler–rebuilder contract

Contract identifier: `mantle-scheduler-rebuilder-contract-v1`.

Mantle separates build scheduling (`Worker`, `GoalRegistry`, `scheduling.rs`) from action identity, cache admission, and execution (`Builder`, `ca_input_resolution.rs`, `action_result.rs`, `crunch-action-result-core`), adapting the Build Systems à la Carte separation between scheduler ordering/re-execution and rebuilder/trace verification.

The scheduler owns dependency readiness, goal deduplication, dispatch ranking, dynamic consumer wait attachment, invocation attempt ownership, and watch retraction/teardown. The rebuilder owns derivation resolution, action identity computation, shared and local cache admission, sandbox execution, output-content verification, and signed admission-evidence publication.

Scheduling policy, diagnostic reports, causal traces, and unauthenticated hints never authorize output reuse and never alter task or output identity.

---

## 1. Readiness, dispatch slots, deduplication, and preflight

### 1.1 Goal states and checked transitions

`GoalRegistry` (`crates/crunch-build/src/goal.rs`) bounds tracked goals at `MAX_GOALS = 32_768`. Every goal transitions strictly through `GoalState` (`crates/crunch-build/src/goal.rs`):

| State | Meaning | Allowed incoming transitions | Allowed outgoing transitions |
|---|---|---|---|
| `AwaitingDerivation` | Dynamic `.drv` placeholder waiting for a producer output | `Goal::new_awaiting` | `set_derivation` -> `Pending`; `notify_dep_failed` -> `Failed` |
| `Pending` | Derivation known; dependencies not yet inspected | `Goal::new` / `Goal::new_root` / `set_derivation` | `inspect` -> `Ready` (0 unbuilt deps) or `Waiting { remaining_deps }` |
| `Waiting { remaining_deps }` | Waiting on `remaining_deps > 0` upstream or bound plan roots | `inspect`; `attach_binding_roots` adds bound plan roots | `notify_dep_done` -> `Ready` (when `remaining_deps` reaches 0); `notify_dep_failed` -> `Failed` |
| `Ready` | All known dependencies are `Done`; eligible for `select_next_ready_goal` | `inspect`; `notify_dep_done` | `mark_building` -> `Building`; `mark_build_failed` -> `Failed` (inline `prepare_build` error or cancellation) |
| `Building` | Owned by a spawned build task tracked in `pending_meta` | `mark_building` | `mark_done` -> `Done`; `mark_build_failed` -> `Failed` |
| `Done` | Terminal success; outcome recorded and consumers notified | `mark_done` | Terminal (watch retraction may prune unreachable goals) |
| `Failed` | Terminal failure; dependents fail via `propagate_failure` | `notify_dep_failed`; `mark_build_failed` | Terminal (`notify_dep_done` is a no-op `Ok(false)`) |

### 1.2 Dispatch readiness vs resource slots and priority

Readiness is a graph property (`GoalState::Ready`), distinct from execution capacity (`Worker::max_jobs` and `Semaphore` capped at `MAX_IN_FLIGHT = 64` in `crates/crunch-build/src/worker.rs`) and dispatch order (`crates/crunch-build/src/scheduling.rs`):

1. `Worker::dispatch_ready` (`crates/crunch-build/src/worker.rs`) settles `late_roots`, computes `available_build_slots = self.max_jobs.checked_sub(running_jobs)`, and stops if `available_build_slots == 0` or `streaming_cancelled()` is true.
2. Callers normalize eligibility and preference facts via `normalize_eligible_preference` (`crates/crunch-build/src/scheduling.rs`) before `Worker::set_goal_scheduling_preference` (`goal_preferences`, copied into `ready_goals` by `enqueue_ready_goal`). `Worker::select_next_ready_goal` evaluates only goals currently in `ready_goals`, refreshing known-graph pressure via `refresh_pressure_cache` (`recompute_affected_pressure`), ranking via `rank_ready_goals` under `SchedulingPolicy`, and recording bounded `PriorityDecisionEvidence` via `priority_decision_evidence` for observation only.
3. Before calling `Builder::prepare_build`, `dispatch_ready` re-runs `resolve_binding_states`; any bound consumer with `binding.failure.is_some()` or an unrealized `realized_path` fails closed without dispatching to `Builder`.
4. `Builder::prepare_build` (`crates/crunch-build/src/orchestrate.rs`) decides whether the goal finishes inline (`PrepareResult::Done`) or requires a sandbox build (`PrepareResult::NeedsBuild`). `available_build_slots` is decremented **only** when `handle_prepare_result` spawns a task (`is_spawned == true`), reserving a `pending_meta` slot and acquiring the `Semaphore` permit inside `spawn_prepared_build`.

### 1.3 Goal deduplication and root upgrades

`Worker::want` and `create_goals_bfs` (`crates/crunch-build/src/worker.rs`) deduplicate goals by canonical absolute `.drv` path (`sp.to_absolute_path()`):

- Repeated `want` calls for an existing goal never insert a duplicate `Goal` or re-schedule an active goal.
- If `root && !goal.is_root`, `goal.is_root` is upgraded to `true` and `pressure_dirty_goals` is marked.
- If the upgraded goal is already `Done` or `Failed`, its key is queued in `self.late_roots` and settled once via `settle_late_roots` (`export_late_root` for `Done`, or emitting a single `FailedGoal` for `Failed`).
- Non-root goals already present in `self.satisfied_drvs` skip build-only prerequisite expansion (`should_expand_deps = root || !self.satisfied_drvs.contains(&key)`).

### 1.4 Root preflight precondition (issue `#41` boundary)

Any root-level preflight fast path (satisfying an eligible root without expanding build-only prerequisites) MUST satisfy all of the following preconditions before bypassing dependency expansion:

- The root is immutable and input-addressed (`!content_addressed`, `!is_mutable_workspace`, `bound_plan_outputs.is_empty()`, and no declared dynamic-plan outputs).
- Output admission passes the full `check_shared_action_result` or `check_cache` trust boundary (`Builder::prepare_build`), never index or `ca_mappings.json` presence alone.
- The root's runtime closure (`PathInfo.references`) is verified complete and materialized in the local/configured store; any missing closure node, untrusted signature, or ambiguity falls back to standard dependency expansion in `Worker::want`.

---

## 2. Five identity roles and invalidation rules

Mantle distinguishes five independent cryptographic identity roles so execution bindings, CA realisations, output blobs, and trust evidence never collapse into one another:

| Role | Identifier / Domain | Source & Storage | Invalidated by |
|---|---|---|---|
| 1. Evaluated derivation | `drv_path` (`StorePath<String>`) & `hash_derivation_modulo` (`[u8; 32]`) | `crunch_glue::convert` & `crunch_glue::resolve_derivation_registration` (`crates/crunch-glue/src/convert.rs`; `structural_recipe_digest` is the internal `ConversionCache` deduplication digest), `RegistryEntry` (`crates/crunch-build/src/registry.rs`) | Any change to builder, system, args, env (including bound execution profile `__MANTLE_FOREIGN_EXECUTION_PROFILE_V1` via `bind_execution_profile`), `input_sources`, `input_derivations` (selected output names or parent HDMs), or `outputs` |
| 2. Resolved derivation | `mantle-resolved-derivation://blake3/...`, domain `b"mantle.resolved-derivation.v1"` | `ca_input_resolution::resolve_ca_inputs` (`crates/crunch-build/src/ca_input_resolution.rs`), recorded via `DerivationRegistry::record_resolved_derivation` | Any change to realized CA parent store paths (`AdmittedResolvedInput`) or unresolved parent HDMs (`remaining_input_hdms`); CA-independent graphs keep `identity: None` |
| 3. Action identity | `action_ref` (`mantle-action://blake3/...`), domains `b"mantle.action-spec.derivation-adapter.v1"` and `b"mantle.action-spec.bound-plan-roots.v1"` | `action_result::action_ref_for_derivation` & `action_ref_for_bound_roots` (`crates/crunch-build/src/action_result.rs`), recorded via `DerivationRegistry::record_resolved_action_ref` | Any change to resolved derivation ATerm bytes or canonical sorted `(placeholder, realized_store_path)` pairs in `bound_plan_outputs`; reuse also scopes by `policy_refs_for_derivation` |
| 4. Output-content identity | `object_ref` (`mantle-object://blake3/...`), `PathInfo`, `path_info_ref` (`mantle-pathinfo://blake3/...`), `output_set_digest` | `crates/crunch-build/src/action_result.rs`, `crates/crunch-action-result-core/src/lib.rs`, `final_ca_path_info_facts` (`crates/crunch-build/src/orchestrate.rs`) | Any change to castore root `Node` digest/structure, store path, references, NAR digest/size, or CA self-reference normalization |
| 5. Admission-evidence identity | `result_ref` (`mantle-action-result://blake3/...`), `SignedActionResultRecord`, `action_receipt_ref`, policy/scan/signature refs | `crates/crunch-action-result-core/src/lib.rs`, `crates/crunch-build/src/action_result.rs` | Any change to `ActionResultRecord` fields, `ActionResultTrustPolicy`, detached record signatures, `PathInfo` signatures, or policy/receipt refs |

### Preserving original recipe identity vs execution bindings

In `Builder::prepare_build` (`crates/crunch-build/src/orchestrate.rs`), `collect_resolvable_input_facts` and `resolve_ca_inputs` resolve floating CA and CA-resolved input-addressed parent edges using `admit_resolved_input_realisation` (`AdmittedResolvedInput`). When a static consumer binds dynamic-plan roots (`bound_plan_outputs` is non-empty):

- `execution_derivation` clones the resolved derivation and extends `execution.input_sources` with the realized bound plan root store paths.
- `action_ref_for_bound_roots` hashes `action_ref_for_derivation(resolved.derivation.as_ref(), store_dir)` together with the canonical sorted `(placeholder, store_path)` bindings (`execution_derivation` is used only for sandbox input staging and `derivation_to_build_request`).
- The scheduler's `Goal::drv_path` and `RegistryEntry::derivation` keep the **original** evaluated derivation identity; execution-time bound roots never rewrite the evaluated `.drv` key.

---

## 3. Reuse decisions, execution, rejection, and non-authority

`Builder::prepare_build` (`crates/crunch-build/src/orchestrate.rs`) evaluates reuse strictly in this order:

1. **CA input admission** (`collect_resolvable_input_facts` -> `admit_resolved_input_realisation` -> `resolve_ca_inputs`): every floating CA or CA-resolved input-addressed parent edge (`parent.content_addressed || is_parent_resolved`) must supply an admitted, signed `ActionResultRecord` whose `outputs[name].store_path` agrees with the completed parent's `RegistryEntry::resolved_outputs` (for CA parents) or `RegistryEntry::resolved_derivation` output path (for CA-resolved input-addressed parents; cf. `DerivationRegistry::get_output_path`). Missing admission facts fail closed with `Error::Store("ca-input-unrealized: ...")` (and bound violations with `Error::Store("ca-resolution-limit: ...")`), while untrusted candidates, path disagreement, or conflicting signed realisations fail closed with `Error::CacheVerification` (`"ca-realisation-untrusted: ..."` / `"ca-realisation-conflict: ..."`). (Bound plan roots in `bound_plan_outputs` are checked separately via `Worker::resolve_binding_states` and incorporated into `execution_derivation.input_sources` and `action_ref_for_bound_roots`.)
2. **Strong shared reuse** (`check_shared_action_result`, skipped only when `is_mutable_workspace` is true):
   - `collect_shared_action_candidates` discovers candidate records for `action_ref`, probes castore/PathInfo availability (`probe_outputs`), and builds bounded `CandidateAdmissionFacts`.
   - `crunch_action_result_core::plan_strong_reuse` deduplicates by `result_ref`, validates record schemas, output sets, policy refs, `ActionResultTrustPolicy`, and detached signatures, and detects conflicting output sets (`admitted_by_output.len() > 1` -> `CONFLICTING_ACTION_RESULTS`).
   - On conflict: input-addressed actions fail closed with `Error::CacheVerification("conflicting-action-results: ...")`; content-addressed or CA-resolved actions record `conflict_class = Some("ca-realisation-conflict")` (falling through to re-execution during `check_shared_action_result`, and failing closed in `admit_resolved_input_realisation`).
   - On admitted reuse: `store.admit_action_result_outputs` materializes and verifies outputs, returning `PrepareResult::Done(BuildOutcome { cached: true, ... })`.
3. **Local/PathInfo reuse** (`check_cache`):
   - Eligible **only** when `!is_mutable_workspace && bound_plan_outputs.is_empty() && !is_ca && resolved.identity.is_none()`.
   - Verifies castore completeness via `store.check_cache` and, unless `trust_unsigned || store.is_overlay_composed()`, validates `PathInfo` signatures (`signing::verify_pathinfo_signatures_with_store_dir`). Untrusted or incomplete local entries return `Ok(None)` (cache miss) and fall through to execution.
4. **Execution** (`PrepareResult::NeedsBuild`):
   - Ensures input nodes, stages sandbox inputs and `execution_profile`, attaches `bind_plan_output_request` and bounded `collect_pool_closure` (`MAX_POOL_GOAL_CLOSURE_BYTES = 67_108_864`), and returns `NeedsBuild`.
   - On completion, `Builder::finish_build` normalizes CA outputs, signs `PathInfo`, and publishes `SignedActionResultRecord` before returning `BuildOutcome`. Failure to publish required CA action-result admission evidence fails closed with `Error::CacheVerification("ca-realisation-untrusted: ...")`.

### Explicit non-authority rules

The following artifacts are advisory or diagnostic only and MUST NEVER independently authorize reuse, alter task identity, or suppress execution:

- `ActionResultIndex` (`REQUIRED_NON_CLAIM_INDEX_NOT_TRUST = "index-presence-is-not-output-trust"` in `crates/crunch-action-result-core/src/lib.rs`): discovery hint only; every candidate still requires full signature, policy, `PathInfo`, and castore blob verification.
- `ca_mappings.json` (`CaMappings` in `crates/crunch-store/src/ca_mapping.rs`; non-claim `"ca-mapping-presence-is-not-output-trust"` via `REQUIRED_NON_CLAIM_CA_MAPPING_NOT_TRUST` in `crates/crunch-action-result-core/src/lib.rs` and `NON_CLAIM_CA_MAPPING` in `crates/crunch-build/src/action_result.rs`): restart hint only; never authorizes CA resolution or skips execution without a verified `SignedActionResultRecord` and complete `PathInfo`.
- `CausalTraceCollector` (`crates/crunch-build/src/causal_trace.rs`) and `BuildJsonReport` (`src/build_report.rs`): observation-only; never read by scheduler dispatch or rebuilder admission.
- `SchedulingPolicy` (`crates/crunch-build/src/scheduling.rs`): priority ordering only; never alters `drv_path`, `resolved_drv_path`, `action_ref`, `BuildOutcome`, or reuse rejection.

---

## 4. Dynamic completion ordering and late consumer resolution

Both fresh sandbox completions (`handle_build_completion` -> `finish_build` -> `handle_completed_outcome`) and inline cache hits (`handle_prepare_result` -> `PrepareResult::Done` -> `handle_completed_outcome`) execute the exact same admission pipeline in `Worker::handle_completed_outcome` (`crates/crunch-build/src/worker.rs`) **before** `complete_goal` notifies static waiters:

1. `declared_native_dynamic_outputs(&outcome, known_paths)` identifies declared plan outputs on the producer.
2. `scan_native_dynamic_plans` parses bounded v1/v2 native dynamic plan JSON from admitted outputs and classifies accepted and rejected plans.
3. For each accepted plan, `admit_v2_source_slices` verifies and publishes source slices before any registry or goal mutation; slice rejection converts the plan into `native_scan.rejected`.
4. `register_accepted_native_dynamic_plan` inserts all admitted units (`DerivationRegistry::insert_registry_ready_dynamic`; pre-staged and preflighted before mutation for v2, topological order for v1), calls `Worker::want(root, known_paths, true)` for exported roots only, and retains `NativePlanBindingFact::Accepted` in `self.native_plan_binding_facts`.
5. Every rejected plan records `NativePlanBindingFact::Rejected` in `self.native_plan_binding_facts`.
6. **Pre-completion consumer attachment**: while the producer goal is still in `Building` or `Ready` (prior to `complete_goal`), `handle_completed_outcome` locates all static consumers in `plan_output_bindings` referencing this producer, runs `resolve_binding_states`, and calls `attach_binding_roots` to add `remaining_deps += 1`, `waitees`, and `waiters` for every non-`Done` generated root. Consequently, `complete_goal` decrementing the producer dependency edge cannot release a static consumer into `Ready` while any generated dynamic root remains incomplete.
7. `detect_dynamic_derivations` scans non-native `.drv` outputs (`dynamic.rs` 4-stage pipeline: `ParsedDynamicDerivation` -> `ValidatedDynamicDerivation` -> `IdentityResolvedDynamicDerivation` -> `RegistryReadyDynamicDerivation`), activates `AwaitingDerivation` goals, and enqueues discovered roots.
8. **Producer completion**: `complete_goal` transitions the producer to `Done` and decrements waiter counts.
9. **Post-completion failure check**: if any bound consumer recorded a binding failure (`PlanRejected`, `RootMissing`, `OutputMissing`, `RootFailed`, `ProducerFailed`, `Undeclared`), `fail_binding_consumer` immediately transitions it to `Failed` without dispatch.

### Late consumer arrival

When a static consumer arrives via `Worker::want` after the dynamic-plan producer has already completed (`Done` or `Failed`), `Worker::want` runs `resolve_binding_states` and `wire_deps_and_inspect` against retained `native_plan_binding_facts` and realized root outputs in `DerivationRegistry::get_output_path`. Incomplete roots are attached as `Waiting` dependencies; rejected plans, failed producers, or failed roots immediately attach `PlanOutputBindingFailureKind` so `dispatch_ready` fails the consumer without deadlock or dispatch.

---

## 5. Invocation ownership, retraction, cancellation, and watch updates

### 5.1 Attempt ownership and bounds

- `Worker` executes a single invocation (`run`, `run_streaming_observed`, `run_streaming_cancellable`, or `run_watch`).
- Every spawned sandbox task owns a unique entry in `state.pending_meta: HashMap<String, PreparedBuild>` keyed by `drv_key` (`spawn_prepared_build` asserts `replaced.is_none()` before spawning; `JoinSet` executes at most one active attempt per goal key).
- `process_join_result` and `handle_build_completion` remove the `pending_meta` entry before admitting results or recording failure metadata; missing `pending_meta` fails with `Error::Store("BUG: ... has no pending metadata ...")` without mutating `Goal` state or writing `PathInfo`.
- Batch and streaming `Worker` invocations execute each goal at most once per invocation (no automatic retry loop on failed goals). Legitimate failures are retained in `state.failed` (for roots) and `dependency_failures` / `dependency_failure_logs` so `settle_late_roots` reports any late-requested failed root once without re-running it.

### 5.2 Watch generation protocol and kernel teardown

`Worker::run_watch` (`crates/crunch-build/src/worker.rs`) enforces a fence between retracted generations and new dispatch:

1. `watch_preflight_update` validates goal/registry capacities and derivation identity immutability before mutating live goals (`WatchWorkerEvent::Rejected` rolls back cleanly while preserving active state).
2. `watch_begin_update` admits new roots, calls `GoalRegistry::watch_retract_roots` (`crates/crunch-build/src/goal.rs`) to identify unreachable stale keys, and triggers per-goal cancellation via `watch_request_cancellation(&stale)` (`build_cancellations`).
3. Kernel-level sandbox teardown (`vendor/snix-build/src/bwrap/watch_cancel.rs`) requires `pidfd_open` support before spawn (`require_pidfd_support`), holds the guest behind `WatchStartGate` (`--block-fd`) until `OwnedOuterBwrap` and `OwnedPidNamespaceLeader` bind the inner PID1 pidfd, and terminates + waits via `pidfd_send_signal(SIGKILL)` + `poll(POLLIN)` and `wait_outer_exit` before releasing `WatchLease` (`OwnedSemaphorePermit` + `WatchWorkspace`). Unbound sandboxes enter `UnobservedWatchSandbox` quarantine.
4. `watch_join` checks whether the completing `drv_key` is in the sorted `stale` slice (`stale.binary_search(key).is_ok()`). Stale completions remove `pending_meta`, emit diagnostic `cancelled`/`cleaned_up` trace events, and **never** call `process_join_result`, `finish_build`, `complete_goal`, or publish `PathInfo`.
5. `watch_finish_retraction` verifies no stale key remains in `pending_meta`, prunes stale goal/scheduler/binding/report entries (`watch_remove_stale`), and prunes unreferenced derivation/source generation metadata (`watch_retain_metadata`).
6. Only after teardown completes does `watch_loop` emit `WatchWorkerEvent::CandidateReady { generation, commit }`; `dispatch_ready` runs only when `active == Some(generation)` and no retraction is pending, and superseded `commit` tokens are ignored.

---

## 6. Bounded explanations and compatibility rules

### 6.1 Observation surfaces

- `BuildJsonReport` (`src/build_report.rs`): exports bounded operational summaries including `scheduler_policy`, `action_result_reports` (`ActionResultRuntimeReport`), `native_dynamic_plans`, `plan_output_bindings`, `scheduler_priority_decisions` (`PriorityDecisionEvidence`), `finish_gates`, `pool_goal_observations`, `outcomes`, and `failed`.
- `CausalTraceCollector` (`crates/crunch-build/src/causal_trace.rs`): emits `mantle-build-trace-v1` traces bounded by `MAX_ACTIONS`, hashes goal keys using `logical_store_prefix`, redacts diagnostics via `mantle_causal_trace_core::redact_diagnostic`, and validates parent chains via `finish()`.
- Both surfaces are strictly observation-only; disabling, enabling, or capping them cannot alter `GoalState`, dispatch eligibility, cache admission, or `BuildOutcome`.

### 6.2 Compatibility rules (documentation vs hashed semantics)

- **Documentation and test revisions**: edits to `docs/scheduler-rebuilder-contract.md`, ADRs under `adr/`, comments, or `#[cfg(test)]` suites MUST NOT alter `drv_path`, `hash_derivation_modulo`, `resolved_drv_path`, `action_ref`, `object_ref`, `PathInfo`, `path_info_ref`, or `result_ref`.
- **Semantic or wire-format revisions**: any change that alters ATerm encoding, `RESOLVED_IDENTITY_DOMAIN` (`b"mantle.resolved-derivation.v1"`), `ACTION_DOMAIN`, `BOUND_ACTION_DOMAIN`, `RECORD_DOMAIN`, `ACTION_RESULT_SCHEMA`, `ACTION_RESULT_POLICY_SCHEMA`, or persisted state schemas MUST bump the explicit domain/schema version constant and provide migration or fail-closed rejection for prior versions.

---

## 7. Non-claims and related contracts

This contract governs the single-invocation scheduler–rebuilder boundary inside Mantle (`crates/crunch-build`, `crates/crunch-action-result-core`, `crates/crunch-glue`, and watch sandbox teardown). It does not claim:

- External proof authority beyond admitted signatures/policies (`#14`), remote worker pool placement semantics (`#35`), multi-host cache service availability, or full hardware/compiler determinism of untrusted builders.
- Root preflight shortcut implementation (`#41`, whose preconditions are specified in §1.4) or future observability/reporting features (`#19`, `#21`). See `docs/bounded-reference-oracle.md` for the `#40` bounded reference evaluator, differential `proptest` suite, and evidence-tier boundaries.
