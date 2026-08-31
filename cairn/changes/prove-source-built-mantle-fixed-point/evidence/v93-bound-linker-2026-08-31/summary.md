# V93 unavailable optional-tool failure

## Verdict

V93 verified the ADR 0095 linker normalization. Protected stage1 did not try
the absent Rust-sysroot `cc`. It then failed closed on a different expected
negative probe: optional `emcc` discovery in the one-entry guarded `PATH`.

This attempt does not prove stage1 completion, stage2, fixed-point equality, the
final receipt, or complete trust.

## Bound inputs

- Source commit: `2e67a21ec7d574e82f876be171f93c1f5efb43a5`
- Orchestrator BLAKE3:
  `a0d99154bf770694a71200a13db2b4b7dab1693412beb0368186bb02e0391fb3`
- Ready source-profile BLAKE3:
  `8923006e7b3a41b68911eb5ff1493a28acdf5e8fa90477fb79b0e55e23a2af9e`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 702,908,284,928

The source and binary transfers had exact checksum and round-trip parity. The
profile was refreshed from V92 and verified `Ready` with zero missing, stale,
unsupported, or untrusted records.

## Passed boundaries

V93 restored the immutable provider checkpoint, relocated all 17 closure
members, validated the bound rustc runtime, created an 862-action stage1 plan,
and began protected execution.

The generated rustc wrapper selected this absolute linker:

```text
<cargo-free-fixed-point>/toolchain/receipt-bound-path/cc
```

The V92 missing sysroot linker candidate did not recur.

## Root cause

Stage1 failed with:

```text
ptrace tracer loop failed: open tracee exec path
<cargo-guard-bin>/emcc: No such file or directory
```

`unavailable-emcc-observation.txt` proves that `emcc` was absent and was not a
symlink. The same file binds the complete guarded tool list and BLAKE3 values.

This is optional compiler discovery. Earlier source-built Rust-bootstrap
evidence already identified the closed companion family: `emcc`, `pkg-config`,
`pkgconf`, and `git`.

Ptrace correctly failed closed. It must not ignore the missing path.

## Decision

ADR 0096 adds exact exit-127 policy adapters for that closed probe family.
Mantle writes an adapter only when no validated toolchain or host-tool binding
owns the alias.

Each adapter uses the source-built BusyBox shell. Its exact path and BLAKE3
identity enter fixed Rust action authority as a `CompilerPolicyAdapter`.
Authority construction rejects modified adapter bytes.

The adapters do not provide compiler, package metadata, network, or Git
capability.

## Validation

`post-repair-validation.log` records Rust 2024 formatting and all 66 cargo-free
self-build tests. The tests include these positive and negative cases:

- all unavailable adapters enter fixed action authority;
- `emcc` exits with status 127;
- a declared `git` alias is not replaced;
- modified adapter bytes fail authority construction.

## Cleanup

`cleanup-v92-before-v93.txt` records no-follow removal of the preserved V92
staging root after repository evidence commit `2e67a21e`.

`cleanup-v92-profile-before-v93.txt` records regular-file removal of the V92
profile only after the V93 replacement profile verified `Ready`. Neither
cleanup changed regular-file modes.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and run a fresh promoted proof.
Preserve V93 until the next proof no longer needs its action diagnostics.
