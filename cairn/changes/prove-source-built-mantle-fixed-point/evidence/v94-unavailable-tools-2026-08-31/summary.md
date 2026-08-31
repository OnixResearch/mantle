# V94 fixed-point guard-composition failure

## Verdict

V94 verified the refreshed profile and restored checkpoint. It then failed
before action-plan publication because fixed-point stage setup measured the new
unavailable-tool adapters before it wrote them.

This attempt does not prove stage1 execution, stage2, fixed-point equality, the
final receipt, or complete trust.

## Bound inputs

- Source commit: `3cccfc11d803039a006791c980be3679c5124ee3`
- Orchestrator BLAKE3:
  `b1804a2bafd3f475eaca2e1f3f81d1960f85626fb1e4aa06b1f03a31d1ddd22c`
- Ready source-profile BLAKE3:
  `069724a6124b67b491c7f29bfd450a041f8409a7b324c5aab04fc8937e866010`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 710,729,269,248

The source and binary transfers had exact checksum and round-trip parity. The
profile verified `Ready` with zero missing, stale, unsupported, or untrusted
records.

## Passed boundaries

V94 restored the immutable provider checkpoint, relocated all 17 closure
members, and validated the binding-owned rustc runtime. Its closure policy
identity was:

```text
e6c25624d6aadf93feca5a5bfcb142fe7f47dba0cfd57ba16495e94387537ec5
```

The run did not reach ptrace supervision or produce a Rust action plan.

## Root cause

V94 failed with:

```text
read unavailable Rust tool alias <cargo-guard-bin>/emcc:
No such file or directory
```

Ordinary cargo-free execution called the new adapter writer through
`prepare_execution_toolchain`. Fixed-point stages used a separate setup
sequence. That sequence wrote validated toolchain and host-tool aliases, then
constructed action authority without writing the unavailable-tool adapters.

The authority builder correctly rejected the missing file. It did not invent or
weaken authority.

## Decision

ADR 0097 makes `prepare_receipt_bound_guard_path` the only imperative-shell
composition function for guarded Rust executables.

Both ordinary and fixed-point routes now perform this order:

1. validated toolchain aliases;
2. source-built host-tool aliases;
3. bounded unavailable-tool adapters;
4. action-authority measurement.

Authority construction remains measurement-only. It does not publish missing
files.

## Validation

`post-repair-validation.log` records Rust 2024 formatting and all 66 cargo-free
self-build tests. The fixed-authority test now calls the shared composition
function. It verifies the one-entry `PATH`, all adapter identities, and
fail-closed modified-byte handling.

## Cleanup

- `cleanup-v93-before-v94.txt` records no-follow removal of the committed V93
  staging root.
- `cleanup-v93-profile-before-v94.txt` records removal of the superseded V93
  profile after V94 verified `Ready`.
- `cleanup-v91-profile-before-v94.txt` records removal of an older superseded
  profile to preserve the proof disk margin.

All three receipts state that regular-file modes were not changed.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Preserve V94 until the next proof no longer needs its setup-order diagnostics.
