# Tasks: Deepen benchmark phase instrumentation

## Phase 1: Multi-phase workflow instrumentation

- [x] Add at least one checked-in workflow benchmark that emits more than one
      named phase metric in a single workload result.
- [x] Keep that workflow benchmark on checked-in fixtures or deterministic local
      state only, with no network dependency.
- [x] Record and test that the workflow benchmark's phase metric names map to
      real workflow boundaries it actually crosses.
- [x] Add or identify a store-aware benchmark path that records store lookup or
      store persistence timing only when the workload genuinely exposes that
      boundary.
- [x] Ensure workloads that do not cross store lookup or persistence boundaries
      omit those metrics.

## Phase 2: Sparse comparison semantics and docs

- [x] Make the comparison path report per-workload missing metrics explicitly
      when a metric is present in only one bundle.
- [x] Ensure comparison logic never fabricates zero-valued metrics for omitted
      phases.
- [x] Document the local-first operator workflow for multi-phase baselines,
      candidate runs, sparse metric interpretation, and store-aware benchmark
      runs.

## Phase 3: Validation coverage

- [x] Add tests that the multi-phase workflow benchmark emits more than one
      named phase metric in a single result entry.
- [x] Add tests that a workload without an observable phase boundary keeps
      `total_wall_ns` and leaves `phase_metrics` empty.
- [x] Add tests that sparse bundle comparison reports missing metrics instead of
      synthetic zero-valued matches.
- [x] Add tests that store lookup or persistence metrics appear only on the
      store-aware workload and are absent from unrelated workloads.

## Validation

- [x] Run the benchmark harness tests, sparse-comparison tests, and any new
      store-aware benchmark tests with the repo's documented build environment
      and keep the `test result:` lines.
- [x] Run `openspec validate deepen-benchmark-phase-instrumentation`.
