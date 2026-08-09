# ADR 0074: Enforce evaluation budgets with an owned worker

## Status

Accepted

## Context

Nickel evaluation ran in the Mantle process. A timeout could report failure, but it could not stop evaluator work or recover its memory.

The benchmark suite also recorded wall time only. It did not bind CPU time, peak RSS support, policy identity, or teardown facts.

## Decision Drivers

- Stop timed-out or cancelled evaluation work.
- Reject oversized messages before allocation.
- Keep admission and terminal classification pure.
- Keep process effects in a thin shell.
- Separate memory enforcement from RSS observation.
- Preserve the in-process path as an observe-only rollback.
- Compare resource metrics only in compatible cohorts.
- Keep unsupported facts explicit.

## Decision

Strict evaluation runs in a hidden same-binary worker process. The request and policy have separate BLAKE3 identities.

The protocol uses one fixed-width, length-bounded request and response. The parent captures bounded stderr and rejects malformed, oversized, or trailing protocol data.

On Linux, the parent applies `RLIMIT_CPU` and `RLIMIT_AS`. The worker applies Landlock read rules for admitted import roots before evaluation. `RLIMIT_AS` is an address-space mechanism, not RSS enforcement. The worker reports CPU time and peak RSS through `getrusage` when it returns a valid response.

The parent owns the deadline, cancellation, termination, kill, and reap sequence. Timeout and cancellation have terminal precedence over late responses.

The pure `crunch-eval-budget-core` crate owns policy admission, request identity, framing checks, truncation, terminal classification, and report construction. The Mantle binary owns files, clocks, processes, host limits, and publication.

Observe-only mode uses the existing in-process evaluator. It records operation-scoped CPU and peak RSS facts as unavailable.

Benchmark bundles bind a resource cohort. Thresholds apply only when host, target, evaluator, toolchain, policy, fixtures, repeats, warm state, and measurement support match.

## Alternatives Considered

### Use an in-process watchdog thread

Rejected. Rust cannot safely stop the evaluator thread or guarantee that work ended after a timeout.

### Present sampled RSS as memory enforcement

Rejected. Observation does not constrain allocation.

### Adopt a generic process runner first

Deferred. Evaluator framing, identities, root semantics, and mechanism support need a stable product contract first.

### Remove the in-process evaluator path

Rejected. Observe-only mode provides a bounded rollback during rollout.

## Consequences

- Strict evaluation has worker startup cost.
- Linux strict mode has enforceable time and address-space boundaries.
- Other hosts fail closed when required strict mechanisms are absent.
- Reports distinguish enforced, observed, unavailable, and truncated facts.
- Explicit force counts remain Mantle API facts, not Nickel thunk counts.
- Budget compliance does not prove evaluator correctness, reproducibility, build correctness, or release eligibility.
