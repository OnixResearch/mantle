## Why

Mantle can execute one explicit Rust dependency edge, but real target-only Rust packages commonly contain more than one local library edge. Operators need a bounded next step that executes a target-only closure from the explicit `unit_derivation_graph` without jumping to Cargo orchestration, build scripts, proc macros, test/doctest modes, or full scheduling.

## What Changes

- Add a bounded target-unit topology executor over existing explicit Rust unit derivations.
- Execute supported target `lib`/`bin` units in dependency order, rebinding produced `.rlib` artifacts into downstream units before invoking `rustc`.
- Emit a chain/topology receipt with ordered per-unit execution receipts, blockers, output artifact BLAKE3 digests, and a bounded target-only claim.
- Expose the executor through `rust-plan` CLI JSON evidence.
- Fail closed for unsupported host/proc-macro/build-script/test/doctest/native-link shapes and missing dependency producers/artifacts.

## Non-Goals

- No Cargo build orchestration.
- No proc-macro/build-script/host artifact execution.
- No test/doctest/example execution.
- No full Cargo compatibility claim.
- No incremental cache scheduler beyond a deterministic one-shot topological execution rail.
