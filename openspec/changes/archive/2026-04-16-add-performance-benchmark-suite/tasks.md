# Tasks: Add performance benchmark suite

## Phase 1: Benchmark harness foundation

- [x] Add a checked-in benchmark entry point and result schema for crunch
      workload runs.
- [x] Create the initial workload matrix covering at least evaluation,
      conversion, substitution or fetcher behavior, and build-graph execution,
      and document why each chosen workload is representative of a real crunch
      layer or workflow.
- [x] Record environment metadata in every result bundle, including commit,
      toolchain, workload name, command or harness entry point, cache mode,
      total wall-clock metric, logical store prefix, relevant store settings,
      hermeticity mode when the workload uses one, host metadata, and repeat
      counts.
- [x] Keep any new benchmark-only dependencies out of the shipped runtime path
      and add a check that guards that boundary.

## Phase 2: Phase-separated metrics and comparison

- [x] Instrument the harness to emit named phase metrics where crunch has real
      architectural boundaries instead of a single total only, explicitly
      covering evaluation, conversion, store lookup or persistence where
      observable, substitution or fetch planning, and build-related execution.
- [x] Add baseline-comparison tooling that matches workloads by stable
      workload name, reports metric deltas for matching phase names, reports the
      largest regressions and wins, and supports configurable percentage or
      absolute comparison thresholds.
- [x] Document a local-first benchmark workflow for ordinary development and
      for optimization-specific experiments.

## Phase 3: Validation coverage

- [x] Add tests that benchmark result bundles always include the required
      metadata and metric fields for the workloads that claim them.
- [x] Add fixture checks that repeated benchmark setup runs resolve the same
      workload inputs and commands before timing starts.
- [x] Add validation coverage for workloads that cannot expose a phase boundary
      and verify the harness omits that metric instead of fabricating one.
- [x] Add deterministic comparison tests that feed fixed baseline and fresh
      bundles into the comparison entry point and verify workload matching,
      phase-metric delta reporting, largest-regression highlighting, and
      configurable-threshold behavior.
- [x] Add at least one CI-compatible local smoke benchmark invocation that uses
      checked-in fixtures only, requires no network access, and can run in
      ordinary local checks without requiring the full benchmark matrix.
- [x] Run the benchmark harness on the initial workload matrix once and keep the
      invocation used plus the produced result-bundle path as reviewable
      evidence for the checked-in workflow.

## Validation

- [x] Run the benchmark schema tests, fixture-determinism checks,
      deterministic comparison tests, harness smoke tests, and runtime-
      dependency boundary guard with the repo's documented build environment and
      keep the `test result:` lines.
- [x] Run `openspec validate add-performance-benchmark-suite`.
