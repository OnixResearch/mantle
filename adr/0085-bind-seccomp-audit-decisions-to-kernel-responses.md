# ADR 0085: Bind seccomp audit decisions to kernel responses

## Status

Accepted (2026-08-29)

## Context

The protected execution supervisor classifies each `execve` or `execveat` request and then sends a Linux seccomp user-notification response.

The old sequence added the policy decision to the audit before it sent the response. It ignored response errors. V81 recorded bound CMake and assembler requests as allowed, but the calling processes received `EACCES`.

An allowed policy classification is not an allowed execution unless the kernel accepts the continue response.

## Decision Drivers

- Keep exact path and BLAKE3 policy unchanged.
- Keep failures fail-closed.
- Do not report an execution as allowed before the kernel accepts the response.
- Handle interruptible ioctls with a fixed retry bound.
- Do not retain promotions from a response that did not complete.
- Preserve an exact response error for later reconciliation.

## Decision

The supervisor sends the seccomp response before it commits the audit event or automatic promotion.

`SECCOMP_IOCTL_NOTIF_SEND` retries only `EINTR`. The retry count is fixed and checked. Other errors fail immediately.

If the response succeeds, the supervisor keeps the classified audit decision and any promotion.

If the response fails, the supervisor changes the audit decision to denied, records the exact ioctl error, and discards the notification’s promotion. Stage reconciliation therefore cannot report a complete action trace after a response failure.

Concurrent positive coverage executes one declared program 512 times from four worker threads. Negative coverage proves that a failed response cannot remain allowed or retain a promotion.

## Alternatives Considered

### Ignore response errors

Rejected. This produced allowed audit records for executions that received `EACCES`.

### Retry all response errors

Rejected. Errors other than `EINTR` can identify invalid notification state or a broken listener. Retrying them can hide a real failure.

### Reduce Rust build concurrency

Rejected. This would avoid pressure instead of repairing the supervisor boundary. It would also change the reviewed build schedule without evidence that concurrency is invalid.

### Keep promotions after response failure

Rejected. The executable did not gain successful execution evidence from that notification.

## Consequences

- Interruptible response sends get a bounded retry.
- Audit and promotion evidence follow the kernel response result.
- A response failure becomes an explicit reconciliation blocker.
- Protected execution policy and resource bounds do not expand.
- This decision does not prove child correctness, compiler correctness, or fixed-point completion.
