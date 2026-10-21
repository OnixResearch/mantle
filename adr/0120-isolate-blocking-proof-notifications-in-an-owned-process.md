# ADR 0120: Isolate blocking proof notifications in an owned process

- **Status:** Proposed
- **Date:** 2026-10-18

## Context

The source-recipe executor admits exact sandbox and leaf executions through
Linux seccomp user notifications. Each observation must be durably recorded and
accepted before the executor allows the intercepted syscall to continue.

A notification can disappear between listener readiness and
`SECCOMP_IOCTL_NOTIF_RECV` when its tracee is cancelled. A receiver thread can then
block inside the ioctl even after the leaf exits. An atomic stop flag followed by
thread join does not interrupt that receive. Closing another descriptor reference
is not a reliable way to cancel an already-running file operation.

## Decision drivers

- Bound cancellation and teardown without admitting unobserved execution.
- Preserve exact executable identity and deny unplanned process children.
- Avoid process-global signal handlers and unsupported nonblocking-ioctl claims.
- Retain one owner for the notification helper, descriptors and failed attempts.

## Decision

Start a syscall-only notification pump process before installing the leaf
worker's irreversible filter. Transfer the listener over a private Unix
`SOCK_SEQPACKET` socket, then close the sender's copy before the command forks.
The pump closes unrelated inherited descriptors with `close_range` and owns the
sole listener. Only the pump can block in notification receive; it forwards
fixed-size notifications and applies acknowledged observer responses.

The observer uses polled, nonblocking socket reads. Observation submission and
reply waits honor cancellation and the action deadline. A denied or unrecorded
observation never receives permission to continue. The existing durable journal
and live action admission remain execution authority; the pump is transport, not
an evidence producer or policy decision maker.

A deadline-bounded process-local lock covers protected pump creation, listener
installation and transfer, and the command's first fork/exec. It is released
when `Command::spawn` returns, not when the leaf finishes. This prevents a
concurrent protected command from retaining another launch's transient listener
while its own first exec waits for a decision. Lock admission, observation and
child completion use one absolute action deadline.

An outstanding response acknowledgment is drained until that deadline even if
the leaf has already exited normally. Normal completion must not turn a
successful kernel response into a cancellation error. `clone3` returns `ENOSYS`:
its mutable-memory flags cannot safely authorize `CONTINUE`. Libc can fall back
to scalar `clone`, whose captured flags admit threads but not unplanned process
children after the two planned executable transitions.

Teardown terminates the owned pump and attempts bounded reaping before joining
the observer. Failure to reap is an error, not successful completion. Child code
after process creation uses only syscall-safe operations, with no allocator,
locks or inherited Rust destructors. Do not replace this boundary with detached
receiver threads, global signal handlers or assumptions about `O_NONBLOCK`.

## Consequences

Each protected leaf incurs an additional owned process. This is preferable to
an uninterruptible receiver-thread join and does not grant another executable or
source authority. A cancelled or rejected leaf cannot promote output authority.
Host filesystem stalls and arbitrary blocking in the caller's observation
callback are outside this notification-receive cancellation mechanism.

Mantle bootstrap/protected-execution maintainers own this boundary. Repeatability
uses the real blocked-receive and pending-decision regressions plus the explicit
static-seed/bubblewrap smoke cases in `source_built_recipe_exec_tests.rs`.
Those cases retain actual allowed/denied notifications and output bytes. They do
not establish authenticated StageX/native/Rust command expansion, six-stage
bootstrap completion, compiler correctness or release eligibility. Execution and
final-source acceptance evidence remain required; this ADR is not that evidence.
