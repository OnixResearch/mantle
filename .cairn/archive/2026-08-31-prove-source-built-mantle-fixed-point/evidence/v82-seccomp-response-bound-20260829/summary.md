# V82 seccomp response-bound rerun blocker

## Outcome

V82 ran with the response-bound supervisor from commit `0d4211e5`. It repeated V81’s stage history exactly and stopped in Rust 1.93.1 at the same boundary.

- MRustC to Rust 1.90.0: 88,038 matched, zero denied.
- Rust 1.91.1: 75,843 matched, zero denied.
- Rust 1.92.0: 76,071 matched, zero denied.
- Rust 1.93.1: 22,192 matched, zero denied, then `stage-execution-failed`.

The response binding worked: the audit has zero `seccomp notification response failed` records and zero denied records. The build log again shows `Permission denied` for `execv` of the bound `as` and `sh`/`cmake` binaries inside the same LLVM target cluster.

## New discriminating facts

1. The failure is deterministic. V81 and V82 both stopped after exactly 22,192 audit events, at the same build step (same log position, same LLVM target cluster: `llvm-pdbutil`, `DebugInfo/GSYM`, `ExecutionEngine/JITLink`, `llvm-readobj`).
2. The victim binary differs between runs (`cmake` in V81, `sh` in V82), so it is not a property of one executable.
3. Earlier stages built the same 133 LLVM target references successfully under the same supervisor with far more audit events, so the targets and a global event cap are not the trigger.
4. A scaled stress test ran 24,000 supervised executions across four workers in one supervisor. All passed. A bare notification-count boundary is ruled out.
5. After the failed stage, the accused binaries remained mode `0555`, owned by the build user, and executed successfully outside the supervisor.

## Interpretation

The supervisor classifies and answers correctly. The EACCES arises outside the classification decision: either the kernel refuses the real exec after `SECCOMP_USER_NOTIF_FLAG_CONTINUE`, or the failing execs stop producing notifications for a reason not yet observed. Both are consistent with current records; the tracee-side errno timing is the missing observation.

## Next diagnostic

Rerun only the Rust 1.93.1 stage from the preserved staging tree as a diagnostic (not proof) with `strace -f -e trace=execve` attached to the build. This records the exact errno of every failing exec and whether a corresponding notification exists, separating kernel post-continue denial from lost notifications.

## Evidence

The directory preserves plans, reconciliations, compressed raw audits for all four stages, compressed build logs, native-prefix materialization records, launch and status records, and the wrapper status.

## Non-claims

V82 does not prove the Rust provider, checkpoint publication, fixed-point equality, or complete action trust.
