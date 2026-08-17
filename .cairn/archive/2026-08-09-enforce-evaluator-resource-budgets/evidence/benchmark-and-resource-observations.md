# Benchmark and resource observations

## Commands

```text
nix develop -c cargo run -q --example benchmark_eval_smoke -- --bundle-out /tmp/mantle-eval-budget-compatible-baseline.json --repeat-count 2
nix develop -c cargo run -q --example benchmark_eval_smoke -- --bundle-out /tmp/mantle-eval-budget-compatible-candidate.json --repeat-count 2
nix develop -c cargo run -q --example benchmark_compare -- --baseline /tmp/mantle-eval-budget-compatible-baseline.json --fresh /tmp/mantle-eval-budget-compatible-candidate.json --absolute-threshold-ns 50000000 --percent-threshold 25 --metric-absolute-threshold peak_rss_bytes=1048576 --metric-percent-threshold peak_rss_bytes=5 --json
nix develop -c cargo run -q --example benchmark_suite -- --bundle-out /tmp/mantle-eval-budget-full-suite.json --repeat-count 2
nix develop -c cargo run -q --bin mantle -- eval fixtures/evaluation-budget/positive-evaluation.ncl --budget-policy config/evaluation/default-policy.json --budget-report /tmp/mantle-eval-budget-strict-report.json
```

All commands completed successfully.

## Compatible comparison

The baseline recorded `evaluation_wall_ns=181003355` and `total_wall_ns=181012011`.

The candidate recorded `evaluation_wall_ns=209158604` and `total_wall_ns=209168924`.

The comparison recorded:

- `resource_cohort_compatible=true`
- no cohort mismatches
- `delta_percent=15.55509785992641` for evaluation wall time
- `delta_percent=15.555273290676826` for total wall time
- no accepted threshold exceedance at 50,000,000 ns and 25 percent
- `cpu_time_ms` and `peak_rss_bytes` in `unavailable_in_both`

The in-process benchmark does not estimate operation-scoped CPU time or peak RSS.

## Full suite

The full suite completed all ten workloads. Representative current metrics include:

- `eval-fetch-git.evaluation_wall_ns=157100794`
- `workflow-package-set-eval-build-graph.evaluation_wall_ns=289350544`
- `lazy-selected-root-wide-package-set.selected_root_total_wall_ns=205097357`
- `lazy-selected-root-wide-package-set.explicit_top_level_root_force_count=2`
- `lazy-selected-root-wide-package-set.explicit_nonselected_root_force_count=0`
- `eager-all-roots-wide-package-set.all_roots_total_wall_ns=198605510`
- `parallel-all-roots-wide-package-set.parallel_all_roots_total_wall_ns=219028196`

## Strict worker observation

The strict report recorded terminal success with:

- parent wall deadline observation: 50 ms
- worker CPU time: 45 ms through `getrusage-self`
- worker peak RSS: 52,920,320 bytes through `getrusage-self-maxrss`
- address-space enforcement: 8,589,934,592 bytes through `rlimit-as-not-rss`
- import read confinement through `landlock-read-file-and-read-dir`
- 33 admitted import entries as an upper bound, not an imported-module count

The report keeps actual imported modules, whole-value root discovery, and nonselected Nickel evaluation unavailable.
