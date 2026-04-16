# Benchmark suite

Checked-in benchmark entry points:

- Smoke path: `cargo run --example benchmark_eval_smoke -- --bundle-out target/benchmarks/eval-smoke.json --repeat-count 2`
- Full matrix: `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/suite.json --repeat-count 2`
- Compare baseline vs fresh: `cargo run --example benchmark_compare -- --baseline target/benchmarks/baseline.json --fresh target/benchmarks/suite.json --absolute-threshold-ns 1000 --percent-threshold 5`

Both commands write a machine-readable JSON bundle with schema
`crunch-benchmark-bundle-v1`.

## Initial workload matrix

| Workload | Kind | Fixture / source | Why it is representative |
|---|---|---|---|
| `eval-fetch-git` | evaluation | `examples/fetch-git.ncl` | Exercises Nickel evaluation and derivation extraction on a checked-in fetcher derivation without introducing build execution. |
| `convert-multi-output` | conversion | `examples/multi-output.ncl` | Exercises glue-layer conversion, store-path construction, and multi-output lowering on a checked-in derivation. |
| `substitution-plan-delta-suite` | substitution | `crates/crunch-delta/src/fixtures.rs::bench_suite()` | Exercises substitution planning against the fixed crunch-delta benchmark suite with no network or mutable store dependency. |
| `build-graph-package-set` | build-graph | `examples/package-set.ncl` | Exercises repeated conversion of a checked-in multi-root package set with shared inputs, which is a cheap proxy for build-graph preparation. |

## Notes

- The smoke path intentionally runs only the evaluation workload so ordinary local checks stay cheap.
- The full suite uses checked-in fixtures only and does not require network access.
- `total_wall_ns` records outer wall time for the workload loop.
- The suite currently records named phase metrics for `evaluation_wall_ns`, `conversion_wall_ns`, `substitution_planning_wall_ns`, and `build_graph_wall_ns`.
- Store lookup or persistence metrics are omitted today because these checked-in workloads do not expose that boundary honestly without switching to a different plan/build workload.
- If a workload can only report honest total wall time, the bundle keeps `total_wall_ns` and leaves `phase_metrics` empty rather than inventing a fake phase metric.
- The comparison entry point matches workloads by stable `workload_name`, compares the shared metric names in each workload, and reports the largest regression and largest win across the matched metrics.
- `--absolute-threshold-ns` highlights changes at or above a fixed nanosecond delta. `--percent-threshold` highlights changes at or above a percentage delta.
- Evaluation and substitution workloads record the default logical store prefix `/crunch/store`.
- Conversion and build-graph workloads record `/nix/store` because their checked-in seed inputs are absolute `/nix/store/...` paths; using `/crunch/store` there would make `crunch_glue::convert()` reject those seed inputs as invalid store paths.

## Local-first workflow

### Ordinary development

1. Run the smoke path while iterating on unrelated work:
   - `cargo run --example benchmark_eval_smoke -- --bundle-out target/benchmarks/eval-smoke.json --repeat-count 2`
2. Run the full suite before landing benchmark-sensitive changes:
   - `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/suite.json --repeat-count 2`
3. Keep the resulting JSON bundles under `target/benchmarks/` or copy them into a review artifact directory.

### Optimization-specific experiments

1. Capture a baseline bundle on the same machine and toolchain:
   - `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/baseline.json --repeat-count 2`
2. Apply the candidate change, then capture a fresh bundle:
   - `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/candidate.json --repeat-count 2`
3. Compare the fresh bundle against the saved baseline:
   - `cargo run --example benchmark_compare -- --baseline target/benchmarks/baseline.json --fresh target/benchmarks/candidate.json --absolute-threshold-ns 1000 --percent-threshold 5`
4. Treat small deltas as noise until the same workload and metric move in the same direction across repeated runs on the same host.
