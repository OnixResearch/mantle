# 0115: Keep the CLI root mechanical

- Status: Accepted
- Date: 2026-09-03

## Context

`src/main.rs` mixed Clap records with application policy and effect execution. It also contained store, remote, source, release, project, and bootstrap operations.

This structure hid dependency direction. Application ports could depend on `RunError` and provider types. It was also hard to show where a command became an effect.

The store, remote, Rust-plan, and build-planning boundaries now provide narrower capabilities. The CLI can compose these boundaries without owning their policy.

## Decision

Keep `src/main.rs` as a mechanical composition root. It can parse arguments, initialize tracing, call one application entry, present the result, and select an exit status.

Keep Clap records in `src/cli_inbound.rs`. Map them to `mantle-application-core::ApplicationCommand` in the inbound adapter.

Use `mantle-application-core` for deterministic command admission, BLAKE3-bound effect identity, limits, and observation classification. The crate is `no_std + alloc`.

Use `mantle-application` for application-owned family ports and effect dispatch. Its contracts contain only Mantle-owned types. The crate is also `no_std + alloc`.

Keep host and provider execution in `CliOperationAdapter`. The adapter delegates to the existing command-family shell while the command modules remain compatible.

Convert legacy `RunError` values to typed capability failures at this adapter. Convert typed outcomes back to stable CLI errors only in the presentation adapter.

Keep these hidden worker commands as explicit shell exceptions:

- `__evaluator-worker`;
- `__evaluator-worker-fixture`;
- `__remote-secret-worker`.

Maintain one deterministic architecture checker. It checks root size, allowed root functions, core purity, port ownership, adapter direction, error ownership, presentation separation, and declared Nix targets.

## Consequences

The root has one visible application entry. Command families pass through typed effects and typed observations before terminal reporting.

Accepted command names, options, output bytes, diagnostics, and exit codes stay unchanged. A 12-case parity corpus checks this compatibility.

The compatibility shell remains large, but it no longer lives in `main.rs`. New policy must go into an owning core, application operation, or adapter.

Dispatch does not prove that an effect succeeded. A port call does not prove provider correctness, deployment success, or release eligibility.
