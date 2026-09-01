# V97 musl ptrace request-type failure

## Verdict

V97 applied the executed-topology action scope and completed 842 protected
actions with 5,208 matched events. It then stopped while the restored musl Rust
toolchain compiled Mantle's ptrace supervisor.

The first causal compiler error was a libc request-parameter type mismatch. The
remaining 20 planned actions formed the unexecuted topology suffix.

This attempt does not prove accepted stage1, stage2, fixed-point equality, the
final receipt, or complete trust.

## Bound inputs

- Source commit: `506e4e0510208c725517ccbb4b489aa05b0b307d`
- Orchestrator BLAKE3:
  `d7e1891a2d61cc6c131c58f5487b837b3400ec20514e135090c1d13b9663ac58`
- Ready source-profile BLAKE3:
  `838e3dff85b1990dd370d6b477c804f122155d1dd1711722a11473c3f2262cc1`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 713,467,973,632

The source and binary transfers had exact checksum and round-trip parity. The
profile verified `Ready` with zero missing, stale, unsupported, or untrusted
records.

## Passed boundaries

V97 restored the immutable provider checkpoint, relocated all 17 closure
members, validated the bound rustc runtime, and retained canonical GCC internal
paths.

The action scope matched the combined topology. The audit and reconciliation
record:

- planned actions: 862;
- matched actions: 842;
- observed events: 5,208;
- matched events: 5,208;
- unknown events: 0;
- denied events: 0;
- drifted events: 0;
- overbound actions: 0;
- missing suffix actions: 20.

`missing-actions-in-topology-order.tsv` proves that the missing actions are the
contiguous order suffix at indices 769 through 788. The first suffix unit is the
target `petgraph 0.7.1`; it is not the causal compiler unit.

## Root cause

The command returned only the reconciliation suffix, which hid the earlier
blocked topology receipt. Bounded replay diagnostics retained both failures and
exposed this compiler blocker:

```text
rustc-failed: error[E0308]: mismatched types
src/protected_exec_ptrace.rs:854
expected i32, found u32
```

The vendored musl libc declares:

```text
ptrace(request: c_int, ...)
```

The host GNU libc declaration accepts an unsigned request. Mantle's Linux
request constants are `c_uint`, so host compilation passed while restored musl
compilation failed.

The replay binaries and forced-reuse diagnostics are diagnostic only. They do
not replace the original protected audit.

## Decision

ADR 0100 passes every direct ptrace request as `REQUEST as _`. Rust infers the
request parameter from the selected libc declaration. Request values, registers,
policy, and deny-before-exec behavior remain unchanged.

When topology is blocked and reconciliation also fails, the command now returns
the topology blocker before the reconciliation detail.

## Restored-toolchain type evidence

`ptrace-musl-typecheck.txt` was produced with V97's restored rustc runtime. It
records:

- cast fixture status: 0;
- uncast fixture status: 1;
- uncast diagnostic: `E0308`;
- expected `i32`, found `u32`;
- positive metadata artifact present.

The positive and negative fixture sources are preserved beside the transcript.

## Validation

`post-repair-validation.log` records Rust 2024 formatting and these passing
suites with one test thread where supervision is involved:

- 12 ptrace supervisor tests;
- 1 topology-blocker reporting test;
- 9 combined-topology tests;
- 14 Rust action-plan tests;
- 2 Rust action-shell tests;
- 67 cargo-free self-build tests.

## Cleanup

- `cleanup-v96-before-v97.txt` records no-follow removal of the committed V96
  staging root.
- `cleanup-v96-profile-before-v97.txt` records removal of the superseded V96
  profile after V97 verified `Ready`.
- `cleanup-v90-profile-before-v97.txt` records removal of an older superseded
  profile to preserve proof disk margin.

No cleanup changed regular-file modes.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Preserve V97 until the next proof validates musl compilation and stage
progression.
