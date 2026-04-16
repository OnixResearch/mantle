## ADDED Requirements

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
