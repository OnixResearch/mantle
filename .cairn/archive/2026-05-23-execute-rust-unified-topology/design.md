# Design: unified Rust topology execution

## Approach

Add a narrow `execute_rust_unit_topology` function that generalizes the current target-only and host-artifact topology rails:

1. Validate `unit_derivation_graph.ready`.
2. Select all supported host units (`custom-build`, `proc-macro`) and all supported target units (`lib`, `bin`).
3. Ensure consumed host artifacts have matching host producers.
4. Build dependency edges among target units from `dependency_artifacts`, ignoring dependencies satisfied by host producers.
5. Topologically execute target units after host units.
6. Bind produced host artifacts, build-script metadata, and produced target library artifacts into each target unit before rustc invocation.
7. Return deterministic receipt material with ordered unit execution receipts, build-script metadata runs, blocker, and receipt hash.

The rail remains bounded: no unsupported Cargo unit modes, no unsupported target kinds, no parallel scheduler, and no ambient Cargo orchestration after oracle capture.

## Receipt shape

The combined CLI receipt keeps the captured `rust_plan` and adds `topology_execution` with:

- `execution_status`
- `claim`
- ordered `unit_executions`
- `build_script_metadata_runs`
- structured `blocker`
- self-reference-safe `receipt_hash`
