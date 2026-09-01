# V83 diagnostic: Rust 1.93.1 stage without seccomp supervision

## Purpose

Discriminate the V81/V82 Rust 1.93.1 failure. Both supervised attempts died with `EACCES` on bound executables at the same build step, with clean audit authority and zero response failures.

## Method

The exact V82 `rust-1.93.1-stage1` chain directory (patched sources and generated script) was copied to a sibling diagnostic root. Embedded absolute paths were rewritten to the copy only; the bootstrap-provider path still binds the preserved 1.92.0 candidate. The stage then ran standalone under `strace -f -qq -e trace=execve` with the seccomp supervisor disabled.

This is a diagnostic control. It is not proof evidence and grants no construction authority.

## Result

The stage completed end to end:

- exit code 0, `Build completed successfully in 2:51:51`
- `rustc stage1 products ready`
- 75,268 traced `execve` calls
- zero `EACCES` anywhere in the trace
- the previously failing LLVM target cluster (`llvm-pdbutil`, `DebugInfo/GSYM`, `ExecutionEngine/JITLink`, `llvm-readobj`) built cleanly

The supervised runs died at the same step after only 22,192 supervised events, in both V81 and V82.

## Finding

The `EACCES` failures are caused by the seccomp user-notification supervision of `execve` itself, not by the build, the filesystem, file modes, or the supervisor's classification logic.

Consistent observations:

- failures occur only under supervision, deterministically at the same build step;
- the audit records the same boundaries as allowed with correct digests and accepted kernel responses;
- victim binaries vary between runs (`cmake`, `sh`, `as`);
- an unsupervised run of the identical work passes with zero errors;
- a 24,000-execution stress test under the supervisor passed, so the boundary is not a plain notification count.

The evidence points at the kernel's `SECCOMP_USER_NOTIF_FLAG_CONTINUE` path for `execve` under sustained parallel load from this process tree, interacting with the supervisor's single-threaded receive/classify/send loop. The known kernel TOCTOU hazard for continuing `execve` through user notification is the prime suspect; exact kernel internals were not proven here.

## Recorded artifacts

- `diag-status.txt`: timing, pid, and exit status
- `diag-run.log`: the full stage build log
- `execve.strace.gz`: the complete compressed execve trace
- `v83-diag-summary.txt`: status, tail, exec count, EACCES count
- `run-v82-diag-193-noseccomp-strace.sh`: the exact diagnostic wrapper

## Non-claims

This diagnostic does not prove the Rust provider, compiler correctness, checkpoint eligibility, fixed-point equality, or any release readiness. It only establishes where the failure lives.
