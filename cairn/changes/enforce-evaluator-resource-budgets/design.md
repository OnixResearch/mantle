# Design: Evaluator resource budgets

## Context

The benchmark suite records `total_wall_ns`, phase wall times, selected-root latency, explicit top-level root force requests, and explicit nonselected root request counts. It does not record peak RSS. `EvaluationSession` runs in process, so a timeout cannot reliably stop evaluator work or recover memory.

The selected-root benchmark names its counts explicitly because Nickel can still deep-export more data than the public request count implies. The new report must preserve that distinction.

## Decisions

### Decision: Author policy in typed Nickel

**Choice:** Add a typed Nickel evaluation policy with named limits for source bytes, import roots, imported modules, discovered roots, selected roots, diagnostics, diagnostic bytes, request bytes, response bytes, wall time, CPU time, peak RSS, worker count, and shutdown grace.

Runtime Rust consumes checked-in generated policy data. Policy has observe-only and enforce modes. Unknown fields, zero invalid bounds, overflow, contradictory limits, and unsupported strict mechanisms fail before evaluation.

**Rationale:** Limits are reviewed configuration. They must not appear as unexplained numeric literals across evaluator code.

### Decision: Use an owned evaluator worker for strict budgets

**Choice:** Strict evaluation runs in a child process through a versioned length-bounded protocol. The parent sends source identity, source bytes or confined source descriptor, import descriptors, selected roots, evaluator descriptor, and policy identity.

The worker returns one terminal response with status, output or error summary, bounded diagnostics, evaluator observations, and worker-side identity. Protocol input does not grant arbitrary filesystem paths or process execution.

**Rationale:** A process boundary lets the parent enforce deadlines, terminate hangs, reap crashes, and recover allocations.

### Decision: Separate pure admission and outcome classification from process control

**Choice:** Pure cores validate requests, select supported policy mode, classify observations, apply truncation rules, and build terminal reports. The shell owns process spawn, pipes, clocks, platform limits, resource sampling, signals, cancellation, kill, reap, and bounded stderr capture.

Every non-trivial shell step consumes or returns typed state. No renderer can convert worker failure into evaluation success.

**Rationale:** Resource policy remains unit-testable without starting an evaluator.

### Decision: Enforce only mechanisms the host can prove

**Choice:** The worker shell probes supported deadline, CPU, and memory mechanisms. Strict wall-time policy requires owned timeout and teardown. Strict memory policy requires a supported enforceable address-space, cgroup, job-object, or equivalent limit plus peak observation when the platform provides it.

When strict policy requests an unavailable mechanism, evaluation fails before worker launch with `evaluation-budget-unsupported`. Observe-only mode can record unavailable metrics as absent with a reason.

**Rationale:** Sampling RSS is not memory enforcement. A configured limit must not become a false claim.

### Decision: Keep metric roles explicit

**Choice:** Reports distinguish public request counts, evaluator-provided observations, and process resource observations.

`explicit_top_level_root_force_count` remains a Mantle API request count. It is not renamed as a thunk count. Actual nonselected evaluation is reported only when the evaluator supplies an honest supported observation. Missing observations remain missing.

**Rationale:** A precise metric name is better than a broad but false laziness claim.

### Decision: Make cancellation terminal and owned

**Choice:** Parent cancellation transitions the worker through request, grace, terminate, kill, and reap states with named deadlines. The final outcome records cancellation, teardown result, response presence, and any bounded worker stderr.

A late worker response cannot replace cancellation. An unreaped worker or failed limit setup blocks a clean terminal claim.

**Rationale:** Cancellation that leaves evaluation running does not enforce a budget.

### Decision: Gate regressions only within compatible cohorts

**Choice:** Benchmark bundles bind host class, target, evaluator identity, toolchain, policy, fixture, repeat policy, measurement support, and warm or cold mode. Comparison enforces thresholds only between compatible cohorts.

Resource thresholds use named absolute and percentage policy values. Missing metrics are reported, not treated as zero. Peak RSS comparison requires the same observation mechanism.

**Rationale:** Cross-host memory and timing values are not directly comparable by default.

### Decision: Roll out through observe-only parity

**Choice:** First run the worker beside the current path on bounded positive and negative fixtures. Compare selected output, error class, diagnostics, root labels, and explicit request observations. Unexplained semantic drift blocks strict cutover.

Performance differences do not prove semantic drift by themselves. The existing path remains available as a bounded rollback until one full accepted validation cycle completes.

**Rationale:** Process isolation changes orchestration and error plumbing around a mature evaluator path.

## Validation

Positive fixtures cover small evaluation, selected roots, import roots, expected evaluator errors, observe-only metrics, strict deadline, supported memory policy, deterministic reports, compatible benchmark comparison, and clean cancellation.

Negative fixtures cover oversized source, excess imports or roots, malformed frames, trailing bytes, response overflow, timeout, memory exhaustion, worker panic, worker signal, stderr flood, late success after cancellation, failed reap, unsupported strict policy, cohort mismatch, and missing metric handling.

## Risks / Trade-offs

- Worker startup adds latency to strict evaluation.
- Address-space limits can reject valid allocator behavior before RSS reaches the configured bound.
- Cross-platform resource mechanisms differ and can leave some strict modes unsupported.
- Dual-run validation increases temporary evaluation cost.
- Evaluator-internal thunk metrics remain unavailable unless Nickel exposes honest observations.
