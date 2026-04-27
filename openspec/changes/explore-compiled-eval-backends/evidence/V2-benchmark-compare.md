Task-ID: V2
Covers: r[compiled-eval.profiling-gate], r[compiled-eval.benchmark-guardrail]
Command: cargo run --example benchmark_suite -- --bundle-out target/benchmarks/compiled-eval-closeout.json --repeat-count 2
Command: cargo run --example benchmark_compare -- --baseline openspec/changes/explore-compiled-eval-backends/evidence/compiled-eval-gate.json --fresh target/benchmarks/compiled-eval-closeout.json --absolute-threshold-ns 1000 --percent-threshold 5

Fresh closeout attempt result: BLOCKED
- pueue task 17 never reached the benchmark commands because `cargo test -p crunch-eval --lib` remained in compilation for 9m11s and was killed.
- Next best checked-in evidence retained from the stale worktree:
  - `evidence/benchmark-suite-baseline.txt` records baseline `eval-fetch-git` eval share 99.98% and `workflow-package-set-eval-build-graph` eval share 99.11%.
  - `evidence/post-prototype-benchmark-compare.txt` records non-eval guardrail drift (`build-graph-package-set.build_graph_wall_ns` +26.15%), so the Cranelift prototype remains feature-gated and non-shipping.
  - Archived `lazy-root-evaluation` design decision 5 tables further compiled-eval work until lazy metrics justify reopening it.
