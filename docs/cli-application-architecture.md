# CLI application architecture

Mantle keeps its CLI entry point as a mechanical composition root. The root does not own build, store, trust, retry, remote, source, or release policy.

## Command flow

```text
argv
  -> Clap DTO in src/cli_inbound.rs
  -> typed inbound adapter
  -> mantle-application-core effect plan
  -> mantle-application family port
  -> CLI operation adapter
  -> typed observation and outcome
  -> CLI presentation adapter
  -> stable output and exit status
```

`src/main.rs` has these duties:

1. Parse CLI input.
2. Reject conflicting machine-output modes through the inbound module.
3. Initialize requested tracing.
4. Call the application entry.
5. Render the final error.
6. Select the exit status.

It does not traverse files, start processes, contact networks, open stores, or construct receipts.

## Owned components

| Component | Owner | Purpose |
|---|---|---|
| `src/cli_inbound.rs` | CLI inbound | Clap records, syntax defaults, and DTO validation |
| `src/cli_architecture/inbound_adapter.rs` | CLI inbound adapter | Typed family, operation, request identity, and mutation mapping |
| `crates/mantle-application-core` | Functional core | Deterministic effect planning and observation classification |
| `crates/mantle-application` | Application shell contract | Family ports and typed effect dispatch |
| `src/cli_architecture/operation_adapter.rs` | CLI operation adapter | Host execution and legacy command-family delegation |
| `src/cli_architecture/presentation_adapter.rs` | CLI presentation adapter | Stable `RunError`, human, JSON, and exit-code mapping |
| `src/cli_application.rs` | Compatibility application shell | Runtime context, adapter selection, and command-family operations |

The application layer owns 12 narrow family ports. These ports cover build, Rust-plan, remote, store, source, release, project, bootstrap, artifact, evaluation, package, and utility operations.

## Error ownership

The functional core returns `ApplicationCoreError`. Application ports return `ApplicationPortError`. Both errors use Mantle-owned types.

The operation adapter preserves the family, effect identity, error class, code, message, and optional reported exit code. The presentation adapter is the only new boundary that reconstructs `RunError`.

Provider details remain in outer adapters. Provider types do not enter core or port contracts.

## Effect and observation boundary

`plan_dispatch` creates a bounded `ApplicationEffect`. Its BLAKE3 identity binds the normalized command, request identity, mutation class, and limits.

A port executes the effect and returns an `ApplicationObservation`. The core rejects a wrong effect identity, wrong family, malformed success, malformed failure, or invalid reported exit code.

A plan is not an execution claim. A successful outcome requires a matching typed observation.

## Hidden workers

Three hidden commands run before normal family dispatch:

- `__evaluator-worker`;
- `__evaluator-worker-fixture`;
- `__remote-secret-worker`.

These commands are explicit shell adapters. They are not alternate application-policy paths.

## Architecture checks

Run the maintained source rail:

```sh
cargo -Zscript scripts/check-cli-application-architecture.rs --root .
```

The rail checks:

- the root line limit and allowed root functions;
- `no_std + alloc` declarations;
- core and application dependencies;
- all family ports and adapter implementations;
- inbound, operation, and presentation separation;
- typed error ownership;
- forbidden infrastructure imports;
- positive and adversarial fixtures;
- host and `wasm32-unknown-unknown` Nix checks.

Validate the Nickel architecture contract with:

```sh
nickel typecheck config/cli-application-architecture.ncl
```

The rail reports source topology. It does not prove command semantics, provider correctness, effect success, deployment success, or release eligibility.
