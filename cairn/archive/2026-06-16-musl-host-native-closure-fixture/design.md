## Context

The current command cannot yet run this path with the real Rust provider because the real provider is GNU-host. A synthetic unit fixture can still exercise the collection/materialization core without faking a user-facing claim.

## Design

### Functional core

Construct deterministic temp directories containing:

- a fake Rust provider root with `bin/rustc`;
- a source-root musl root with target-prefixed compiler/linker helpers and `<target>/lib` runtime/startup files;
- source-built provider identities for Rust, host, and target inputs.

Call `collect_native_closure_candidates` with both host and target identities classified as `SourceRootMusl`, then call `materialize_source_built_native_closure` on the candidates.

### Assertions

The fixture asserts that the manifest has zero seed exceptions, includes the required source-built members, and maps host `cc`/`ld`/runtime entries to the source-root musl paths. A negative coverage path remains the current GNU-host guardrail from the previous change.

### Claim boundary

This is a unit fixture only. It does not report a real source-built provider proof and does not retire `not-source-built-toolchain-closure`.
