# Focused validation summary

## Baseline and compatibility

- Clean baseline: 222 Mantle `rust_plan::` tests and 57 Rust-plan CLI tests passed.
- Final `mantle-rust-plan-core`: 21 tests and one compile-fail doctest passed.
- Final `mantle-rust-plan`: 6 application and port tests passed.
- Final Mantle `rust_plan::`: 222 tests passed.
- Final Mantle `rust_plan_hexagon::`: 3 adapter tests passed.
- Final `tests/rust_plan_cli.rs`: 57 tests passed.
- Cargo-oracle facts, compatibility status, native unit identity, and receipt-preimage identity match checked golden fixtures.
- The legacy native unit identity keeps the exact accepted byte preimage.

One first full Rust-plan run had a pre-existing parallel fixture race: a temporary `counting-rustc` executable returned `Text file busy`. The exact failed test passed on immediate rerun, and the required 222-test command then passed. Both logs are preserved.

## Cargo-free fixture proof

The bounded `--full` fixture proof passed.

- Status: `success`
- Smoke output: `42`
- Cargo forbidden marker absent: `true`
- The proof bundle preserves the plan receipt, output digests, blocker summary, streams, preflight, and non-claims.

## Architecture and quality

- Host and `wasm32-unknown-unknown` core checks passed.
- The architecture checker reports zero findings and detects 13 forbidden authority fixtures.
- Focused Nix checks for the architecture rail, core tests, and core `wasm32` build passed.
- Strict first-party Clippy passed with `-D warnings`.
- The exact Tiger Style Nix gate passed after structural repairs. No allowance, baseline, or reduced scope was added.
- Machine-contract generation and validation passed with 24 contracted and 57 classified surfaces.
- Durable-publication adoption passed with refreshed exact Cargo and flake bindings.
- Formatting and `git diff --check` passed.
- Cairn validation reports `"valid": true`.
- Tracey reports 155/155.
- Proposal, design, and tasks gates return PASS with 10 completed tasks and V5 pending committed-source validation.

## Oracle checkpoint

- **Question:** Does the extraction isolate deterministic Rust planning while preserving accepted behavior?
- **Inspected evidence:** The core is `no_std + alloc`; the application defines five Mantle-owned ports; process-backed Cargo and compiler adapters moved outside `src/rust_plan.rs`; eight accepted legacy decisions delegate to the core; old and new tests, golden fixtures, CLI tests, Cargo-free proof, architecture negatives, Clippy, and Tiger Style pass.
- **Decision:** Accept the focused extraction. Keep filesystem parsing, Cargo JSON decoding, compiler execution, cache materialization, and CLI rendering in the existing shell and adapters.
- **Owner:** Mantle maintainers.
- **Next action:** Commit the implementation, then run committed-source Nix and lifecycle validation.

## Non-claims

The evidence does not prove Cargo equivalence outside the accepted matrix, rustc correctness, linker correctness, cache correctness, full ecosystem support, reproducibility, bootstrap correctness, or release eligibility.
