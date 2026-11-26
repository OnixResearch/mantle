# Output Selection Specification

## Purpose

Defines how consuming derivations reference specific outputs of
multi-output dependencies.

## Requirements

### Requirement: Selective Output Dependency

A derivation MUST be able to depend on a single named output of a
multi-output dependency, so that only that output is mounted in the
build sandbox.

#### Scenario: Select dev output

- GIVEN a multi-output derivation `libfoo` with outputs `["out", "dev", "lib"]`
- WHEN a consuming derivation includes `mantle.select libfoo "dev"` in its inputs
- THEN `input_derivations` maps `libfoo`'s drv path to `{"dev"}`
- AND only `libfoo`'s `dev` output path is mounted in the sandbox

#### Scenario: Bare derivation selects all outputs

- GIVEN a multi-output derivation `libfoo` with outputs `["out", "dev"]`
- WHEN a consuming derivation includes `libfoo` directly in its inputs
- THEN `input_derivations` maps `libfoo`'s drv path to `{"out", "dev"}`
- AND both output paths are mounted in the sandbox

### Requirement: Invalid Output Rejection

The glue layer MUST reject output selections that name a nonexistent
output at convert time.

#### Scenario: Nonexistent output name

- GIVEN a derivation `libfoo` with outputs `["out", "dev"]`
- WHEN a consuming derivation selects output `"headers"` from `libfoo`
- THEN conversion fails with an error naming the invalid output and listing the available outputs

### Requirement: Coalescing Duplicate Selections

The system MUST coalesce duplicate selected-output dependencies into a
single `input_derivations` entry with the union of selected outputs when
the same dependency appears multiple times in `inputs`.

#### Scenario: Two selections from same dep

- GIVEN a derivation `libfoo` with outputs `["out", "dev", "lib"]`
- WHEN a consumer's inputs include both `mantle.select libfoo "dev"` and `mantle.select libfoo "lib"`
- THEN `input_derivations` maps `libfoo` to `{"dev", "lib"}`

### Requirement: Select Helper Function

The stdlib MUST provide a function `mantle.select` that takes a
derivation record and an output name string, returning a structured
record suitable for use in the `inputs` array.

#### Scenario: Helper produces correct structure

- GIVEN `let pkg = { name = "foo", builder = "/bin/sh", outputs = ["out", "dev"], ... } | mantle.Derivation`
- WHEN evaluating `mantle.select pkg "dev"`
- THEN the result is `{ drv = pkg, output = "dev" }`

### Requirement: Input Contract Extension

The Nickel `Input` contract MUST accept three forms:
1. A string (source store path)
2. A derivation record (all outputs)
3. A record with `drv` and `output` fields (selected output)

#### Scenario: Mixed inputs array

- GIVEN inputs `[seed.bash, dep_pkg, mantle.select dep_pkg "dev"]`
- WHEN the array is validated against `Array Input`
- THEN all three entries pass validation

### Requirement: Selected-Output Scheduling Demand Propagation

The build scheduler (`Worker`) MUST track demanded output sets (`BTreeSet<String>`) per goal (`goal_demands`) and satisfied output sets (`satisfied_outputs`) per completed goal, propagating selected output names from `derivation.input_derivations` down dependency edges and coalescing live consumer unions. `Worker::want_outputs` MUST accept an optional selected output set (`None` for all declared outputs), and `Worker::want` MUST delegate to `Worker::want_outputs(..., None)` so unqualified roots retain all-output semantics.

#### Scenario: Concurrent and sequential consumers coalesce demand

- GIVEN a multi-output producer `sdk` with outputs `["out", "dev"]`
- WHEN `consumer_out` selects `"out"` and `consumer_dev` selects `"dev"` before or during scheduling
- THEN `goal_demands[sdk]` becomes the union `{"out", "dev"}`
- AND `sdk` executes at most once to produce both outputs before either consumer dispatches

#### Scenario: Worker boundary rejects empty or nonexistent output selections

- GIVEN a derivation `sdk` with outputs `["out", "dev"]`
- WHEN `Worker::want_outputs` or a consumer's `input_derivations` edge (including in `watch_preflight_update`) requests `""`/`<empty>` or `"headers"`
- THEN scheduling fails closed before dispatch with `Error::InvalidOutputSelection` naming the invalid output and listing available outputs `out`, `dev`

### Requirement: Late-Consumer Widening and Goal Reopening

When new demand arrives for an already-tracked goal whose `satisfied_outputs` do not cover the widened demand, the scheduler MUST satisfy the widened demand without leaving stale `Done` assumptions:
- Active goals (`AwaitingDerivation`, `Pending`, `Waiting`, `Ready`, `Building`) MUST union the new outputs into `goal_demands` (with `Building` producing all declared outputs on completion).
- Completed (`Done`) goals missing newly demanded outputs MUST reopen (`Done -> Pending` via `Goal::reopen_for_demand` without altering `GoalState` variants), clear stale `satisfied_outputs` and `late_roots`, expand any previously unexpanded dependencies, and re-wire waiting consumers.
- Bound-plan consumers (`PlanOutputBinding`) MUST widen demand on bound dynamic root outputs before treating a `Done` root as realized.
- Watch-mode retraction (`watch_finish_retraction`) MUST prune `goal_demands` and `satisfied_outputs` for retracted stale keys, and `WorkerResult` / watch `Settled` MUST upsert outcomes by `drv_path` and retain only `GoalState::Done` goals.

#### Scenario: Streaming or watch late consumer reopens Done goal

- GIVEN `sdk` (`["out", "dev"]`) completed `Done` with `satisfied_outputs = {"out"}` (serving `consumer_out` from cache)
- WHEN a later `consumer_dev` selecting `sdk:dev` arrives via `run_streaming` or a subsequent `run_watch` generation
- THEN `sdk` reopens (`Done -> Pending -> Ready`), executes once to produce `dev`, and `consumer_dev` waits on `sdk` until `dev` is satisfied

### Requirement: Demand-Projected Cache Reuse and Closure Verification

`Builder::prepare_build_with_demand` MUST thread `demanded_outputs` to `Builder::check_cache` while keeping `drv_path`, `resolved.path`, `action_ref`, `RegistryEntry::derivation`, and `BuildRequest` unchanged. For eligible immutable input-addressed derivations, `Builder::check_cache` MUST probe only the demanded outputs using a demand-projected temporary `Derivation` (`is_root = false` during verification), run a bounded fixed-point sibling loop (`<= derivation.outputs.len()`) to pull in and verify any sibling outputs referenced by `PathInfo.references`, verify signatures and transitive non-sibling runtime closure completeness (`resolve_closure` + per-member `check_cache` and signature verification), and export root outputs only after all verification succeeds. On any cache miss, missing sibling/closure member, or untrusted signature, `Builder::check_cache` MUST fail closed (`Ok(None)`) and fall through to unmodified full-output `NeedsBuild` execution.

#### Scenario: Reuse trusted out when unreferenced dev is missing

- GIVEN an immutable input-addressed `sdk` (`["out", "dev"]`) where signed `sdk:out` is cached and `sdk:dev` is absent
- WHEN `consumer` selects only `sdk:out`
- THEN `Builder::check_cache` reuses `sdk:out` with zero producer builds (`cached = true`)
- AND `consumer` mounts only `sdk:out` in its sandbox

#### Scenario: Implicit sibling or runtime reference must be present and trusted

- GIVEN an immutable input-addressed `sdk` (`["out", "lib", "dev"]`) where `sdk:out`'s `PathInfo.references` includes `sdk:lib` (or external runtime dependency `ext_dep`) and excludes `sdk:dev`
- WHEN `consumer` selects `sdk:out` while `sdk:lib` (or `ext_dep`) is missing, incomplete, or untrusted
- THEN `Builder::check_cache` rejects partial reuse and triggers full-output execution of `sdk`
- WHEN `sdk:out` and `sdk:lib` (and any transitive runtime references) are complete and trusted while `sdk:dev` is absent
- THEN `Builder::check_cache` reuses `sdk:out` and `sdk:lib` with zero producer builds

### Requirement: Fail-Closed Unsupported Lanes for Partial Cache Projection

Partial demand projection in `Builder::check_cache` MUST remain disabled (preserving existing all-output fail-closed semantics) for:
1. Content-addressed (`is_ca`) derivations
2. CA-resolved input-addressed (`resolved.identity.is_some()`) derivations
3. Fixed-output derivations (`ca_hash.is_some()`)
4. Mutable-history workspace derivations (`is_mutable_workspace`)
5. Bound-plan consumers (`!bound_plan_outputs.is_empty()` or `PLAN_OUTPUT_BINDINGS_ENV_KEY`)
6. Dynamic-plan producers and `.drv` output derivations (`!dynamic_plan_outputs.is_empty()`, `dynamic_admission_identity.is_some()`, or `.drv` output paths)
7. Strong shared action-result admission (`check_shared_action_result` MUST continue requesting all `derivation.outputs.keys()`)
8. Unqualified roots (`Worker::want` / `demanded_outputs = None`), which retain all-output requirements

#### Scenario: Unsupported lanes reject partial cache reuse

- GIVEN any derivation in the CA, CA-resolved IA, FOD, mutable-workspace, bound-plan, dynamic-plan/`.drv`, or signed `ActionResultRecord` lanes with only `"out"` cached and `"dev"` absent
- WHEN a consumer selects only `"out"`
- THEN partial `PathInfo` projection is NOT applied and the lane preserves its all-output miss or fail-closed behavior
