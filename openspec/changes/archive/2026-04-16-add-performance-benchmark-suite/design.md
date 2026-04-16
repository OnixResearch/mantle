# Design: Add performance benchmark suite

## Context

Performance work in crunch spans multiple layers:

- Nickel evaluation and derivation extraction
- record conversion and store-path construction
- closure lookup and substitution planning
- local build orchestration and finalization
- self-build and bootstrap subflows

Those layers have different bottlenecks and different setup costs. A single
wall-clock number is useful, but it does not tell maintainers whether a change
helped evaluation, cache lookup, build execution, or simply altered warm-cache
state.

## Goals / Non-Goals

**Goals:**

- make representative performance measurement a checked-in workflow
- capture phase-separated metrics for the main crunch execution layers
- produce result bundles that can be compared across commits and machines
- keep benchmark-only dependencies out of the shipped runtime path
- keep the benchmark suite local-first and scriptable

**Non-Goals:**

- enforce a perf budget in CI immediately
- cover every command or every fixture shape in the first version
- hide host metadata that materially affects comparability
- treat one machine's absolute timings as globally authoritative

## Decisions

### 1. Use a checked-in workload matrix

**Choice:** ship a small checked-in workload matrix instead of ad hoc benchmark
commands pasted into issue threads.

**Rationale:** representative fixed workloads are the only way to compare
optimization attempts honestly over time.

**Implementation:** the initial matrix includes at least an eval-focused
workload, a derivation conversion workload, a substitution or fetcher-focused
workload, and a build-graph workload. A reduced self-build subphase workload may
stand in for the full proof path as long as the measured phase boundary is named
explicitly. The CI-compatible smoke path uses checked-in fixtures only, avoids
network access, and runs a reduced subset rather than the full matrix.

### 2. Emit result bundles, not only terminal text

**Choice:** benchmark runs produce machine-readable result bundles with commit,
environment, workload, and metric data.

**Rationale:** optimization work needs artifacts that can be compared later and
fed into local dashboards or review notes. Human-only text makes that tedious
and error-prone.

**Implementation:** each run writes JSON or JSONL under a benchmark output
directory and records at least commit, command, workload name, cache warm/cold
mode, logical store prefix, hermeticity mode when relevant, and phase metrics.

### 3. Separate phase timing where crunch has real boundaries

**Choice:** the harness records named metrics for the layers crunch already
models explicitly, rather than inventing arbitrary micro-phases.

**Rationale:** a benchmark suite should mirror architecture boundaries the repo
can act on.

**Implementation:** collect metrics such as evaluation time, conversion time,
store lookup or persistence time where observable, cache or substitution time,
dispatch or wait time, and total wall time when the workload exposes those
boundaries cleanly. If a workload cannot separate a phase honestly, the bundle
omits that submetric instead of fabricating one.

### 4. Keep benchmark-only dependencies out of the runtime path

**Choice:** any new benchmark-only crate or dependency stays outside the shipped
runtime dependency path.

**Rationale:** measurement tooling should not quietly enlarge the bootstrap or
runtime surface of crunch.

**Implementation:** benchmark helpers live behind benchmark-only targets,
scripts, or dev-dependencies, and validation checks the runtime dependency set
stays unchanged.

### 5. Keep baselines local-first and explicit

**Choice:** comparison runs use saved local baselines chosen by the operator or
by the autoresearch harness, not an implicit remote service.

**Rationale:** crunch already favors local-first workflows. Baseline files in
or near the repo are easy to inspect and easy to reproduce.

**Implementation:** add a comparison entry point that takes a saved result bundle
and a fresh one, computes deltas for matching workloads and metrics, and flags
changes above configurable percentage or absolute thresholds.

## Risks / Trade-offs

**[Benchmark drift]**
Fixtures can stop representing real workloads.

**Mitigation:** keep the matrix small, named, and easy to revise when the repo's
real workloads change.

**[Measurement noise]**
Host variance can hide small wins.

**Mitigation:** record host metadata, cache mode, and repeat counts; compare on
the same host before claiming small improvements.

**[Harness complexity]**
A giant benchmark system can become its own maintenance burden.

**Mitigation:** start with a narrow workload matrix and phase naming aligned to
current architecture boundaries.
