# ADR 0086: Supervise exec with ptrace stops instead of user-notify continue

## Status

Accepted (2026-08-29)

## Context

The protected execution supervisor supervises `execve` and `execveat` with a seccomp filter that returns `SECCOMP_RET_USER_NOTIF`. One listener thread receives each notification, resolves and hashes the target, classifies against the exact policy, and answers with `SECCOMP_USER_NOTIF_FLAG_CONTINUE` or `EACCES`.

V81 and V82 proved this mechanism unsafe for this workload. Both Rust 1.93.1 stage attempts failed deterministically at the same LLVM build step with `EACCES` on correctly bound executables, while the audit recorded those boundaries as allowed with accepted kernel responses. The same stage with the same sources, run on the same machine with the supervisor disabled, completed all 75,268 execs with zero errors. A 24,000-execution stress test under the supervisor passed, so the trigger is not a plain notification count. The kernel's user-notification continue path for `execve` under sustained parallel load from deep compiler process trees is the failing component.

The supervisor's own decisions, audit binding, and response handling were exonerated: audits and kernel responses agree, and the audit layer now fails closed on response errors (ADR 0085).

## Decision Drivers

- Keep exact byte binding, deny-before-exec, fail-closed errors, promotions, and the existing action-plan and reconciliation semantics.
- Keep supervision invisible to build correctness: supervised and unsupervised runs of the same work must both succeed.
- Do not use `SECCOMP_USER_NOTIF_FLAG_CONTINUE` for `execve` again.
- Keep one tracer thread per supervised stage and bounded resources.
- Preserve the existing `ProtectedExecPolicy`, audit-event schema, evidence files, and reconciliation inputs so stage plans and proofs remain comparable.

## Decision

Replace the notification mechanism with `SECCOMP_RET_TRACE` plus ptrace supervision.

The supervised stage child installs, in its `pre_exec` hook: `PR_SET_NO_NEW_PRIVS`, a seccomp filter that returns `SECCOMP_RET_TRACE` for `execve` and `execveat` (all else allow), and then raises `SIGSTOP`. The child writes its pid through a pre-opened pipe.

The supervisor thread reads the pid from the pipe, attaches with `PTRACE_SEIZE`, sets `PTRACE_O_TRACEFORK | PTRACE_O_TRACECLONE | PTRACE_O_TRACEVFORK | PTRACE_O_TRACESYSGOOD`, and resumes it. Because attaching reparents the tracee to the tracer for wait purposes, and because the attach options auto-attach every descendant to the same tracer, one tracer thread observes the entire process tree through `waitpid`.

On each `PTRACE_EVENT_SECCOMP` syscall-entry stop, the tracer:

1. reads the syscall number and arguments from registers;
2. reads the target path from tracee memory with the existing bounded reader;
3. resolves the host path through `/proc/<pid>/root` and hashes the exact bytes;
4. classifies against the existing `ProtectedExecPolicy` (allow, deny, or promote);
5. allows by resuming, or denies by rewriting registers so the syscall returns `EACCES` without executing.

Because the decision happens at a syscall-entry stop with the syscall not yet executed, denial is true deny-before-exec with no continue path and no continue TOCTOU.

Signal-delivery stops (dominated by `SIGCHLD` in parallel builds) are forwarded unchanged. Genuine exits are reaped and counted. `PR_SET_CHILD_SUBREAPER` stays set on the tracer so orphaned descendants remain observable; the tracer loop absorbs them.

The existing response-failure binding from ADR 0085 becomes a tracer-failure binding: any waitpid, register, memory-read, or hash error is recorded as a denied audit event and drops promotions, keeping every stage reconciliation fail closed.

## Alternatives Considered

### Keep user-notify and lower build parallelism

Rejected. It avoids the trigger by luck of scheduling rather than by design, and it slows every future stage.

### Bound-launcher: only a receipt-bound launcher may be exec'd, and it self-verifies then fexecve's a held file descriptor

Rejected. GCC, CMake, Python, and x.py exec subprograms internally (`cc1`, `as`, `collect2`, compiler detection probes); those internal exec chains cannot be routed through a launcher.

### Restrict to `SECCOMP_RET_ERRNO` with no supervision of allowed execs

Rejected. Deny-before-exec for unexpected targets requires observing every exec; pre-approved allowlisting without observation loses the promoted-output and audit guarantees.

### eBPF LSM (fexit on `bprm_execve`)

Rejected for now: requires a loaded BPF program with root and CAP_BPF, a new kernel dependency for the proof, and a different evidence model. Revisit only if ptrace supervision also proves unreliable.

## Consequences

- ptrace stops and signal forwarding add bounded overhead per exec and per signal; heavy parallel builds pay more than before but remain practical (the unsupervised diagnostic completed the same stage in under three hours even under strace).
- The tracer is a single thread per supervised stage; tracer failure fails the stage closed.
- `PR_SET_CHILD_SUBREAPER` and the tracer replace the listener fd; the ADR 0053 fresh-worker-lineage rule still applies, with the tracer thread as the stage's protected worker.
- Existing stage plans, policies, audit schemas, reconciliations, and checkpoint payloads are unchanged, so completed stage evidence stays comparable across the mechanism switch.
- This decision does not itself prove any construction; supervised reruns must re-establish the Rust provider evidence from the first affected stage.
