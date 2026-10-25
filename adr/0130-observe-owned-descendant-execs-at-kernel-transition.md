# ADR 0130: Observe owned descendant execs at the kernel transition

## Status

Proposed (2026-10-23). Integrated runtime verification remains required; this
record does not accept the Rust differential or wait-diagnostic Cairn changes.

## Context

The retained wrapper-isolated ordinary Cargo baseline rejected
`generated-check/b/forced-unchanged`: Cargo completed and reported a non-fresh
application artifact, but the independent process capture contained no compiler.
The complete attempt lasted 37,378,499 ns. A `/proc` sampling pass can miss a
compiler whose entire lifetime lies between passes. Artifact freshness does not
replace evidence that the selected executable actually ran. The strict
`forced-compiler-reentry-unobserved` predicate therefore remains unchanged.

Separately, the normal derivation sandbox masks random devices. Reading a
wrapper-launch nonce from `/dev/urandom` made diagnostics silently unavailable
although the compiler still ran. Observation must not require broader sandbox
execution authority.

## Decision Drivers

- Retain exact executable/process facts for successful short-lived execs.
- Preserve real exits, signals, fault targeting, bounded teardown and sampling.
- Never infer compiler execution from Cargo artifacts or reduce polling latency
  and present that as complete lifecycle coverage.
- Keep diagnostic identity, timing and observation outside build/cache authority.

## Decision

The Linux owned-process adapter in `crunch-rustc-wrapper` uses `PTRACE_TRACEME`
for its directly owned command and bounded fork/vfork/clone/exec notifications
for descendants. It records `/proc/<pid>/exe` and command identity while the
kernel has stopped the successful exec, before the executable's first
instruction. It then resumes that exact tracee. Waits target only its owned
tracees, not arbitrary children of the caller. Ordinary signals are forwarded;
a real stopped process remains subject to the existing owned deadline.

The observer admits at most 128 simultaneous tracees and 65,536 lifecycle
notifications per supervised command. Overflow or failed observation is
nonpassing, not complete coverage. Existing process-group, task, output,
filesystem, elapsed-time and teardown bounds remain independent. Exit-kill and
bounded owned cleanup prevent an error path from leaving running tracees.
Polling remains for resource sampling and watchdog observations, not proof that
all descendant execs were seen. The kernel observation adds measured overhead;
comparisons need fresh, equally instrumented cohorts and unchanged strict
population/equality requirements.

Clone tracing makes every thread of an observed process a tracee, and a traced
thread-group leader is reapable only after each traced thread has been reaped.
Teardown therefore reaps by owned process group (`waitpid(-root, WNOHANG |
__WALL)`), including threads whose creation event was never drained, and the
owned-child wait is bounded by the teardown deadline instead of blocking. An
observation error during teardown is recorded as the run's failure; it does not
abandon reaping. The first differential run with the observer deadlocked in a
blocking leader wait during a deadline case; that run is retained as evidence.

The ordinary Cargo wait wrapper derives its invocation-local launch token from
the already selected invocation, process ID and monotonic launch instant. This
is a diagnostic discriminator, not an authentication nonce, source identity,
receipt preimage or cache key. It does not open a random device or change the
derivation's random-device policy.

## Consequences

Linux ptrace permission is an explicit prerequisite for this owned observation
surface. No artifact-based fallback, fabricated process fact, faster polling
substitute, new compiler provider or early-interface treatment is admitted.
Signal, short-lived child, stopped-child and thread-bound-overflow teardown
regression scenarios plus the real ordinary/native differential and sandbox Cargo
paths must run after integration.

The Rust harness and execution-adapter maintainers own this mechanism and its
kernel/toolchain compatibility review. Observed process identity is not compiler
correctness, sandbox completeness, full Cargo compatibility, performance,
reproducibility, bootstrap or release eligibility.
