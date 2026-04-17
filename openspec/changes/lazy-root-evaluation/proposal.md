# Lazy root evaluation

## Why

Crunch currently asks Nickel for an export-ready deep result before it extracts
root derivations. That is simple, but it pushes the eval path toward eager
whole-program forcing even when the caller only needs top-level root names or
one selected root.

If crunch wants Nix/snix-like laziness, the next step is not more compiled
backend work. The next step is to separate top-level root discovery from
selected-root forcing and to thread that lazy boundary through planning,
building, and performance measurement.

This change captures that lazy-evaluation direction explicitly, keeps the
recent Cranelift work tabled as an experiment rather than the main plan, and
defines the benchmark + autoresearch metrics needed to optimize the lazy path
honestly.

## What Changes

- add a lazy root-discovery and selected-root forcing model to `crunch-eval`
- update pipeline-facing requirements so build execution no longer requires a
  whole-program deep export before root selection
- define checked-in lazy-eval benchmark workloads and machine-readable metrics
  for autoresearch
- make single-selected-root latency the primary optimization target, while
  keeping full-root and force-count guardrails visible
- explicitly defer further compiled-eval work until the lazy path is built and
  benchmarked

## Capabilities

### New Capabilities

- `lazy-root-discovery`: crunch can inspect available top-level roots without
  eagerly deep-exporting every root value
- `selected-root-forcing`: crunch can force one selected derivation root on
  demand through `crunch-eval`
- `lazy-eval-benchmark-metrics`: the benchmark suite and future autoresearch
  loop have explicit, honest metrics for lazy root evaluation

## Impact

- **Files**: new change-local specs under `nickel-eval`, `pipeline`, and
  `performance`
- **Architecture**: shifts the next optimization focus from compiled backends
  to lazy evaluation boundaries
- **Benchmarks**: adds lazy root discovery and selected-root workloads on a
  checked-in multi-root fixture
- **Autoresearch**: plans a dedicated optimization loop around selected-root
  latency rather than whole-program export time

## Verification

A reviewer should expect this change to land with:

- lazy-session tests for single, array, and record top-level outputs
- equivalence tests showing selected-root forcing matches the current eager
  path on checked-in fixtures
- a checked-in lazy benchmark fixture plus a baseline benchmark bundle and
  `benchmark_compare` transcript
- an autoresearch plan whose primary metric is
  `selected_root_total_wall_ns` and whose secondary metrics include the lazy
  discovery/force counts and full-root guardrail timing

## Non-Goals

- ship a new compiled backend as the primary path for this change
- replace Nickel semantics with a crunch-specific parser
- promise incremental re-evaluation caches in this change
- claim that every build flow becomes single-root only; full-root builds still
  matter and remain guardrails
