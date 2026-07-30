# ADR 0045: Adopt orphaned StageX exec descendants before inspection

## Status

Accepted (2026-07-30)

## Context

The StageX seccomp supervisor reads each `execve` or `execveat` path from the stopped tracee. Linux applies a ptrace access check to `process_vm_readv` and `/proc/<pid>/mem`.

Direct children and ordinary deep descendants passed this check. Some Bash and Make command shapes orphaned a child before its next `execve`. The kernel reparented that child outside the protected process tree. On hosts with Yama `ptrace_scope=1`, the supervisor then received `EPERM` and `EACCES` when it read the path.

Mantle correctly denied execution on that read failure. However, the orphan changed process ancestry without changing the declared executable authority.

## Decision Drivers

- Deny execution when the supervisor cannot read the tracee path.
- Preserve exact path-and-digest authorization.
- Keep deep shell and Make child processes inside the owned process tree.
- Do not require `CAP_SYS_PTRACE` or a weaker host Yama policy.
- Do not grant a global ptrace exception.
- Keep process adoption explicit and covered by positive and negative tests.

## Decision

Before Mantle installs the StageX seccomp listener, it sets the current process as a Linux child subreaper with `PR_SET_CHILD_SUBREAPER`.

If an intermediate shell exits before its child calls `execve`, the kernel reparents that child to the Mantle process. The seccomp supervisor can then apply the normal tracee-memory permission check while the child remains in the owned descendant tree.

Subreaper setup is fail-closed. Mantle does not install the seccomp filter if the `prctl` call fails.

The regression test creates an intermediate process, forks a delayed child, and lets the intermediate process exit. The parent waits for the adopted child and requires two allowed, exact exec audit events. Existing tests continue to require denial for unreadable paths, undeclared paths, relative paths, digest mismatches, and unsupported `execveat` forms.

## Alternatives Considered

### Allow execution when tracee memory is unreadable

Rejected because the supervisor would authorize an executable without knowing its path.

### Set `PR_SET_PTRACER` or `PR_SET_PTRACER_ANY`

Rejected because exec transitions clear this state and earlier diagnostic attempts did not close deep process trees.

### Require `CAP_SYS_PTRACE` or change `ptrace_scope`

Rejected because host-wide privilege or policy changes are outside the bounded StageX authority.

### Move the supervisor into a separate parent broker

Deferred because process adoption repairs the observed orphan boundary without a new cross-process policy protocol. A parent broker remains an option if a future executable clears dumpability or leaves the adopted tree.

## Consequences

- Orphaned shell and Make descendants remain observable by the StageX supervisor.
- Mantle must reap adopted descendants to avoid zombies.
- The process becomes a subreaper for later descendants in the same protected run.
- This decision does not weaken path, digest, source-stage, event-count, or fallback checks.
- This decision does not prove that the complete binutils transition is denial-free. That claim still requires a current full protected run.
