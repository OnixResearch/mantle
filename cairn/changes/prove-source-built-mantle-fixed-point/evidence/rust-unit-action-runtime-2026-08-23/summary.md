# Rust unit action runtime

## Goal

Run each Cargo-free Rust unit under its pre-execution action plan. Record every compiler, linker, build-script, shell, and helper execution through the Linux seccomp supervisor.

## Result

The Cargo-free stage shell now writes a stage-specific fixed-authority file before it launches `rust-plan`. That file binds the selected rustc, all executable native-provider artifacts, receipt-bound aliases, the source-built BusyBox shell, and all five source-built Rust host tools to BLAKE3 bytes and producer receipts.

`rust-plan` reads the bound rustc identity without executing rustc during planning. It derives and writes the complete unit action plan before it installs seccomp or starts a compiler.

The Rust topology execution port now:

- starts and ends one compile scope for each unit;
- promotes each generated build-script executable through its compile action and output identity before execution;
- assigns raw seccomp events to the active action;
- rejects overlapping scopes and unassigned events;
- writes raw audit and normalized reconciliation files;
- writes failure evidence before it returns an incomplete-reconciliation error.

Both fixed-point stages expose authority, plan, audit, and reconciliation paths in `meta.json`. The source-built proof shell requires typed, digest-valid, complete action evidence for both stages before it accepts binary equality.

## Negative controls

Tests reject fixed executable byte drift before filter installation, an unpaired action-authority CLI, use outside full topology execution, wrong requested rustc bytes, path-only authority, missing producer edges, denied events, unknown events, missing actions, digest drift, and audit tampering.

## Validation

Pueue task `9832` wrote the main focused log. Pueue task `9835` appended corrected exact-name CLI tests after the first two exact filters selected zero tests. Pueue task `9838` reran the core, port, native-artifact authority, and seccomp tests after the full native executable inventory was added.

`focused-tests.log` records:

- 10 action-plan, audit, and reconciliation tests passing;
- 2 Rust topology action-port tests passing;
- 2 Linux seccomp runtime tests passing;
- all 62 `cargo_free_self_build` tests passing with one test thread;
- both exact CLI/identity tests passing in the appended run.

Pueue task `9840` wrote the final `focused-clippy.log`. Focused first-party Clippy passed with warnings denied and only the repository's named baseline allowances.

## Remaining boundary

This slice covers stage1 and stage2 Rust units. It does not cover the earlier full-source Rust-provider scripts and their nested compilers. It also does not make the historical checkpoint eligible, because that checkpoint carries origin-path host-tool authority and no Rust-provider action reconciliation.

Do not launch a promoted proof yet. Next, derive staged provider child-action plans before each mrustc/rustc build, enforce them, and preserve their plans and reconciliations in checkpoint schema v2.
