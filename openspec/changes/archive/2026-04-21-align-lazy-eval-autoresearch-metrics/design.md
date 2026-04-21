# Design: align lazy-eval autoresearch metrics with the active workstream

## Context

Crunch now has two completed lazy-eval optimization slices:

1. selected-root latency work from `lazy-root-evaluation`
2. bounded multi-root throughput work from `parallel-root-evaluation`

The main performance spec still reflects the first slice as if it were the only
future autoresearch target. The repo-root harness reflects the second slice:
recent commits tune `parallel_all_roots_total_wall_ns`, and the current
`autoresearch.md`/`autoresearch.sh` pair already names that metric.

The missing rule is not another benchmark. The missing rule is how the repo
records which lazy-eval sub-workstream is active and how it switches the
primary metric without corrupting experiment history.

## Goals / Non-Goals

**Goals**
- remove ambiguity between selected-root and parallel-root autoresearch targets
- keep selected-root metrics visible when tuning parallel-root throughput
- define how `autoresearch.jsonl` records a primary-metric switch
- let the repo-root harness track the active lazy-eval workstream without
  rewriting archived change-local evidence

**Non-Goals**
- redesign the benchmark harness or workload matrix
- add a second repo-root autoresearch entry point
- rewrite archived `openspec/changes/archive/...` evidence
- implement the repo-file updates in this design document

## Decisions

### 1. Split lazy-eval autoresearch policy by optimization boundary

**Choice:** selected-root sessions and parallel-root sessions use different
primary metrics.

Selected-root sessions keep `selected_root_total_wall_ns` as the primary metric.
Parallel-root sessions use `parallel_all_roots_total_wall_ns` as the primary
metric.

**Rationale:** the two sessions optimize different user-visible costs. The
selected-root path optimizes time to one fully materialized root. The
parallel-root path optimizes bounded throughput across many roots. One rule for
both creates the exact ambiguity now present in the repo.

**Implementation:** modify `openspec/specs/performance/spec.md` so the existing
selected-root autoresearch requirement applies only to selected-root sessions,
and add a new parallel-root autoresearch requirement for throughput sessions.

### 2. Repo-root autoresearch follows the active lazy-eval workstream

**Choice:** the top-level `autoresearch.md` and `autoresearch.sh` describe the
current lazy-eval optimization focus, not every historical focus at once.

**Rationale:** the repo already keeps older rationale and evidence in archived
OpenSpec changes. The repo-root autoresearch files need one current target so
operators know what to run next.

**Implementation:** the current active repo-root lazy-eval session is the
parallel-root throughput session. The implementation phase will update the
spec, markdown, shell comments, and emitted primary metric text to say so
explicitly while retaining selected-root guardrails in the same run.

### 3. Primary-metric switches append a new config segment in the existing file

**Choice:** when a lazy-eval session changes primary metric,
`autoresearch.jsonl` keeps the existing file but appends a new config header
before any run using the new metric is recorded.

**Rationale:** truncating the file would throw away selected-root history, while
keeping both metrics under one config header makes the history ambiguous and
breaks baseline comparisons. Appending a new segment preserves old runs without
mixing metric identities.

**Implementation:** the implementation phase will update the checked-in
`autoresearch.jsonl` file in place by appending a new config record with at
least `"type":"config"` and
`"metricName":"parallel_all_roots_total_wall_ns"`, then checking in the first
parallel-root baseline run record under that segment. That keeps the latest
checked-in segment aligned with the active repo-root session, and the first
non-config run after that config header will already use
`parallel_all_roots_total_wall_ns` as its top-level `metric` value.
`autoresearch.md` will document that segment rollover is an operator step
performed before the first new-metric run. `./autoresearch.sh` will then
capture a fresh isolated baseline against that already-active segment before
logging later parallel-root experiments.

### 4. Baseline isolation must be visible in runner output

**Choice:** the repo-root workflow must print the fresh baseline bundle path
and the isolated Cargo target directory used for a metric-switch baseline using
stable `BASELINE_BUNDLE_OUT=<path>` and `BASELINE_CARGO_TARGET_DIR=<path>`
lines.

**Rationale:** otherwise the new isolation rule is not reviewable. A reviewer
needs to see which bundle artifact belongs to the new segment and which target
root was isolated from unrelated Cargo work.

**Implementation:** the implementation phase will make the checked-in workflow
print `BASELINE_BUNDLE_OUT=<path>` plus
`BASELINE_CARGO_TARGET_DIR=<path>`, and the printed Cargo target dir will be a
fresh run-specific non-default directory rather than the repo default `target/`
tree or shared `~/.cargo-target`. `autoresearch.md` will explicitly tell
operators not to share the measurement window with unrelated Cargo build or
test work. Verification tasks will inspect the fresh baseline transcript for
`METRIC parallel_all_roots_total_wall_ns=...`, both stable baseline-path lines,
the non-default run-specific target-dir shape, and the documented
no-concurrent-Cargo procedure alongside the checked-in JSONL config segment and
first following run record.

### 5. Selected-root metrics remain guardrails for parallel-root sessions

**Choice:** parallel-root sessions do not replace or hide selected-root lazy
metrics.

**Rationale:** the archived `parallel-root-evaluation` change already states
selected-root latency is a guardrail, not the main throughput target. The
repo-root autoresearch policy should preserve that contract.

**Implementation:** the new parallel-root requirement will require
`selected_root_total_wall_ns`, `root_discovery_wall_ns`,
`selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
`explicit_nonselected_root_force_count`, and `all_roots_total_wall_ns` to stay
visible alongside `parallel_all_roots_total_wall_ns`. Review proof will come
from the JSON bundle produced by the same `./autoresearch.sh` run, not from a
hand-copied metric summary.

## Risks / Trade-offs

**[Future drift returns]**
A later retarget could change the harness again without updating the spec.

**Mitigation:** require metric-switch segmentation in `autoresearch.jsonl` and
bind the repo-root harness to an explicit requirement in the performance spec.

**[Operators read archived docs instead of repo-root docs]**
Historical selected-root artifacts could still look like the current target.

**Mitigation:** keep archived evidence untouched, but make the repo-root files
state clearly that they represent the current parallel-root session.

**[Guardrails get dropped during cleanup]**
A future cleanup could simplify the harness around the primary metric only.

**Mitigation:** state the guardrail metric set in the spec requirement and in
implementation tasks, not only in prose.
