## Why

Mantle's native Rust topology can execute explicit `rustc` units without Cargo as the hidden build orchestrator. That is the right build boundary, but it currently has no first-class way to require an external compiler policy such as Octet. Octet can already enforce Rust lints through a Dylint/rustc-driver path and standards checks, but Mantle receipts cannot yet prove that a produced Rust artifact was accepted only after those policy gates ran.

This change makes compiler policy an explicit adapter at Mantle's Rust unit invocation boundary. The goal is a bounded, receipt-backed claim: a Rust unit or topology was compiled and accepted under a named compiler-policy profile, with any policy exceptions surfaced as structured waivers rather than hidden bypasses.

## What Changes

- Introduce a generic Rust compiler policy adapter interface for native Rust unit execution.
- Add an Octet/Dylint adapter implementation that can route supported direct-rustc executions through an Octet-provided driver, lint library, config, and standards policy manifest.
- Add operator modes for plain execution, audit/warn execution, deny/error execution, and fail-closed required execution.
- Bind adapter identity into Rust execution receipts, output reuse decisions, and topology/fixed-point evidence so raw-rustc outputs cannot satisfy Octet-required builds.
- Preserve non-claims: Mantle may claim compliance with the configured policy profile, not program correctness, complete architecture proof, or whole-ecosystem Cargo parity.

## Impact

- **Files**: `src/rust_plan.rs`, `src/main.rs`, Rust-plan receipt structs, cargo-free self-build plumbing if the option is promoted there, CLI tests, and JSON receipt fixtures.
- **Testing**: adapter-resolution unit tests, positive and negative execution-cache tests, CLI parsing tests, fail-closed missing-adapter tests, Octet-required diagnostic tests, `cairn validate --root .`, and Cairn gates for this change.
