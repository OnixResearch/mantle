# V35 musl rlimit resource-type repair

## Goal

Continue the preserved-provider diagnostic after the no-replace syscall repair.

## Starting evidence

Remote pueue task `268` used source commit `bcc0666b` and release orchestrator
BLAKE3 `32cfbabd08bec5228a3932b756e62fb16db4589d9c3521338a4fb6d33d4bef8e`.
It produced 770 stage1 unit receipts and reached the final Mantle binary.

The first failure was:

```text
error[E0425]: cannot find type `__rlimit_resource_t` in crate `libc`
  --> src/evaluator_budget.rs:652:39
```

The libc crate uses `__rlimit_resource_t` for GNU Linux. Its musl `setrlimit`
signature uses `c_int`.

## Repair

Mantle now defines one `LinuxRlimitResource` alias per supported Linux libc
environment:

- GNU: `libc::__rlimit_resource_t`;
- musl: `libc::c_int`.

The limit logic, values, child pre-exec hook, and error handling remain
unchanged.

## Validation

The host target type test, focused strict Clippy, changed-file formatting, and
`git diff --check` passed in pueue tasks `6970` and `6985`. See
`local-validation.log`. The source-built Rust provider must supply the musl
compile evidence.

## Non-claims

This diagnostic reuses provider outputs and cannot satisfy the promoted proof.
The type alias does not change the configured limits or add a practical
fallback.
