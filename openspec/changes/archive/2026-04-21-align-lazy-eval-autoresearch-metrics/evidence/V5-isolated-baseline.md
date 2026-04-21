Evidence-ID: align-lazy-eval-autoresearch-metrics-v5-isolated-baseline
Task-ID: V5
Artifact-Type: verification-note
Covers: performance.autoresearch.metricsegmentation, performance.autoresearch.metricsegmentation.parallelafterselected, performance.autoresearch.metricsegmentation.workflowprocedure
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-21

Fresh isolated baseline run evidence:
- stdout prints:
  - `BASELINE_BUNDLE_OUT=openspec/changes/archive/2026-04-21-align-lazy-eval-autoresearch-metrics/evidence/parallel-root-baseline.json`
  - `BASELINE_CARGO_TARGET_DIR=target/autoresearch-parallel-root-baseline/run-ttWuLH`
- stderr confirms the same run-specific target dir was used for the benchmark binary:
  - `Running 'target/autoresearch-parallel-root-baseline/run-ttWuLH/debug/examples/benchmark_lazy_eval ...'`

Creation/existence checks after the run:
- bundle exists = `1`
- target dir exists = `1`
- target dir resolved path = `/home/brittonr/git/crunch/crunch/target/autoresearch-parallel-root-baseline/run-ttWuLH`
- repo default target resolved path = `/home/brittonr/git/crunch/crunch/target`
- shared Cargo target resolved path = `/home/brittonr/.cargo-target`
- `TARGET_IS_REPO_TARGET = 0`
- `TARGET_IS_SHARED_TARGET = 0`

The run command pre-removed the bundle/stdout/stderr artifacts before launch, and the script then created a fresh run-specific non-default target dir plus a new bundle path for this baseline.
