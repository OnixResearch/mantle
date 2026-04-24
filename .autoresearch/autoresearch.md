# Autoresearch: Parallel root evaluation

## Goal

Optimize `parallel_all_roots_total_wall_ns` — the end-to-end latency from
opening a lazy evaluation session to materializing all roots from the checked-in
wide package-set fixture through the bounded parallel path.

## Active repo-root session

This checked-in repo-root workflow tracks the active lazy-eval parallel-root
throughput session.

Selected-root latency sessions still optimize
`selected_root_total_wall_ns`, but that mode is a separate lazy-eval session
family rather than the repo-root primary target for this harness.

## Primary metric

- **`parallel_all_roots_total_wall_ns`** (lower is better)
  - Defined at the `crunch-eval` API boundary: includes session open plus
    bounded multi-root forcing for the full root set.

## Secondary metrics (guardrails)

- `all_roots_total_wall_ns` — serial all-root baseline on the same fixture
- `parallel_root_eval_concurrency` — recorded concurrency used for the run
- `selected_root_total_wall_ns` — selected-root guardrail
- `root_discovery_wall_ns` — discovery guardrail
- `selected_root_force_wall_ns` — selected-root force guardrail
- `explicit_top_level_root_force_count` — must stay exactly 1 for selected-root
- `explicit_nonselected_root_force_count` — must stay 0 for selected-root

## Workload

Fixed 16-root package set: `tests/fixtures/wide_package_set.ncl`.

Parallel workload: `parallel-all-roots-wide-package-set`.
Selected-root guardrail label: `alpha`.

## Sampling strategy

Each experiment run executes the benchmark with `--repeat-count 10` and reports
the median of the 10 samples.

## Isolation rule

Run benchmarks in an isolated Cargo target directory. Do not share that target
with unrelated Cargo build/test work during the measurement window.

`./autoresearch.sh` prints `BASELINE_BUNDLE_OUT=<path>` and
`BASELINE_CARGO_TARGET_DIR=<path>` before the benchmark starts. The script uses
a fresh run-specific non-default Cargo target directory for the measurement
window so the printed target path stays distinct from the repo default
`target/` tree and shared `~/.cargo-target`.

## Entry point

```sh
./autoresearch.sh
```

Runs the lazy benchmark bundle, captures results, and prints the primary metric.

## Metric-switch rollover

When the lazy-eval primary metric changes, keep the same `autoresearch.jsonl`
file but append a new config segment before the first run under the new metric.

Required operator steps:
1. append a new `{"type":"config","metricName":"<primary-metric>",...}`
   record to `autoresearch.jsonl`
2. keep older selected-root or prior lazy-eval segments in the same file
3. capture one fresh isolated baseline with `./autoresearch.sh` before logging
   later runs under the new segment
4. keep the measurement window free of unrelated Cargo build or test work

## Baseline

Capture before starting experiments:

```sh
BUNDLE_OUT=target/benchmarks/parallel-root-baseline.json ./autoresearch.sh
```

Review the printed `BASELINE_BUNDLE_OUT=` and `BASELINE_CARGO_TARGET_DIR=`
lines from that same run before comparing later experiments.

## Comparison

```sh
cargo run --example benchmark_compare -- \
  --baseline target/benchmarks/parallel-root-baseline.json \
  --fresh target/benchmarks/parallel-root-candidate.json \
  --absolute-threshold-ns 1000000 \
  --percent-threshold 5
```
