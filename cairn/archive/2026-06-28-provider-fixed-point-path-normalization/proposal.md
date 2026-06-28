## Why

Provider-bound witness replay currently rebuilds the same source bytes under a witness-owned scratch path, but native rust-plan/provider fixed-point execution can embed absolute source, execution-output, or provider helper paths into rustc arguments and compile-time environment. That makes otherwise valid witness replays fail binary digest comparison for path reasons instead of source/toolchain reasons.

## What Changes

- Add a deterministic cargo-free provider replay mode that maps source and execution roots to stable logical prefixes at the rustc boundary.
- Ensure provider-only compile-time helper paths use deterministic placeholders instead of witness/publisher scratch paths.
- Record the deterministic path mode in rust-plan receipts so release evidence can explain the replay boundary.

## Impact

- **Files**: `src/main.rs`, `src/cargo_free_self_build.rs`, `src/rust_plan.rs`, `tests/release_cli.rs` if needed.
- **Testing**: focused rust-plan/cargo-free unit tests, Cairn validation/gates, then provider and witness replay proof evidence.
