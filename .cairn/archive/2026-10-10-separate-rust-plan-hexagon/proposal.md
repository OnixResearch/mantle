# Change: Separate the Rust-plan hexagon

## Why

`src/rust_plan.rs` currently owns deterministic package and unit policy together with workspace traversal, Cargo subprocesses, rustc execution, cache access, environment access, receipt rendering, and CLI presentation. The `CargoOracle` port and its process adapter also share one module and return the CLI-owned `RunError` type.

This shape makes the functional core difficult to reuse, test, or compile without host authority. It also permits adapter and presentation concerns to change alongside package-planning policy.

## What Changes

- Add a dedicated `no_std + alloc` Rust-planning core over normalized in-memory package, manifest, lock, target, feature, toolchain, and artifact facts.
- Move package selection, feature resolution, host and target classification, unit topology, action planning, compatibility classification, deterministic identities, blockers, and receipt preimages into the core.
- Add application-owned ports for workspace facts, Cargo oracle capture, compiler inspection, unit execution, and Rust cache access.
- Keep filesystem, process, environment, Cargo, rustc, store, cache, path, and rendering details in adapters and the imperative shell.
- Replace `RunError` at application ports with typed Rust-plan domain and adapter errors.
- Preserve accepted JSON, canonical identities, execution behavior, and Cargo-oracle parity through compatibility fixtures.
- Add positive, negative, compile-fail, architecture, and parity tests.

## Non-Goals

- Expanding Mantle's supported Cargo behavior.
- Changing package resolution, feature selection, unit topology, build-script, proc-macro, linker, cache, or execution semantics.
- Replacing Cargo as the bounded oracle where current policy requires it.
- Claiming Cargo equivalence, rustc correctness, native-link correctness, or full ecosystem support.
- Creating ports for deterministic internal helper functions.

## Dependencies

- `complete-store-capability-migration` supplies the narrow cache and artifact capabilities used by Rust-plan adapters.
- The accepted `rust-package-planning` and `rustc-cache-adapter` requirements remain authoritative.
- Existing Cargo-free and source-built proof receipts remain compatibility evidence, not core inputs.

## Impact

- **Affected spec:** `rust-package-planning`
- **Affected code:** `src/rust_plan.rs`, Rust-plan CLI integration, cache adapters, source planning, topology execution, receipts, and tests
- **Compatibility:** accepted CLI, JSON, digest, unit identity, and execution behavior remain unchanged
- **Testing:** core policy tests, adapter tests, Cargo parity fixtures, execution fixtures, negative blockers, dependency guards, and Cairn gates
