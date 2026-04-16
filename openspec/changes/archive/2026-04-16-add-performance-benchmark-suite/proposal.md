# Add performance benchmark suite

## Why

crunch has several promising optimization ideas, but the repo still lacks a
stable benchmark suite that separates evaluation cost from conversion, store,
substitution, and build execution cost. That leaves roadmap decisions driven by
intuition instead of evidence.

The gap is visible already: compiled evaluation work is explicitly supposed to
be profiling-gated, yet the repo does not ship the benchmark harness needed to
make that decision confidently.

## What Changes

- add a checked-in benchmark harness with representative crunch workloads
- emit machine-readable benchmark result bundles with environment metadata
- track phase-separated metrics for evaluation, conversion, store,
  substitution, and build work so optimization effort can target the real
  bottleneck
- add baseline-comparison tooling and docs for local-first performance work
- add at least one CI-compatible local smoke benchmark invocation

## Capabilities

### New Capabilities
- `performance-benchmark-harness`: run representative crunch workloads from a
  supported local benchmark entry point
- `phase-separated-metrics`: record evaluation, conversion, substitution, and
  build-related timing separately
- `baseline-comparison`: compare a new run with a saved baseline and report the
  largest regressions or wins

## Impact

- **Files**: benchmark harness scripts or benches, representative fixtures,
  docs for performance workflows, optional JSON schema for result bundles
- **APIs**: internal harness helpers for stable workload execution and result
  capture
- **Dependencies**: benchmark-only crates are allowed if they stay out of the
  runtime path
- **Testing**: result-schema tests, fixture determinism tests, and at least one
  smoke benchmark invocation in CI-compatible local checks

## Non-Goals

- a hard CI performance gate in this first change
- benchmarking every internal helper or micro-optimization target
- changing shipped runtime behavior
- choosing compiled evaluation backends in this change
