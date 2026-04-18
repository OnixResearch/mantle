# Benchmark suite

README's benchmark section points here. These are the checked-in entry points
under `examples/`:

- Smoke path: `cargo run --example benchmark_eval_smoke -- --bundle-out target/benchmarks/eval-smoke.json --repeat-count 2`
- Full matrix: `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/suite.json --repeat-count 2`
- Compare baseline vs fresh: `cargo run --example benchmark_compare -- --baseline target/benchmarks/baseline.json --fresh target/benchmarks/suite.json --absolute-threshold-ns 1000 --percent-threshold 5`

The smoke and full-suite commands write a machine-readable JSON bundle with
schema `crunch-benchmark-bundle-v1`.

## Initial workload matrix

| Workload | Kind | Fixture / source | Why it is representative |
|---|---|---|---|
| `eval-fetch-git` | evaluation | `examples/fetch-git.ncl` | Exercises Nickel evaluation and derivation extraction on a checked-in fetcher derivation without introducing build execution. |
| `convert-multi-output` | conversion | `examples/multi-output.ncl` | Exercises glue-layer conversion, store-path construction, and multi-output lowering on a checked-in derivation. |
| `substitution-plan-delta-suite` | substitution | `crates/crunch-delta/src/fixtures.rs::bench_suite()` | Exercises substitution planning against the fixed crunch-delta benchmark suite with no network or mutable store dependency. |
| `build-graph-package-set` | build-graph | `examples/package-set.ncl` | Exercises repeated conversion of a checked-in multi-root package set with shared inputs, which is a cheap proxy for build-graph preparation. |
| `workflow-package-set-eval-build-graph` | workflow | `examples/package-set.ncl` | Exercises one honest multi-phase workflow result by timing package-set evaluation and the follow-up build-graph lowering in the same workload entry. |
| `store-persist-lookup-blob` | store | `examples/benchmark_support.rs::benchmark_store_aware_workload` | Exercises a fresh local temp store per sample, timing one signed output persistence phase and the follow-up reopened-store lookup phase on deterministic local bytes. |
| `lazy-root-discovery-wide-package-set` | lazy-eval | `tests/fixtures/wide_package_set.ncl` | Measures lazy root label discovery on a 16-root package set without deep-forcing any root values. |
| `lazy-selected-root-wide-package-set` | lazy-eval | `tests/fixtures/wide_package_set.ncl` | Measures end-to-end latency to obtain one selected root through the lazy session API. Primary autoresearch target. |
| `eager-all-roots-wide-package-set` | lazy-eval | `tests/fixtures/wide_package_set.ncl` | Guardrail: measures all-roots eager path on the same fixture to detect regressions from lazy changes. |

## Notes

- The smoke path intentionally runs only the evaluation workload so ordinary local checks stay cheap.
- The full suite uses checked-in fixtures only and does not require network access.
- `total_wall_ns` records outer wall time for the workload loop.
- The suite currently records named phase metrics for `evaluation_wall_ns`, `conversion_wall_ns`, `substitution_planning_wall_ns`, `build_graph_wall_ns`, `store_persistence_wall_ns`, and `store_lookup_wall_ns`.
- `workflow-package-set-eval-build-graph` is the checked-in multi-phase workflow entry: its `evaluation_wall_ns` and `build_graph_wall_ns` metrics come from two real boundaries crossed in the same workload run.
- `store-persist-lookup-blob` is the only workload that records store phase metrics. Other workloads omit those store metrics because they never cross that boundary honestly.
- The three `lazy-eval` workloads share a single wide package-set fixture (`tests/fixtures/wide_package_set.ncl`). They emit `root_discovery_wall_ns`, `selected_root_total_wall_ns`, `selected_root_force_wall_ns`, `explicit_top_level_root_force_count`, `explicit_nonselected_root_force_count`, and `all_roots_total_wall_ns`. The standalone lazy benchmark can also be run via `cargo run --example benchmark_lazy_eval -- --repeat-count 10`.
- If a workload can only report honest total wall time, the bundle keeps `total_wall_ns` and leaves `phase_metrics` empty rather than inventing a fake phase metric.
- The comparison entry point matches workloads by stable `workload_name`, compares only the shared metric names in each workload, reports per-workload missing metrics explicitly, and reports the largest regression and largest win across the matched metrics.
- `benchmark_compare` also accepts `--json` when you want machine-readable comparison output.
- `--absolute-threshold-ns` highlights changes at or above a fixed nanosecond delta. `--percent-threshold` highlights changes at or above a percentage delta.
- All three entry points support `-h` / `--help` for their exact flag list.
- Evaluation and substitution workloads record the default logical store prefix `/crunch/store`.
- Conversion and build-graph workloads record `/nix/store` because their checked-in seed inputs are absolute `/nix/store/...` paths; using `/crunch/store` there would make `crunch_glue::convert()` reject those seed inputs as invalid store paths.

## Local-first workflow

### Ordinary development

1. Run the smoke path while iterating on unrelated work:
   - `cargo run --example benchmark_eval_smoke -- --bundle-out target/benchmarks/eval-smoke.json --repeat-count 2`
2. Run the full suite before landing benchmark-sensitive changes:
   - `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/suite.json --repeat-count 2`
3. Inspect `workflow-package-set-eval-build-graph` when you need one workload entry that splits evaluation cost from build-graph preparation cost.
4. Keep the resulting JSON bundles under `target/benchmarks/` or copy them into a review artifact directory.

### Optimization-specific experiments

1. Capture a baseline bundle on the same machine and toolchain:
   - `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/baseline.json --repeat-count 2`
2. Apply the candidate change, then capture a fresh bundle:
   - `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/candidate.json --repeat-count 2`
3. Compare the fresh bundle against the saved baseline:
   - `cargo run --example benchmark_compare -- --baseline target/benchmarks/baseline.json --fresh target/benchmarks/candidate.json --absolute-threshold-ns 1000 --percent-threshold 5`
   - add `--json` when another tool should consume the comparison result
4. Read `missing_from_fresh` and `missing_from_baseline` as honest omission, not zero. Sparse metrics stay sparse on purpose.
5. When a change is likely store-related, inspect `store-persist-lookup-blob` first. That is the only checked-in workload that should carry `store_persistence_wall_ns` or `store_lookup_wall_ns`.
6. Treat small deltas as noise until the same workload and metric move in the same direction across repeated runs on the same host.
