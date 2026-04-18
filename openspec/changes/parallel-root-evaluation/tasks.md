# Tasks: Parallel root evaluation

## Phase 1: Spec and API boundary

- [x] Add delta specs for `nickel-eval`, `pipeline`, and `performance`
      covering bounded multi-root forcing, streaming pipeline integration, and
      benchmark evidence
- [ ] Define the `crunch-eval` batch-forcing API shape and root-labeled error
      contract in `crates/crunch-eval`
- [ ] Keep selected-root lazy APIs and semantics intact while adding the new
      multi-root path

## Phase 2: `crunch-eval` implementation and proof

- [ ] Implement bounded multi-root forcing in `crates/crunch-eval/src/session.rs`
      without requiring callers to deep-export all roots first
- [ ] Confirm the relevant Nickel evaluator state is `Send` or `!Send` with a
      compile-time proof or narrowly targeted test, so the implementation does
      not guess about cross-thread safety
- [ ] Choose and implement the isolation strategy implied by that finding
      (for example per-worker `EvaluationSession` / source-string worker state),
      and do not share a mutable Nickel `Context` across threads without proof
- [ ] Add unit and equivalence tests for single, array, and record outputs that
      compare serial vs batch-forced derivations by label
- [ ] Add failure tests showing missing labels or per-root forcing failures keep
      the failed label in the surfaced error

## Phase 3: Pipeline streaming integration

- [ ] Replace the serial `session.force_all_roots::<CrunchDerivation>()`
      prefix in `crates/crunch-pipeline/src/lib.rs` with bounded incremental
      forcing and conversion
- [ ] Stream `EvalMessage`s to the worker as roots are converted instead of
      waiting for a full evaluated root vector, while keeping
      `crunch_glue::convert()` / `ConversionCache` / `DerivationRegistry`
      mutation on one serialized path rather than making conversion itself
      concurrent
- [ ] Reuse `max_jobs` as the first public upper bound for eval parallelism and
      document any internal safety cap or derivation strategy
- [ ] Verify all new conversion call sites keep using the configured store
      prefix instead of hardcoded `/nix/store`
- [ ] Add integration coverage that asserts correct label -> output association
      under concurrent root forcing and conversion

## Phase 4: Benchmark and validation evidence

- [ ] Add checked-in wide-fixture benchmark coverage for serial vs parallel
      all-root evaluation throughput
- [ ] Record machine-readable evidence that includes serial and parallel
      all-root metrics plus the concurrency value used for the run
- [ ] Re-run selected-root lazy benchmarks and compare `selected_root_total_wall_ns`
      guardrails before/after the parallel path on an otherwise idle host or
      isolated target directory so concurrent Cargo work does not corrupt the
      measurements
- [ ] Validate the change with `openspec validate parallel-root-evaluation`
- [ ] Run proposal, design, and tasks gates for `parallel-root-evaluation`
