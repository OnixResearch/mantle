## Why

Mantle now records Rust source closures, Cargo oracle unit graphs, reviewable per-unit `rustc` derivation plans, host/proc-macro boundaries, and fail-closed unsupported behavior. That evidence is necessary but still planning-only: supported Rust units are not yet executed through Mantle's build path, so Mantle cannot produce output artifact receipts or make a bounded Cargo-free build claim for even the simplest supported `lib`/`bin` unit.

`execute-rust-unit-derivations` moves the next narrow seam from receipt planning to actual unit execution while keeping the same fail-closed posture. The first execution slice should consume accepted `unit_derivation_graph` nodes, invoke `rustc` using explicit receipt material, materialize declared outputs, and record artifact digests/rebuild reasons without falling back to Cargo as a hidden build orchestrator.

## What Changes

- Add a Rust unit execution rail that consumes ready `unit_derivation_graph` nodes for supported `lib`/`bin` units.
- Add execution receipts that bind unit identity, source-closure digest, dependency/host artifact digests, toolchain identity, `rustc` argument digest, output artifact digest, and rebuild/reuse reason.
- Require deterministic blockers when source closure material, dependency artifacts, host artifacts, declared outputs, or supported execution preconditions are missing.
- Keep claims bounded: this change proves execution for the supported explicit-unit subset only, not broad Cargo compatibility, compiler correctness, native-link probing, doctests/tests/examples, or full bootstrap correctness.

## Impact

- **Files**: expected changes in `src/rust_plan.rs` and/or a focused Rust unit execution module plus CLI integration where the execution rail is exposed.
- **Testing**: focused positive unit execution fixture, missing-input negative fixtures, receipt digest assertions, `cargo fmt --check`, focused Rust tests via `nix develop -c cargo ...`, upstream Cairn validate/gates, and `git diff --check`.
