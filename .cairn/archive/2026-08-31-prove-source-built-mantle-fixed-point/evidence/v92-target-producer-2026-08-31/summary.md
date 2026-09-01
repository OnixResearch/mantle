# V92 rustc linker-probe failure

## Verdict

V92 passed checkpoint, closure, runtime, source-identity, and graph-producer
boundaries. It created an 862-action Rust plan and began protected stage1
execution. The ptrace supervisor then failed on a nonexistent rustc linker
candidate before the kernel could return `ENOENT`.

This attempt does not prove stage1 completion, stage2, fixed-point equality, the
final receipt, or complete trust.

## Bound inputs

- Source commit: `bdb1d5653aa9390964883cf28692424a464536ef`
- Orchestrator BLAKE3:
  `74a417a886acb97f2c0f1d168fc753052edf6de358a8ffab614a27d8c2386256`
- Ready source-profile BLAKE3:
  `e7a165e196f7293e9b11a2cb7ecfdf567819b2296f68e6cbe20bf353b6d44b5a`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 705,473,802,240

## Passed boundaries

V92 restored the immutable checkpoint without rebuilding providers. Closure and
rustc runtime checks passed.

Stage1 accepted all typed source identities, consumed-host proc-macro edges,
and 2,203 target dependency producer fallbacks. It wrote complete Rust action
authority and a plan with 862 actions.

## Root cause

During the first protected rustc link, the tracee attempted:

```text
<rust-provider>/lib/rustlib/x86_64-unknown-linux-musl/bin/cc
```

That path does not exist. The ptrace supervisor could not open it and failed
closed, as designed.

`linker-execvp-strace.log.gz` records the same invocation without supervision.
Normal `execvp` first tried the missing sysroot candidate, received `ENOENT`,
then executed the receipt-bound stage `cc` and completed successfully.

Weakening ptrace to ignore missing paths would violate its fail-closed path
observation rule.

## Decision

ADR 0095 generates the rustc runtime wrapper only after receipt-bound aliases
exist. It appends the absolute alias as the final rustc option:

```text
-C linker=<receipt-bound cc>
```

The generated wrapper's exact linker alias enters fixed Rust action authority as
a C compiler. Zero or multiple wrapper linker aliases fail.

Ptrace handling remains unchanged. No missing path, relative executable, unreadable
path, or hash failure receives execution authority.

`absolute-linker-strace-status.txt` records the positive diagnostic: status 0,
zero missing sysroot-linker attempts, two bound-linker exec observations, and a
produced executable.

## Validation

`post-repair-validation.log` records the wrapper/linker positive and negative
tests, complete cargo-free self-build module tests, and Rust formatting.

## Preserved evidence

This directory contains the exact launch records, full proof log, failed status,
checkpoint and closure reports, compatibility wrapper, 862-action plan and
authority, stage1 stderr, original and normalized linker straces, operator
scripts, and validation evidence.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Preserve V92 until the new proof no longer needs its protected execution
diagnostics.
