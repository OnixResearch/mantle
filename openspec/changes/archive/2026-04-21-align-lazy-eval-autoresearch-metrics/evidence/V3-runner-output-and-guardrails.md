Evidence-ID: align-lazy-eval-autoresearch-metrics-v3-runner-output-and-guardrails
Task-ID: V3
Artifact-Type: verification-note
Covers: performance.autoresearch.parallelrootprimary, performance.autoresearch.parallelrootprimary.reporootsession
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-21

Fresh isolated baseline artifacts:
- stdout: `openspec/changes/align-lazy-eval-autoresearch-metrics/evidence/parallel-root-baseline.stdout.txt`
- stderr: `openspec/changes/align-lazy-eval-autoresearch-metrics/evidence/parallel-root-baseline.stderr.txt`
- bundle: `openspec/changes/align-lazy-eval-autoresearch-metrics/evidence/parallel-root-baseline.json`

Runner-output proof:
- stdout primary section prints only the parallel-root primary metric:
  - `=== Primary metric ===`
  - `parallel_all_roots_total_wall_ns = 106820627 ns`
  - `METRIC parallel_all_roots_total_wall_ns=106820627`
- The runner summary does not present `selected_root_total_wall_ns` as the primary emitted metric.

Bundle guardrail proof from `parallel-root-baseline.json`:
- `selected_root_total_wall_ns = 60086922`
- `root_discovery_wall_ns = 43871554`
- `selected_root_force_wall_ns = 15738483`
- `explicit_top_level_root_force_count = 1`
- `explicit_nonselected_root_force_count = 0`
- `all_roots_total_wall_ns = 61013110`
- parallel workload also records `parallel_root_eval_concurrency = 4`

This shows the repo-root runner emits `parallel_all_roots_total_wall_ns` as the primary metric while the generated bundle keeps the selected-root metrics as guardrails.
