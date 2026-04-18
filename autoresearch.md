# Autoresearch: Lazy root evaluation

## Goal

Optimize `selected_root_total_wall_ns` — the end-to-end latency from opening
a lazy evaluation session to obtaining one fully materialized root from a
checked-in wide package-set fixture.

## Primary metric

- **`selected_root_total_wall_ns`** (lower is better)
  - Defined at the `crunch-eval` session boundary: includes session open
    (shallow eval + root discovery) plus per-root deep forcing.

## Secondary metrics (guardrails)

- `root_discovery_wall_ns` — time for shallow eval + label enumeration
- `selected_root_force_wall_ns` — time for deep-forcing one root after discovery
- `explicit_top_level_root_force_count` — must be exactly 1
- `explicit_nonselected_root_force_count` — must be 0
- `all_roots_total_wall_ns` — guardrail: full-root path must not regress

## Workload

Fixed 16-root package set: `tests/fixtures/wide_package_set.ncl`.

Selected root: `alpha`.

## Sampling strategy

Each experiment run executes the benchmark with `--repeat-count 10` and
reports the median of the 10 samples. The median avoids outlier distortion
from cold caches or scheduling jitter.

## Entry point

```sh
./autoresearch.sh
```

Runs the lazy benchmark, captures results, and prints the primary metric.

## Baseline

Capture before starting experiments:

```sh
cargo run --example benchmark_lazy_eval -- \
  --repeat-count 10 \
  --bundle-out target/benchmarks/lazy-eval-baseline.json
```

## Comparison

```sh
cargo run --example benchmark_compare -- \
  --baseline target/benchmarks/lazy-eval-baseline.json \
  --fresh target/benchmarks/lazy-eval-candidate.json \
  --absolute-threshold-ns 1000000 \
  --percent-threshold 5
```
