Evidence-ID: align-lazy-eval-autoresearch-metrics-v4-jsonl-segmentation
Task-ID: V4
Artifact-Type: verification-note
Covers: performance.autoresearch.metricsegmentation, performance.autoresearch.metricsegmentation.parallelafterselected, performance.autoresearch.metricsegmentation.activereporootsegment
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-21

`autoresearch.jsonl` inspection summary:
- config count = 3
- older config segments still include `metricName":"selected_root_total_wall_ns"`
- latest config record is line 28 with `metricName":"parallel_all_roots_total_wall_ns"`
- first non-config run after that latest config record is line 29 (`run":26`)
- that first run record keeps the segment-owned top-level `metric` field for the active parallel-root segment (`803100983` under the `parallel_all_roots_total_wall_ns` config segment)

This preserves older selected-root history in the same file while keeping the latest checked-in config segment aligned with the active repo-root parallel-root session. The required parallel-root config segment and first following run record were already present in the checked-in file, so this change confirmed and relied on that active segment instead of appending a redundant same-metric segment.
