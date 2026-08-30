# ADR 0087: Acknowledge ptrace seize before the root stop

## Status

Accepted (2026-08-30)

## Context

ADR 0086 introduced `PTRACE_SEIZE` supervision for protected `execve` and
`execveat` calls. Its first implementation used one child-to-tracer pipe. The
child wrote its pid and immediately raised `SIGSTOP`.

V84 completed the MRustC-to-Rust-1.90.0 stage with 88,038 allowed events. It
then failed before Rust 1.91.1 execution. The second root reported
`PTRACE_SETOPTIONS: ESRCH` after `waitpid` returned `SIGSTOP`.

This failure exposed a scheduling race. The tracer can seize the child before
or after the child enters a process group stop. A pre-seize group-stop report
is not a guaranteed ptrace-stop. Ptrace operations that require a ptrace-stop
can therefore return `ESRCH` even while the child exists.

`PTRACE_SEIZE` already applies its complete option mask immediately. A later
`PTRACE_SETOPTIONS` call is redundant and cannot repair the ambiguous stop.
Blind retries would hide the state error and weaken fail-closed behavior.

## Decision Drivers

- Guarantee that every root becomes a tracee before it raises `SIGSTOP`.
- Preserve the pid-pipe, `SIGSTOP`, and `PTRACE_SEIZE` ownership model.
- Keep concurrent root launches independent.
- Keep all attach, pipe, wait, and signal failures fail closed.
- Preserve policy, audit, promotion, and reconciliation semantics.

## Decision

Use a separate acknowledgment pipe for each supervised root.

Before spawn, the supervisor allocates a bounded, monotonic handshake token.
It stores the acknowledgment writer under that token. The child installs its
seccomp filter, writes one atomic `{pid, token}` record to the root pipe, and
blocks on its acknowledgment reader.

The tracer reads the record, removes the matching writer, and calls
`PTRACE_SEIZE` with the complete option mask. Only after `PTRACE_SEIZE`
succeeds does the tracer write the exact acknowledgment byte. The child then
raises `SIGSTOP`.

Because the tracer owns the child before this `SIGSTOP`, `waitpid` observes a
signal-delivery ptrace-stop. The tracer suppresses the handshake signal and
continues the child. No later `PTRACE_SETOPTIONS` call is permitted.

A missing token, duplicate token, invalid acknowledgment, closed pipe, failed
seize, unexpected stop, or short read or write records a technical denial and
kills the affected root.

## Alternatives Considered

### Remove `PTRACE_SETOPTIONS` but keep the unacknowledged stop

Rejected. `PTRACE_CONT` can fail for the same reason when the observed stop
predates ptrace ownership.

### Retry ptrace operations after `ESRCH`

Rejected. `ESRCH` also reports tracee death and wrong tracing state. Retrying
would make authority depend on timing.

### Use one shared acknowledgment pipe

Rejected. Under concurrent launches, one child could consume another child's
acknowledgment before its own seize completes.

### Replace `SIGSTOP` with `PTRACE_INTERRUPT`

Rejected for this repair. The accepted boundary uses the explicit child stop,
and the acknowledged protocol removes its race.

## Consequences

- Each pending root holds one bounded pipe pair until seize completes.
- Root-pipe records grow from a pid to an atomic pid-and-token record.
- The supervisor can safely return to zero tracees and later attach another
  root stage.
- Tests cover sequential roots after quiescence, concurrent roots, invalid
  acknowledgments, and closed acknowledgment pipes.
- V84 remains failed evidence. A new promoted run must re-establish the Rust
  provider from the first affected stage.
