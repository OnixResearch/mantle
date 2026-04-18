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

Any future autoresearch loop for lazy root evaluation MUST optimize
`selected_root_total_wall_ns` first and treat the remaining lazy-eval metrics as
secondary guardrails.

The autoresearch harness MUST use a workload and sampling strategy that avoids
sub-millisecond noise dominating decisions.

#### Scenario: Autoresearch session uses selected-root metric as primary target

- GIVEN an autoresearch session is created for lazy root evaluation
- WHEN the session configuration is inspected
- THEN the primary metric is `selected_root_total_wall_ns`
- AND the session keeps the lazy-eval discovery/force counts and
  `all_roots_total_wall_ns` as secondary metrics or checks
- AND the workload or repeated-sampling strategy is chosen so the primary
  metric is stable enough to compare across runs on the same host

