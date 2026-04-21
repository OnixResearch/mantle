Evidence-ID: align-lazy-eval-autoresearch-metrics-v2-session-labeling
Task-ID: V2
Artifact-Type: verification-note
Covers: performance.autoresearch.parallelrootprimary, performance.autoresearch.parallelrootprimary.reporootsession, performance.autoresearch.metricsegmentation, performance.autoresearch.metricsegmentation.parallelafterselected, performance.autoresearch.metricsegmentation.workflowprocedure
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-21

`autoresearch.md` now documents:
- active repo-root session = parallel-root throughput
- primary metric = `parallel_all_roots_total_wall_ns`
- operator procedure: do not share the measurement window with unrelated Cargo build/test work
- metric-switch rollover: append a new config segment before first run under a new lazy-eval primary metric
- baseline review inputs: `BASELINE_BUNDLE_OUT=<path>` and `BASELINE_CARGO_TARGET_DIR=<path>`

`autoresearch.sh` now documents and implements the same session labeling by:
- naming the active repo-root session in the file header comment
- emitting only the parallel-root primary metric in the runner summary
- printing `BASELINE_BUNDLE_OUT=...` and `BASELINE_CARGO_TARGET_DIR=...` before the benchmark starts
- creating a fresh run-specific Cargo target dir under `CARGO_TARGET_ROOT`
