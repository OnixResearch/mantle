# Rust child-action authority core

## Goal

Plan every Rust compile and build-script action before execution. Bind direct and nested child executables to fixed source-built bytes or producer outputs.

Completion for this slice requires a deterministic plan, exact seccomp policy inputs, positive reconciliation, denied-child rejection, and no path-only authority.

## False completion

This slice does not claim complete root action trust. It does not claim that stage1 or stage2 currently run under the new policy. It does not make the historical V48 checkpoint eligible.

A generated path, ambient `/bin/sh`, a post-execution list, or a test-only seccomp event is not completion evidence.

## Search budget

This round used serial source inspection and one implementation path. It reused Mantle's existing protected-exec policy and seccomp supervisor. No subagent or remote proof ran.

## Mechanism registry

### Pure Rust action plan

- Mechanism: Convert normalized unit-graph producer edges into compile and build-script actions.
- Artifact: `src/source_built_rust_action_plan.rs`.
- Evidence: Fixed toolchain authority, producer-linked build-script authority, bounded resources, and deterministic digests have positive and negative tests.
- State: validated for the pure core.

### Existing seccomp supervisor

- Mechanism: Export exact fixed executable inputs into `ProtectedExecPolicy::from_action_plan`.
- Artifact: `src/protected_exec.rs` and `src/protected_exec_seccomp.rs`.
- Evidence: A subprocess fixture permits the exact digest and denies changed bytes before `execve`.
- State: validated as the runtime primitive.

### Ambient shell aliases

- Mechanism: Keep generated receipt-bound aliases on `#!/bin/sh`.
- Evidence: `cargo_free_self_build.rs` generated every alias and compatibility probe wrapper with ambient `/bin/sh`.
- Gap: weaker than the goal because a path does not bind interpreter bytes or a producer.
- State: falsified.

### Source-built BusyBox aliases

- Mechanism: Load BusyBox bytes and construction receipt from the full-source Rust binding. Generate aliases with its `sh` applet.
- Artifact: `src/cargo_free_self_build.rs`.
- Evidence: Positive tests bind the selected shell. Negative tests reject BusyBox or receipt digest drift.
- State: validated for cold-path alias generation.

## Adversarial audit

The planner rejects an unready graph, unknown or cyclic producers, non-BLAKE3 inputs, seed exceptions, relative executable paths, missing producer authority, duplicate identities, remote policy, cache-only completion, denied events, unknown events, missing actions, excess events, and digest drift.

The remaining production gap is explicit. The promoted checkpoint does not preserve and relocate the six Rust host-tool outputs and construction receipts. A restored Rust binding therefore retains origin-attempt paths. Production Rust topology execution also does not yet install this action policy or promote generated build-script executables.

## Validation

Pueue task `9854` wrote `focused-tests.log`. It records 8 Rust action-core tests, 2 protected-policy tests, 1 Linux seccomp subprocess test, and all 61 `cargo_free_self_build` tests passing. The shell tests ran with one test thread because this host can return transient `ETXTBSY` for parallel execution of newly written fixture scripts.

Pueue task `9858` wrote `focused-clippy.log`. It records successful focused first-party Clippy with warnings denied and only the repository's named baseline allowances.

## Decision

Keep the staged-plan route. Use the source-built BusyBox binding for aliases and compatibility probes. Do not authorize ambient `/bin/sh`.

Next, preserve and relocate Rust host-tool authority in checkpoint schema v2. Then install the action policy in a fresh Rust-topology worker and reconcile its events.
