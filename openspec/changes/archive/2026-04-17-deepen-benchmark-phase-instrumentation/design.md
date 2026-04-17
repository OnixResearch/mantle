# Design: Deepen benchmark phase instrumentation

## Context

The current benchmark suite is intentionally conservative. Each checked-in
workload emits only the phase metrics it can support honestly, and the compare
path already tolerates different workload shapes. That keeps the suite honest,
but it also means the current workflow benchmarks mostly tell operators which
workload moved, not which internal phase moved inside a realistic workflow.

The next useful step is not a giant benchmark system. It is a narrow extension
that gives maintainers at least one benchmark path with multiple real phase
boundaries, keeps store lookup or persistence metrics optional and evidence-
driven, and preserves the rule that missing metrics are omitted rather than
fabricated.

## Goals / Non-Goals

**Goals:**
- add at least one checked-in workflow benchmark that emits more than one named
  phase metric in a single run
- define when store lookup or persistence metrics are allowed to appear
- define how comparison behaves when one workload omits a phase metric
- keep the suite local-first and based on checked-in fixtures or deterministic
  local state

**Non-Goals:**
- introduce a remote benchmark service
- require every workload to expose every possible phase
- add fake zero-valued metrics just to make reports rectangular
- force benchmark-only helpers into the shipped library path

## Decisions

### 1. Add one multi-phase workflow benchmark before broadening the suite

**Choice:** require at least one checked-in workflow benchmark that emits more
than one named phase metric in the same result bundle entry.

**Rationale:** a single honest multi-phase workflow is enough to prove the
instrumentation boundary and comparison semantics without overbuilding the
system.

**Implementation:** add a benchmark path that times at least two distinct
workflow phases inside one run, such as evaluation + conversion or evaluation +
build-related execution when both boundaries are observable.

### 2. Keep omission explicit when a phase is not observable

**Choice:** if a workload cannot isolate a phase honestly, the bundle omits that
phase metric entirely.

**Rationale:** zero-valued or synthetic metrics would make comparison output
look precise while hiding that the benchmark never observed the phase.

**Implementation:** result bundles keep `total_wall_ns`, may keep an empty
`phase_metrics` list, and comparison reports missing metrics as missing instead
of converting them to zeroes.

### 3. Treat store lookup and persistence as opt-in workflow phases

**Choice:** store lookup or store persistence metrics appear only when the
benchmark path genuinely crosses those boundaries and the timing window is clear.

**Rationale:** some checked-in workloads are pure evaluation or pure conversion.
Those workloads should not claim store behavior they never exercised.

**Implementation:** add a store-aware workflow benchmark or plan-style probe
that can isolate store lookup and/or persistence timing. Keep those metrics out
of unrelated workloads.

### 4. Keep sparse comparison deterministic

**Choice:** compare matched workload names, then compare only the metric names
present in both runs. Report missing metrics separately.

**Rationale:** sparse bundles are an expected outcome of honest omission.
Comparison must stay stable and readable without forcing every workload into the
same metric shape.

**Implementation:** comparison output keeps per-workload missing-from-baseline
and missing-from-fresh lists, highlights the largest win/regression only among
matched metrics, and never fabricates a missing metric value.

### 5. Document a local-first operator workflow around baselines

**Choice:** document the expected baseline capture, candidate capture, and
compare flow for same-host optimization work.

**Rationale:** instrumentation is only useful if maintainers use it the same way
across optimization sessions.

**Implementation:** docs explain when to use the smoke path, when to use the
full workflow path, where to keep baseline bundles, and how to interpret sparse
metric comparisons.

## Risks / Trade-offs

**[Instrumentation drift]**
A benchmark path can silently stop measuring the phase it claims.

**Mitigation:** add tests that assert expected phase names for multi-phase
workloads and explicit omission for opaque workloads.

**[Sparse comparison confusion]**
Operators may misread a missing metric as a zero metric.

**Mitigation:** comparison output must list missing metrics explicitly and docs
must explain omission semantics.

**[Benchmark maintenance cost]**
Adding many phase-specific workflows could turn into a second build system.

**Mitigation:** start with one or a few high-value workflow paths and expand
only when a real optimization question demands it.
