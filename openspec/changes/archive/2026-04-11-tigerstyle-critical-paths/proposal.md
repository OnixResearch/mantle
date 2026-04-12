## Why

The Tiger Style audit on 2026-04-12 found same class of structural debt in
several critical paths:

- `src/main.rs::run` still handles too many unrelated command flows in one
  function.
- `src/self_build.rs::cmd_self_build` still mixes staging, tool bootstrapping,
  build execution, and verification in one shell path.
- `crates/crunch-build/src/orchestrate.rs` still mixes pure content-addressed
  planning with registry mutation, store mutation, and logging.
- `crates/crunch-build/src/rewrite.rs` and `crates/crunch-store/src/export.rs`
  still recurse through tree-shaped castore data instead of using explicit
  bounded worklists.
- Assertion density is still thin in large, safety-critical functions.

That leaves crunch with code that works, but is harder to audit, harder to
verify, and easier to regress in exactly the paths that control build identity,
self-build, and store export.

## What Changes

- Require narrow shell/orchestrator entrypoints for the biggest command paths,
  especially CLI dispatch and self-build.
- Require content-addressed output finalization to split pure planning from
  effectful persistence and logging.
- Require castore tree traversal in critical paths to use explicit bounded
  iteration instead of recursive async helpers.
- Require stronger assertion and limit coverage in these critical paths.

## Capabilities

### New Capabilities
- `tigerstyle-critical-path-auditability`: contributors can inspect the main
  build/self-build control flow without reading one giant mixed shell function
- `ca-finalization-planning-boundary`: CA output planning can be tested as pure
  logic before store mutation happens
- `bounded-castore-traversal`: rewrite/export tree walks have explicit queue or
  stack bounds instead of relying on recursion depth

### Modified Capabilities
- `cli-dispatch`: top-level command dispatch becomes a thin shell over focused
  handlers
- `self-build-orchestration`: stage control remains in one place, but each
  stage becomes a dedicated helper with explicit contracts
- `build-finalization`: CA rewrite/path planning becomes distinct from
  persistence, registry updates, and logging

## Impact

- **Files**: `src/main.rs`, `src/self_build.rs`, `crates/crunch-build/src/orchestrate.rs`, `crates/crunch-build/src/rewrite.rs`, `crates/crunch-store/src/export.rs`, and supporting modules/tests
- **APIs**: internal helper boundaries and test seams will change; CLI surface should stay stable
- **Dependencies**: none required
- **Testing**: targeted unit tests for pure planning helpers, regression tests for bounded traversal, and `openspec validate tigerstyle-critical-paths`
