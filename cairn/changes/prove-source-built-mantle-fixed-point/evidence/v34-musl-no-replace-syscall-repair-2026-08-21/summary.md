# V34 musl no-replace syscall repair

## Goal

Continue the preserved-provider Cargo-free diagnostic after the optional
feature-edge repair.

## Starting evidence

Remote pueue task `267` used source commit `1c2e1e57` and release orchestrator
BLAKE3 `f0cae9784bca97267bd38e40559b3b88f75d97fc3abca5385e5242e8a5bd48ca`.
The source transfer had root-anchored exclusions and exact checksum parity. The
diagnostic produced 736 stage1 unit receipts, which moved 28 units beyond the
prior blocker.

The first failed unit was `crunch-rustc-wrapper` for
`x86_64-unknown-linux-musl`:

```text
error[E0425]: cannot find function `renameat2` in crate `libc`
```

The `libc` crate exposes the `renameat2` function on glibc Linux targets. It
does not expose that function on musl, although it exposes the Linux
`SYS_renameat2` number and `RENAME_NOREPLACE` flag.

## Repair

The wrapper now calls the Linux kernel ABI through:

```text
libc::syscall(libc::SYS_renameat2, ..., libc::RENAME_NOREPLACE)
```

This matches Mantle's existing `src/linux_rename.rs` boundary. The wrapper
keeps its existing multi-output rollback policy and does not transfer that
product authority to another component.

## Validation

The pre-change no-clobber, rollback, and concurrent race tests passed in pueue
task `6735`.

Pueue task `6758` recorded `local-validation.log`. All 20 wrapper library tests
passed, including positive publication, stale-destination rejection,
transaction rollback, and concurrent same-destination behavior. Strict package
Clippy, changed-file formatting, and `git diff --check` passed.

The local Nix Rust toolchain reports a musl target directory but does not contain
its `core` or `std` libraries. The direct local target check therefore stopped
before this crate. See `local-musl-check.log`. The source-built Rust provider on
Leviathan must supply the musl compile evidence.

## Non-claims

This diagnostic reuses provider outputs and cannot satisfy the promoted proof.
The syscall change does not weaken no-replace semantics, add a fallback rename,
or claim filesystem durability.
