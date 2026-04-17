## ADDED Requirements

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
