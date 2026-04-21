## MODIFIED Requirements

### Requirement: Autoresearch targets selected-root latency first

For lazy selected-root optimization sessions, the autoresearch loop MUST
optimize `selected_root_total_wall_ns` first and treat the remaining lazy-eval
metrics as secondary guardrails.
ID: performance.autoresearch.selectedrootprimary

The autoresearch harness MUST use a workload and sampling strategy that avoids
sub-millisecond noise dominating decisions.

Parallel-root throughput sessions are governed by `Parallel-root autoresearch
tracks parallel throughput`, not by this selected-root requirement.

#### Scenario: Selected-root session keeps selected-root primary metric
ID: performance.autoresearch.selectedrootprimary.sessionconfig

- GIVEN an autoresearch session is created for selected-root lazy-eval tuning
- WHEN the session configuration is inspected
- THEN the primary metric is `selected_root_total_wall_ns`
- AND the session keeps the lazy-eval discovery/force counts and
  `all_roots_total_wall_ns` as secondary metrics or checks
- AND the workload or repeated-sampling strategy is chosen so the primary
  metric is stable enough to compare across runs on the same host

## ADDED Requirements

### Requirement: Parallel-root autoresearch tracks parallel throughput

For lazy parallel-root optimization sessions, the repo-local autoresearch loop MUST
optimize `parallel_all_roots_total_wall_ns` first and keep the selected-root
lazy metrics visible as guardrails.
ID: performance.autoresearch.parallelrootprimary

The checked-in repo-root autoresearch workflow MUST continue to persist
`selected_root_total_wall_ns`, `root_discovery_wall_ns`,
`selected_root_force_wall_ns`, `explicit_top_level_root_force_count`,
`explicit_nonselected_root_force_count`, and `all_roots_total_wall_ns` in the
same generated benchmark bundle that reports
`parallel_all_roots_total_wall_ns`, so reviewers can inspect the guardrail
values from that persisted artifact.

The checked-in runner output MUST surface
`parallel_all_roots_total_wall_ns` as the primary emitted metric for the active
parallel-root session.

#### Scenario: Repo-root parallel session names the throughput metric
ID: performance.autoresearch.parallelrootprimary.reporootsession

- GIVEN the maintainer inspects the active lazy-eval autoresearch files
- WHEN they read `autoresearch.md` and run `./autoresearch.sh`
- THEN the documented primary metric is `parallel_all_roots_total_wall_ns`
- AND the runner prints `METRIC parallel_all_roots_total_wall_ns=...`
- AND the generated benchmark bundle from that run includes the selected-root
  lazy metrics as guardrails

### Requirement: Lazy-eval autoresearch metric changes start a new session segment

The repo-local `autoresearch.jsonl` history MUST record a new config header before any run
using a new primary lazy-eval optimization metric is logged.

The metric-switch segment rollover is a documented operator step in
`autoresearch.md`; operators must create the active config segment there before
invoking `./autoresearch.sh` for baseline or run output.
ID: performance.autoresearch.metricsegmentation

A segment-start config record MUST include at least `"type":"config"` and
`"metricName":"<primary-metric>"` so reviewers can identify which lazy-eval
primary metric owns the following runs.

Historical runs MAY remain in the same file, but runs from different lazy-eval
primary metrics MUST NOT share one config header segment or one baseline.

Run records logged after a config segment MUST use that segment's primary lazy-
eval metric as their top-level `metric` value.

A fresh isolated baseline means a new benchmark bundle path and an isolated
Cargo target directory that the operator does not share with unrelated Cargo
build or test work during the measurement window.

The checked-in repo-root workflow MUST print `BASELINE_BUNDLE_OUT=<path>` and
`BASELINE_CARGO_TARGET_DIR=<path>` so reviewers can inspect the recorded
baseline inputs.

The no-concurrent-Cargo operator procedure MUST be documented in
`autoresearch.md` as part of the checked-in workflow for this session.

#### Scenario: Parallel-root session starts after selected-root history
ID: performance.autoresearch.metricsegmentation.parallelafterselected

- GIVEN `autoresearch.jsonl` already contains a selected-root config segment
- WHEN the repo switches the active lazy-eval session to parallel-root
  throughput tuning
- THEN a new config record for `parallel_all_roots_total_wall_ns` appears
  before the first new run
- AND the old selected-root runs remain in the older segment
- AND the new session captures a fresh isolated baseline for the new segment
- AND the workflow records the new baseline bundle path and isolated Cargo
  target directory used for that baseline

#### Scenario: Active repo-root parallel session owns the latest config segment
ID: performance.autoresearch.metricsegmentation.activereporootsegment

- GIVEN the repo-root lazy-eval workflow is the active parallel-root
  throughput session
- WHEN a maintainer inspects the checked-in `autoresearch.jsonl`
- THEN the latest config record has `"type":"config"`
- AND that same latest config record has
  `"metricName":"parallel_all_roots_total_wall_ns"`
- AND an older selected-root config segment still appears earlier in the file
- AND the first non-config run record after the latest config segment uses
  `parallel_all_roots_total_wall_ns` as its segment-owned top-level `metric`
  value

#### Scenario: Workflow documents and prints baseline-isolation inputs
ID: performance.autoresearch.metricsegmentation.workflowprocedure

- GIVEN a maintainer reads `autoresearch.md` and runs `./autoresearch.sh` for a
  fresh baseline
- WHEN the workflow prepares the active parallel-root session baseline
- THEN `autoresearch.md` tells operators not to share the measurement window
  with unrelated Cargo build or test work
- AND the runner prints `BASELINE_BUNDLE_OUT=<path>` for the baseline bundle
- AND the runner prints `BASELINE_CARGO_TARGET_DIR=<path>` for the isolated
  Cargo target directory
