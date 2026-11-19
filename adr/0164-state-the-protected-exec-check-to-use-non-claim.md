# 0164. State the protected-exec check-to-use non-claim

Status: Proposed

## Context

The protected-exec supervisor intercepts `execve` and `execveat` through a
seccomp user-notification listener. For each notification it reads the path
from tracee memory, opens `/proc/<pid>/root/<path>`, hashes that file, applies
the exact digest policy, and replies with `SECCOMP_USER_NOTIF_FLAG_CONTINUE`
when allowed.

On continue, the kernel re-reads the pathname from tracee memory and resolves it
again. Two windows remain:

- another thread sharing the tracee address space can rewrite the path buffer
  after the supervisor read it;
- a rename or symlink swap in a writable directory can re-point the path after
  hashing.

`SECCOMP_IOCTL_NOTIF_ID_VALID` confirms the notification is still pending; it
does not close either window. Linux's own `seccomp.h` and `seccomp_unotify(2)`
documentation warn that the notifier with continue cannot implement a security
policy in this shape.

## Decision Drivers

- Keep admission fail-closed and the existing exact-digest policy.
- Do not present audit observations as proof of executed bytes.
- Preserve a feasible path to a preventive mechanism later.
- Avoid weakening the supervisor while the stated limit is documented.

## Decision

The supervisor remains fail-closed on the paths it inspects, and this limit is
stated explicitly in the protected-exec documentation, audit consumers, and
this ADR. Audit events are observations of inspected paths under a
cooperative-tracee assumption, not proof of the executed bytes.

The current decision does not adopt post-exec `/proc/<pid>/exe` comparison as a
substitute for a preventive mechanism. A future change may add one or more of:

- restricting admitted exec targets to immutable, read-only mounts;
- enforcing execute rights with a kernel-evaluated mechanism such as Landlock;
- verifying `/proc/<pid>/exe` identity against the admitted digest after exec
  and killing on mismatch, classified as detective rather than preventive.

Any such change needs its own race fixtures and distinct audit classifications.

## Consequences

- Consumers must not derive an executed-bytes proof from a protected-exec audit
  event.
- The existing race is a known, documented limit rather than a silent claim.
- A multithreaded tracee or writable path swap can still execute bytes that
  differ from the inspected file; fail-closed admission does not close that
  window.
- The regression fixture and receipt non-claim enforcement remain follow-up
  work tracked by the issue.
