Evidence-ID: align-lazy-eval-autoresearch-metrics-v1-openspec-validate
Task-ID: V1
Artifact-Type: verification-note
Covers: performance.autoresearch.selectedrootprimary, performance.autoresearch.selectedrootprimary.sessionconfig, performance.autoresearch.parallelrootprimary, performance.autoresearch.parallelrootprimary.reporootsession, performance.autoresearch.metricsegmentation, performance.autoresearch.metricsegmentation.parallelafterselected, performance.autoresearch.metricsegmentation.activereporootsegment, performance.autoresearch.metricsegmentation.workflowprocedure
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-21

Validation command:
- `openspec validate align-lazy-eval-autoresearch-metrics`
- Output: `Change 'align-lazy-eval-autoresearch-metrics' is valid`

Main-spec inspection confirmed the rebuilt `openspec/specs/performance/spec.md` now contains:
- `### Requirement: Autoresearch targets selected-root latency first`
- `### Requirement: Parallel-root autoresearch tracks parallel throughput`
- `### Requirement: Lazy-eval autoresearch metric changes start a new session segment`
- `METRIC parallel_all_roots_total_wall_ns=...`
- `BASELINE_BUNDLE_OUT=<path>` and `BASELINE_CARGO_TARGET_DIR=<path>`

The earlier lazy-benchmark requirements and existing parallel-root throughput evidence requirement remain present outside the modified block, so the sync did not drop unrelated performance-spec clauses.
