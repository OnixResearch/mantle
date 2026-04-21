# Align lazy-eval autoresearch metrics with the active workstream

## Why

The main performance spec still says lazy-eval autoresearch optimizes
`selected_root_total_wall_ns` first. That matched the archived
`lazy-root-evaluation` change, but the repo-root autoresearch harness has since
been retargeted to parallel-root throughput:

- `autoresearch.md` now names `parallel_all_roots_total_wall_ns` as the goal
- `autoresearch.sh` prints `METRIC parallel_all_roots_total_wall_ns=...`
- `autoresearch.jsonl` still begins with the old selected-root config header

That leaves one active harness, two different primary metrics, and one mixed
history file. Before implementation touches the live autoresearch files, the
spec needs to say which lazy-eval sub-workstream the repo-root harness now
represents, how metric switches are recorded, and whether the existing
`autoresearch.jsonl` file is segmented in place or replaced.

## What Changes

- clarify that lazy-eval autoresearch now has two distinct optimization modes:
  selected-root latency sessions and parallel-root throughput sessions
- keep `selected_root_total_wall_ns` as the primary metric for selected-root
  sessions only
- require the repo-root parallel-root session to use
  `parallel_all_roots_total_wall_ns` as its primary metric while preserving the
  selected-root metrics as guardrails
- require a new `autoresearch.jsonl` config segment within the existing file
  and a fresh isolated baseline whenever the lazy-eval primary metric changes,
  where isolation means a new bundle path plus an isolated Cargo target
  directory not shared with unrelated Cargo work during measurement

## Capabilities

### New Capabilities

- `parallel-root-autoresearch-target`: the repo can declare a parallel-root
  throughput session with an explicit primary metric and guardrails
- `autoresearch-metric-segmentation`: lazy-eval experiment history can keep
  multiple primary metrics in one file without mixing them under one config
  header

### Modified Capabilities

- `lazy-eval-autoresearch-targeting`: the main performance spec now separates
  selected-root tuning from parallel-root throughput tuning instead of treating
  all lazy-eval sessions as one metric family

## Impact

- **Files**:
  - `openspec/specs/performance/spec.md`
  - `autoresearch.md`
  - `autoresearch.sh`
  - `autoresearch.jsonl`
- **APIs**: none
- **Dependencies**: none
- **Testing**:
  - `openspec validate align-lazy-eval-autoresearch-metrics`
  - proposal/design/tasks gates for this change
  - one fresh isolated `./autoresearch.sh` baseline after implementation to
    prove the emitted primary metric is `parallel_all_roots_total_wall_ns`
  - bundle inspection from that same run proving the selected-root guardrail
    metrics remain present
  - `autoresearch.jsonl` inspection proving a new `{"type":"config",
    "metricName":"parallel_all_roots_total_wall_ns"}` segment exists while
    the older selected-root segment remains present in the same file, and that
    subsequent run records under the new segment use the new primary metric as
    their top-level `metric` value
  - runner-visible evidence of `BASELINE_BUNDLE_OUT=<path>` and
    `BASELINE_CARGO_TARGET_DIR=<path>` for that segment, while the
    no-concurrent-Cargo condition stays a documented operator procedure in
    `autoresearch.md`

## Non-Goals

- change lazy-eval benchmark semantics or workload names
- drop selected-root guardrail metrics from the parallel-root workflow
- implement the file updates in this change proposal itself
