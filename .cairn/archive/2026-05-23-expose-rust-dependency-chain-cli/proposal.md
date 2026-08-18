## Why

Mantle can now execute a bounded explicit Rust dependency edge internally: a producer `lib` unit is compiled first, its produced artifact is rebound into a supported consuming target unit, and ordered chain receipt evidence is emitted. That seam is still not operator-facing through the `mantle rust-plan` CLI, so users cannot capture reviewable JSON evidence for the chain without writing Rust tests or calling internal functions.

`expose-rust-dependency-chain-cli` makes the bounded dependency-chain execution rail available as first-class CLI evidence while preserving the existing Cargo-free and fail-closed boundaries.

## What Changes

- Add a `mantle rust-plan` CLI flag to execute the first bounded Rust dependency chain from the captured explicit derivation graph.
- Reuse the existing `--execution-output-root` artifact directory for both producer and consumer outputs.
- Emit a JSON receipt that includes the captured Rust plan and the chain-level dependency execution receipt.
- Preserve existing single-unit execution CLI behavior and keep the claim bounded to one explicit dependency edge.

## Impact

- **Files**: expected changes in `src/main.rs`, `src/rust_plan.rs`, `tests/rust_plan_cli.rs`, and the Rust package-planning Cairn spec.
- **Testing**: focused CLI fixture with two path crates, assertion of ordered producer/consumer receipts and output digests, `cargo fmt --check`, focused Rust plan tests, focused CLI test, Cairn validation/gates, and `git diff --check`.
