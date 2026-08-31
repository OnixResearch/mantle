# V95 GCC subprogram-path failure

## Verdict

V95 verified the shared guard composition. It wrote and bound all unavailable
policy adapters, published a complete stage1 action plan, and began protected
execution. GCC then generated a `cc1` exec path with a lexical parent component.
Ptrace rejected that path before execution.

This attempt does not prove stage1 completion, stage2, fixed-point equality, the
final receipt, or complete trust.

## Bound inputs

- Source commit: `433b9fb3263538116665fb2391ee9507eceee2f4`
- Orchestrator BLAKE3:
  `daab7af7f186e48b2cd5da58ac77575cd2bc3b18d580af5b4e5e89987e2f246c`
- Ready source-profile BLAKE3:
  `d1853c44dabafc474686632d300ed7b79e2c9f670b4f21ebbdf0976428c002b3`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 708,281,581,568

The source and binary transfers had exact checksum and round-trip parity. The
profile verified `Ready` with zero missing, stale, unsupported, or untrusted
records.

## Passed boundaries

V95 restored the immutable provider checkpoint, relocated all 17 closure
members, and validated the binding-owned rustc runtime.

The shared guard setup wrote validated toolchain aliases, source-built host-tool
aliases, and all four unavailable-tool policy adapters before authority
measurement. Stage1 published complete action authority and a Rust action plan.
The V94 missing-adapter failure did not recur.

## Root cause

Protected execution failed on this path:

```text
<native-provider>/bin/../libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1
```

The native binding already authorized the target `cc1` bytes, but ptrace rejects
parent components. This lexical rejection prevents alternate spellings from
bypassing exact path identity and must remain unchanged.

GCC derived the path from its driver location. The failure was not missing
compiler authority.

## Decision

ADR 0098 derives one canonical GCC subprogram directory from the bound `cc1`,
`cc1plus`, `collect2`, and `lto-wrapper` artifacts.

Mantle now requires all four artifacts, recomputes each BLAKE3 digest, and
requires one shared parent. Guarded C, C++, and preprocessor driver aliases
append this final option:

```text
-B<canonical compiler-subprogram directory>/
```

Ptrace parent-component and relative-path rejection remain unchanged. Mantle
does not use ambient `GCC_EXEC_PREFIX` or `COMPILER_PATH`.

## Diagnostic evidence

`gcc-subprogram-prefix-strace.log.gz` records an unsupervised diagnostic using
the final `-B` position produced by the repaired alias.

`gcc-subprogram-prefix-status.txt` records:

- status 0;
- zero parent-component `cc1` exec paths;
- one canonical `cc1` exec path;
- one produced object.

This diagnostic proves the narrow path transformation only. It is not
fixed-point evidence.

## Validation

`post-repair-validation.log` records Rust 2024 formatting and all 67 cargo-free
self-build tests. Positive and negative coverage requires:

- one shared canonical subprogram directory;
- bound C and C++ driver identities;
- a final normalized `-B` argument;
- rejection of a missing internal executable;
- rejection of a compiler-internal digest mismatch.

## Cleanup

- `cleanup-v94-before-v95.txt` records no-follow removal of the committed V94
  staging root.
- `cleanup-v94-profile-before-v95.txt` records removal of the superseded V94
  profile after V95 verified `Ready`.

Neither cleanup changed regular-file modes.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Preserve V95 until the next proof no longer needs its GCC diagnostics.
