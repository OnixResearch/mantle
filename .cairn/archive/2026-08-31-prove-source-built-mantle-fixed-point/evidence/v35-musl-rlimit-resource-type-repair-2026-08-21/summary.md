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
`local-validation.log`.

Remote pueue task `269` then used source commit `082bde0f`, release
orchestrator BLAKE3
`ec0f416ad1be0f467fd09dd39274619fe343dfab308137cda8a853214f2d572a`,
and the preserved V30 Rust provider. Both strict Cargo-free stages completed
789 units with no failed units. Their Mantle binaries were byte-identical and
had BLAKE3
`7ed29eac630d7d500ae04d06d0d3cd12064e938d990df47411f72b3213442fb3`.
The receipt reports strict proof admission and an enforced 17-member
source-built toolchain closure. See `preserved-provider-fixed-point-meta.json`,
`preserved-provider-fixed-point.log`, and `pueue-task-269.json`.

## Decision

The musl resource alias is accepted for the promoted proof source. No further
Cargo-free source portability blocker was observed.

## Non-claims

This diagnostic reused provider outputs and does not satisfy the promoted
proof. The type alias does not change the configured limits or add a practical
fallback.
