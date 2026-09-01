# V84 ptrace root-attach blocker

## Outcome

V84 ran the promoted proof from commit `28bfb932` with ptrace supervision,
strict hermeticity, no substitution, 16 jobs, and the unchanged 700 GB disk
bound.

The source profile was `Ready` with BLAKE3
`75ef2e49b42902d7350420b8a9cdfc611781c8f1deed1b1583cc75061852312f`.
The release binary had exact transfer parity and BLAKE3
`e5dd8dd2f65be9ca60ae592f7b78b207dcf78efa52f63d02d650fa24074968e8`.
The isolated V61 native provider revalidated with BLAKE3
`63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`.

The MRustC-to-Rust-1.90.0 stage completed successfully:

- 88,038 observed events;
- 88,038 matched events;
- zero denied events;
- zero technical failures;
- 177 promotions;
- reconciliation BLAKE3
  `920aeabc2f7efccacd05b7cc719e52a6256ccebabf0c01569469f9baf1171617`.

The proof then acquired and prepared Rust 1.91.1. Its root command failed
before stage execution with:

```text
PTRACE_SETOPTIONS: No such process (os error 3)
```

The wrapper stopped at `2026-08-30T01:06:06-04:00` with exit code 1. The
failed staging tree remains preserved at:

```text
/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v84-ptrace-supervised-20260829.source-built-fixed-point-staging-2155887
```

## Root cause

The first ptrace implementation let the child write its pid and immediately
raise `SIGSTOP`. The tracer could call `PTRACE_SEIZE` before or after that
stop. If the child entered a process group stop first, `waitpid` could return
a stop that was not ready for ptrace commands. The redundant
`PTRACE_SETOPTIONS` call then returned `ESRCH`.

`PTRACE_SEIZE` already applies its complete option mask. ADR 0087 replaces the
racy ordering with a per-root acknowledgment. The child now blocks after its
atomic `{pid, token}` record. It raises `SIGSTOP` only after the tracer has
seized it and sent the exact acknowledgment.

## Repair validation

The post-repair serialized checks passed:

- ptrace supervisor: 12 passed;
- Rust action shell: 2 passed;
- Rust provider action: 2 passed;
- Rust source provider: 121 passed.

The V84 source-built Rust 1.90 compiler also accepted the compatibility form
of `fetch_update` and the reason-bearing allow attribute. These checks test
the repair boundary. They do not promote V84 or replace a new proof run.

## Evidence

This directory preserves:

- detached launch, host, wrapper, transfer, profile, and proof records;
- the fixed-point and Rust-stage plans;
- the successful MRustC raw audit and reconciliation;
- compressed MRustC and failed Rust 1.91.1 logs;
- native-provider import and revalidation records;
- BLAKE3 identities for the preserved files.

## Non-claims

V84 does not prove Rust 1.91.1, the final Rust provider, checkpoint
publication, fixed-point equality, final receipt validity, or complete trust.
The completed MRustC stage remains failed-attempt evidence and is not a
promoted checkpoint.
