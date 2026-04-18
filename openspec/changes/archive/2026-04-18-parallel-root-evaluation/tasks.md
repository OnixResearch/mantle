# Tasks: Parallel root evaluation

## Phase 1: Spec and API boundary

- [x] Add delta specs for `nickel-eval`, `pipeline`, and `performance`
      covering bounded multi-root forcing, streaming pipeline integration, and
      benchmark evidence
- [x] Define the `crunch-eval` batch-forcing API shape and root-labeled error
      contract in `crates/crunch-eval`
  - Evidence: `crates/crunch-eval/src/session.rs` now exposes
    `IsolatedWorkerInput`, `force_root_isolated`,
    `force_selected_roots{,_bounded}`, and `force_all_roots_bounded`.
    Missing-root batch failures now keep the requested label in the surfaced
    error string.
- [x] Keep selected-root lazy APIs and semantics intact while adding the new
      multi-root path
  - Evidence: existing `force_root` / `force_all_roots` remain, and
    `session::tests::force_root_isolated_matches_same_session_force_root`,
    `tests::session_force_root_matches_eager_path`, and
    `tests::session_force_all_roots_matches_eager_path` still pass.

## Phase 2: `crunch-eval` implementation and proof

- [x] Implement bounded multi-root forcing in `crates/crunch-eval/src/session.rs`
      without requiring callers to deep-export all roots first
  - Evidence: `force_selected_roots_with_workers(...)` opens isolated worker
    sessions from shared immutable source/import input and bounds each batch via
    `normalize_concurrency_cap(...)`.
- [x] Confirm the relevant Nickel evaluator state is `Send` or `!Send` with a
      compile-time proof or narrowly targeted test, so the implementation does
      not guess about cross-thread safety
  - Evidence: `session::tests::isolated_worker_input_is_send` proves the only
    cross-thread shared worker state is `Send`, and the worker implementation
    stringifies per-thread Nickel errors before crossing thread boundaries,
    avoiding any guess that raw Nickel error/context state is `Send`.
- [x] Choose and implement the isolation strategy implied by that finding
      (for example per-worker `EvaluationSession` / source-string worker state),
      and do not share a mutable Nickel `Context` across threads without proof
  - Evidence: `IsolatedWorkerInput::force_root(...)` opens a fresh
    `EvaluationSession` per worker from the same source text, import paths, and
    source name; no mutable `Context` is shared across threads.
- [x] Add unit and equivalence tests for single, array, and record outputs that
      compare serial vs batch-forced derivations by label
  - Evidence: `session::tests::force_all_roots_bounded_matches_serial_path`,
    `session::tests::force_root_isolated_matches_same_session_force_root`, and
    existing eager-vs-session tests in `crates/crunch-eval/src/lib.rs` cover
    record, array, and selected-root equivalence paths.
- [x] Add failure tests showing missing labels or per-root forcing failures keep
      the failed label in the surfaced error
  - Evidence: `session::tests::force_selected_roots_bounded_reports_failed_label`.

## Phase 3: Pipeline streaming integration

- [x] Replace the serial `session.force_all_roots::<CrunchDerivation>()`
      prefix in `crates/crunch-pipeline/src/lib.rs` with bounded incremental
      forcing and conversion
  - Evidence: `build_linux(...)` now runs `stream_roots_into_worker(...)`
    alongside `Worker::run_streaming(...)` instead of precomputing one full root
    vector before scheduling starts.
- [x] Stream `EvalMessage`s to the worker as roots are converted instead of
      waiting for a full evaluated root vector, while keeping
      `crunch_glue::convert()` / `ConversionCache` / `DerivationRegistry`
      mutation on one serialized path rather than making conversion itself
      concurrent
  - Evidence: `stream_roots_into_worker(...)` serializes conversion through one
    `ConversionCache`, emits `EvalMessage`s as each root finishes, and leaves
    `Worker::run_streaming(...)` / `DerivationRegistry` mutation on the existing
    single receiver path.
- [x] Reuse `max_jobs` as the first public upper bound for eval parallelism and
      document any internal safety cap or derivation strategy
  - Evidence: `resolve_eval_parallelism(max_jobs, requested_root_count)` clamps
    eval parallelism to `min(max_jobs, requested_root_count)` with no extra
    hidden cap.
- [x] Verify all new conversion call sites keep using the configured store
      prefix instead of hardcoded `/nix/store`
  - Evidence: `stream_roots_into_worker(...)` and `convert_root_drv_paths(...)`
    both construct `ConversionCache::new(&config.store_dir)` /
    `ConversionCache::new(store_dir)`.
- [x] Add integration coverage that asserts correct label -> output association
      under concurrent root forcing and conversion
  - Evidence: `pipeline_preserves_label_to_output_association_under_parallel_root_streaming`
    and `pipeline_reports_labeled_eval_failure_after_prior_root_dispatch` in
    `crates/crunch-pipeline/tests/integration_build.rs`.

## Phase 4: Benchmark and validation evidence

- [x] Add checked-in wide-fixture benchmark coverage for serial vs parallel
      all-root evaluation throughput
  - Evidence: `examples/benchmark_lazy_eval.rs`, `examples/benchmark_support.rs`,
    `docs/benchmark-suite.md`, and `tests/benchmark_harness.rs` now include
    `parallel-all-roots-wide-package-set` alongside the serial lazy-eval
    workloads.
- [x] Record machine-readable evidence that includes serial and parallel
      all-root metrics plus the concurrency value used for the run
  - Evidence: `openspec/changes/parallel-root-evaluation/evidence/parallel-root-benchmark.json`
    and `.../parallel-root-benchmark-summary.md` record
    `all_roots_total_wall_ns`, `parallel_all_roots_total_wall_ns`, and
    `parallel_root_eval_concurrency=4`.
- [x] Re-run selected-root lazy benchmarks and compare `selected_root_total_wall_ns`
      guardrails before/after the parallel path on an otherwise idle host or
      isolated target directory so concurrent Cargo work does not corrupt the
      measurements
  - Evidence: `bash autoresearch.sh` writes the bundle above using isolated
    target dir `/home/brittonr/git/crunch/crunch/target/autoresearch-parallel-root`
    and keeps `selected_root_total_wall_ns`, `root_discovery_wall_ns`,
    `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`, and
    `explicit_nonselected_root_force_count` in the same machine-readable run.
- [x] Validate the change with `openspec validate parallel-root-evaluation`
  - Evidence: `openspec validate parallel-root-evaluation` returned
    `Change 'parallel-root-evaluation' is valid`.
- [x] Run proposal, design, and tasks gates for `parallel-root-evaluation`
  - Evidence:
    - `openspec_gate stage=proposal change=parallel-root-evaluation` ->
      `VERDICT: PASS`
    - `openspec_gate stage=design change=parallel-root-evaluation` ->
      `Verdict: Ready to advance to tasks. No blockers, two minor
      clarifications worth adding inline.`
    - `openspec_gate stage=tasks change=parallel-root-evaluation` ->
      `Verdict: Ready to advance to tasks, no blockers.`
