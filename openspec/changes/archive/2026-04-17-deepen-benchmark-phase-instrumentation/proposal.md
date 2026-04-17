# Deepen benchmark phase instrumentation

## Why

The checked-in benchmark suite now gives crunch a stable workload matrix,
machine-readable result bundles, and baseline comparison tooling. That is enough
to spot which workload regressed, but it still leaves a high-value blind spot:
a realistic workflow benchmark cannot yet say whether time moved in evaluation,
conversion, store lookup, substitution, build execution, or store persistence
within the same run.

That gap matters for the next round of optimization work. Compiled-evaluation
experiments, cache/store work, and scheduler/build-pipeline work all need a
sharper answer than "the suite got slower." Maintainers need at least one
multi-phase benchmark path that localizes regressions inside a workflow, plus
clear rules for when a phase metric is omitted because the boundary is not
observable.

## What Changes

- add deeper benchmark requirements for multi-phase workflow instrumentation
- require explicit omission of unobservable phases instead of fabricated zeros
- require store lookup or persistence timing only when the workload truly
  exposes those boundaries
- require baseline comparison to handle sparse phase metrics without inventing
  missing data
- document the local-first operator workflow for collecting and comparing
  multi-phase results

## Capabilities

### New Capabilities
- `multi-phase-workflow-benchmarks`: record more than one named phase metric in
  a single checked-in workflow benchmark
- `sparse-phase-omission`: omit phase metrics that are not honestly observable
  and keep that omission stable in result bundles and comparisons
- `store-phase-benchmarking`: record store lookup or persistence timing when a
  benchmark path genuinely exposes those boundaries

### Modified Capabilities
- `baseline-comparison`: treat missing phase metrics as missing metrics rather
  than invented zero-valued data

## Impact

- **Files**: benchmark helper code, benchmark docs, result-bundle comparison
  logic, and tests for sparse metric handling
- **APIs**: internal benchmark helper structures for phase metrics and compare
  output may grow richer omission/reporting semantics
- **Dependencies**: no new runtime dependency surface; any benchmark-only
  helpers remain in dev-only paths
- **Testing**: add deterministic tests for omitted metrics, sparse comparisons,
  and at least one checked-in workflow that emits multiple real phase metrics

## Non-Goals

- a hard performance gate in CI
- benchmarking every internal helper in isolation
- fabricating store metrics for workloads that cannot expose them honestly
- changing build or store behavior only to make benchmarking easier
