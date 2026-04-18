# Parallel root benchmark summary

Command:

```sh
bash autoresearch.sh
```

Environment:
- isolated target dir: `/home/brittonr/git/crunch/crunch/target/autoresearch-parallel-root`
- bundle: `openspec/changes/parallel-root-evaluation/evidence/parallel-root-benchmark.json`
- repeat count: `10`

Observed medians from the machine-readable bundle:

- `root_discovery_wall_ns`: `72,637,150`
- `selected_root_total_wall_ns`: `280,872,794`
- `selected_root_force_wall_ns`: `70,565,643`
- `explicit_top_level_root_force_count`: `1`
- `explicit_nonselected_root_force_count`: `0`
- `all_roots_total_wall_ns`: `208,601,181`
- `parallel_all_roots_total_wall_ns`: `803,100,983`
- `parallel_root_eval_concurrency`: `4`

Notes:
- Evidence is intentionally honest: on this host, the first isolated-worker
  parallel implementation is slower than the serial all-root path.
- Selected-root guardrails remain visible in the same bundle.
- The bundle records the exact benchmark argv and workload metadata for review.
