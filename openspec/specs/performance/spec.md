# performance Specification

## Purpose
TBD - created by archiving change add-performance-benchmark-suite. Update Purpose after archive.
## Requirements
### Requirement: Checked-in representative benchmark suite

The repo MUST provide a checked-in benchmark suite for representative crunch
workloads.

The initial suite MUST cover at least:
- an evaluation-focused workload,
- a derivation conversion workload,
- a substitution- or fetcher-focused workload,
- and a build-graph workload.

The repo MUST also provide at least one CI-compatible local smoke benchmark
invocation that exercises the benchmark workflow without requiring the full
benchmark matrix.

That smoke path MUST use checked-in fixtures only, MUST NOT require network
access, and MUST be small enough for ordinary local checks.

#### Scenario: Maintainer runs the checked-in suite

- GIVEN a developer working from the repo checkout
- WHEN they invoke the documented benchmark entry point
- THEN crunch runs the checked-in workload matrix
- AND the run does not depend on unpublished one-off commands

#### Scenario: Local checks can run a benchmark smoke path

- GIVEN an ordinary local check run that cannot afford the full benchmark
  matrix
- WHEN it invokes the benchmark smoke entry point
- THEN the benchmark workflow runs at least one representative workload
- AND the invocation uses checked-in fixtures with no network dependency
- AND the invocation is suitable for CI-compatible local checks

### Requirement: Benchmark result bundles are machine-readable

Every benchmark run MUST emit a machine-readable result bundle.

Each workload result MUST record at least the commit identifier, workload name,
command or harness entry point, cache mode, and total wall-clock metric.

#### Scenario: Benchmark run produces reusable artifact

- GIVEN a completed benchmark run
- WHEN the result bundle is inspected later
- THEN it includes the commit and workload identity for each recorded result
- AND another tool can compare it without scraping human terminal text

### Requirement: Benchmark bundles disclose relevant execution context

Benchmark result bundles MUST disclose the execution context needed to judge
comparability.

At minimum the bundle MUST record toolchain identity, logical store prefix,
and any relevant hermeticity or warm-cache mode used by the workload.

#### Scenario: Operator can see why two runs differ

- GIVEN two benchmark bundles from different runs
- WHEN an operator compares them
- THEN they can see whether the toolchain, logical store prefix, or cache mode
  differed between those runs

### Requirement: Phase-separated metrics follow architecture boundaries

The benchmark harness MUST record phase-separated metrics rather than only a
single total duration when a workload exposes honest crunch phase boundaries.

At minimum the suite MUST be able to distinguish evaluation, conversion,
store lookup or persistence where observable, substitution or fetch planning,
and build-related execution for the workloads that exercise those phases.

#### Scenario: Evaluation and build cost are not conflated

- GIVEN a workload that exercises both derivation evaluation and build
  execution
- WHEN the benchmark harness records its result
- THEN the result contains separate named metrics for those phases when both
  boundaries are observable
- AND a maintainer can tell which phase changed across runs

### Requirement: Baseline comparison is supported

The benchmark workflow MUST support comparison between a fresh result bundle and
a saved baseline bundle.

The comparison output MUST match workloads by stable workload name and report
metric deltas for matching phase names.

#### Scenario: Maintainer compares a candidate optimization with baseline

- GIVEN a saved benchmark baseline and a fresh benchmark result bundle
- WHEN the comparison entry point runs
- THEN it reports the delta for each matching workload and metric
- AND it highlights the largest regressions or wins

### Requirement: Benchmark-only dependencies stay out of the runtime path

Any benchmark-only crates or dependencies added for this suite MUST stay out of
the shipped runtime dependency path.

#### Scenario: Benchmark helper does not change runtime dependency surface

- GIVEN the repo adds a benchmark-only helper dependency
- WHEN the runtime build targets are inspected
- THEN that dependency is reachable only from benchmark-only targets,
  scripts, or dev-only paths
- AND ordinary runtime builds do not gain a new runtime dependency because of
  the benchmark suite

### Requirement: Multi-phase workflow benchmarks expose real internal boundaries

The benchmark suite MUST provide at least one checked-in workflow benchmark that
records more than one named phase metric in a single workload result.

That workflow benchmark MUST use only checked-in fixtures or deterministic local
state.

Its phase metrics MUST correspond to real crunch workflow boundaries rather than
invented subdivisions.

#### Scenario: Multi-phase workflow result records more than one boundary

- GIVEN a maintainer runs the checked-in workflow benchmark
- WHEN the result bundle is written
- THEN at least one workload result contains more than one named phase metric
- AND those phase metric names correspond to real workflow boundaries exercised
  by that workload

### Requirement: Unobservable phases are omitted, not fabricated

A benchmark workload MUST omit any phase metric whose timing boundary is not
honestly observable in that workload.

The result bundle MUST still keep `total_wall_ns` for that workload.

#### Scenario: Opaque workload emits total only

- GIVEN a workload that can report honest total wall time but cannot isolate a
  named internal phase
- WHEN the benchmark harness records its result
- THEN the workload result omits that unobservable phase metric
- AND the workload result still records `total_wall_ns`
- AND the harness does not fabricate a zero or placeholder phase value

### Requirement: Store lookup or persistence metrics require real store activity

The benchmark suite MUST record store lookup or store persistence metrics only
for workloads that actually exercise those boundaries and can isolate them
honestly.

A workload that does not cross a store lookup or persistence boundary MUST NOT
claim those metrics.

#### Scenario: Store metric appears only for store-aware benchmark

- GIVEN a workload that performs benchmarked store lookup or store persistence
  work with a clear timing boundary
- WHEN the result bundle is written
- THEN the workload result may contain the corresponding named store phase
  metric
- AND unrelated workloads omit that store metric

### Requirement: Sparse metric comparison remains explicit

The comparison workflow MUST compare matched workload names and matched metric
names without fabricating missing metrics.

When a workload metric is present in only one bundle, the comparison result MUST
report it as missing from the other bundle.

#### Scenario: Sparse phase metrics compare without synthetic zeroes

- GIVEN a saved baseline bundle and a fresh bundle for the same workload name
- AND one of those workload results omits a phase metric because the phase is
  not observable there
- WHEN the comparison entry point runs
- THEN it reports that metric as missing from one side
- AND it does not invent a zero-valued metric to force a match

### Requirement: Benchmark docs match the checked-in workflow

The repo MUST document the checked-in benchmark workflow in README-linked docs.

That documentation MUST name the smoke benchmark entry point, the full suite
entry point, and the baseline-vs-fresh comparison entry point.

That documentation MUST describe sparse phase metrics as omission, not as
implicit zeroes.

#### Scenario: Reader finds the three benchmark entry points

- GIVEN a contributor wants to run the checked-in benchmark workflow
- WHEN they follow the README benchmark link into the focused benchmark doc
- THEN they can find the smoke command, the full-suite command, and the
  comparison command
- AND those commands match the checked-in benchmark tooling

#### Scenario: Sparse metrics stay explicit in docs

- GIVEN a contributor reads the benchmark comparison guidance
- WHEN they learn how missing phase metrics are reported
- THEN the doc says missing metrics are explicit omissions
- AND it does not imply that absent metrics should be read as zero

### Requirement: Lazy root evaluation benchmarks use honest selected-root metrics

The benchmark suite MUST provide checked-in lazy-evaluation workloads on a
fixed multi-root fixture and MUST emit machine-readable metrics that reflect the
`crunch-eval` API boundary honestly.

At minimum the suite MUST include the following checked-in workload names on
one shared fixture:
- `lazy-root-discovery-wide-package-set`,
- `lazy-selected-root-wide-package-set`,
- and `eager-all-roots-wide-package-set`.

The benchmark/autoresearch metric set MUST include:
- `selected_root_total_wall_ns` as the primary selected-root latency metric,
- `root_discovery_wall_ns`,
- `selected_root_force_wall_ns`,
- `explicit_top_level_root_force_count`,
- `explicit_nonselected_root_force_count`,
- and `all_roots_total_wall_ns`.

`selected_root_total_wall_ns` MUST be defined as the inclusive end-to-end wall
clock from opening the lazy session to obtaining one fully materialized selected
root. It MAY therefore be larger than
`root_discovery_wall_ns + selected_root_force_wall_ns` when session setup or
other caller-visible overhead exists.

`explicit_top_level_root_force_count` and
`explicit_nonselected_root_force_count` MUST count only top-level root values
that crunch explicitly forces through the lazy API. The harness MUST NOT infer
or guess hidden internal Nickel thunk activity and report it as those metrics.

The shared wide package-set fixture MUST be large enough, or repeated-sampled
enough, that the selected-root primary metric is stable above the noise floor
on the reference host before autoresearch begins.

#### Scenario: Selected-root benchmark reports the full lazy metric set

- GIVEN a maintainer runs the checked-in lazy selected-root benchmark
- WHEN the result bundle or autoresearch script output is inspected
- THEN it includes `selected_root_total_wall_ns` and the supporting lazy-eval
  metrics
- AND those metrics are defined at the `crunch-eval` discovery/force boundary,
  not by inferred Nickel-internal events

#### Scenario: Non-selected root force count stays explicit

- GIVEN a lazy selected-root benchmark run on a multi-root fixture
- WHEN crunch forces only one requested root
- THEN `explicit_nonselected_root_force_count` reports how many top-level roots
  other than the selected one crunch explicitly forced
- AND the metric is omitted or fails loudly if the harness cannot measure that
  count honestly

### Requirement: Autoresearch targets selected-root latency first

For lazy selected-root optimization sessions, the autoresearch loop MUST
optimize `selected_root_total_wall_ns` first and treat the remaining lazy-eval
metrics as secondary guardrails.

The autoresearch harness MUST use a workload and sampling strategy that avoids
sub-millisecond noise dominating decisions.

Parallel-root throughput sessions are governed by `Parallel-root autoresearch
tracks parallel throughput`, not by this selected-root requirement.

#### Scenario: Selected-root session keeps selected-root primary metric

- GIVEN an autoresearch session is created for selected-root lazy-eval tuning
- WHEN the session configuration is inspected
- THEN the primary metric is `selected_root_total_wall_ns`
- AND the session keeps the lazy-eval discovery/force counts and
  `all_roots_total_wall_ns` as secondary metrics or checks
- AND the workload or repeated-sampling strategy is chosen so the primary
  metric is stable enough to compare across runs on the same host

### Requirement: Parallel-root autoresearch tracks parallel throughput

For lazy parallel-root optimization sessions, the repo-local autoresearch loop MUST
optimize `parallel_all_roots_total_wall_ns` first and keep the selected-root
lazy metrics visible as guardrails.

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

- GIVEN a maintainer reads `autoresearch.md` and runs `./autoresearch.sh` for a
  fresh baseline
- WHEN the workflow prepares the active parallel-root session baseline
- THEN `autoresearch.md` tells operators not to share the measurement window
  with unrelated Cargo build or test work
- AND the runner prints `BASELINE_BUNDLE_OUT=<path>` for the baseline bundle
- AND the runner prints `BASELINE_CARGO_TARGET_DIR=<path>` for the isolated
  Cargo target directory

### Requirement: Parallel root evaluation has checked-in throughput evidence

The benchmark workflow MUST provide checked-in evidence for serial-versus-
parallel all-root evaluation throughput on `tests/fixtures/wide_package_set.ncl`
or another fixed multi-root fixture with the same `/nix/store`-compatible
constraints.

That fixture MUST expose at least 16 top-level roots so parallel throughput
measurements are meaningful.

That evidence MUST report machine-readable metrics for:
- a serial all-root evaluation path,
- a bounded parallel all-root evaluation path, and
- the concurrency value used for the parallel run.

The workflow MUST keep selected-root lazy metrics visible as guardrails rather
than silently replacing them.

The workflow MUST record evidence only from an isolated benchmark run: no
concurrent Cargo build or test may share the same target directory during the
measurement window.

#### Scenario: Maintainer can inspect serial and parallel all-root metrics

- GIVEN a maintainer runs the checked-in parallel-root benchmark workflow
- WHEN the machine-readable result bundle or comparison artifact is inspected
- THEN it includes metrics for both serial and bounded parallel all-root
  evaluation on the same fixed fixture
- AND it reports the concurrency value used for the parallel run
- AND the evidence comes from an isolated benchmark run rather than a shared
  target directory with unrelated Cargo work

#### Scenario: Selected-root guardrails remain visible

- GIVEN a maintainer reviews evidence for parallel root evaluation
- WHEN they inspect the benchmark artifacts
- THEN selected-root lazy metrics remain present as guardrails
- AND the workflow does not treat wide all-root throughput as permission to hide
  selected-root regressions

