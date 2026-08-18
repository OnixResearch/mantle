# Design: Native Rust registry workspace dependency topology planning

## Current state

Mantle's native Rust registry planning/execution rail supports declared vendored registry source facts, unified topology execution, registry host producers, proc macros, transitive registry dependency chains, bounded feature-selected optional dependencies, and bounded target-cfg dependency edges.

The native manifest parser sees workspace members, but registry dependency inheritance through `[workspace.dependencies]` plus member `{ workspace = true }` is not represented as explicit native facts. Cargo can resolve those edges implicitly, so Mantle must either model a bounded subset or block before execution.

## Approach

1. Extend native workspace manifest parsing with a bounded `workspace.dependencies` map.
2. Add receipt evidence for workspace dependency inheritance decisions, including:
   - workspace root manifest;
   - member package id/name;
   - member dependency key;
   - inherited package name/version/source selector;
   - selected manifest path for path or vendored registry sources;
   - ready source fact identity for registry packages;
   - decision/blocker class.
3. Resolve member dependencies with `{ workspace = true }` only when:
   - the dependency key exists in root `[workspace.dependencies]`;
   - the inherited entry is a supported path dependency or declared vendored-registry dependency;
   - any explicit package rename is deterministic;
   - no unsupported inherited feature/default-feature/platform behavior is required;
   - selected registry packages have ready native registry source facts.
4. Feed selected inherited dependencies into native package dependency facts so the existing unit graph/topology rail executes producer-first and binds dependency artifacts through existing receipts.
5. Emit deterministic blockers before `rustc` for unsupported or ambiguous workspace dependency inheritance.

## Receipt shape

The implementation should add a stable native receipt surface near `native_package_target_planning`, for example per-package `workspace_dependencies` or a graph-level summary. It should be sorted/deduped and included in native hash material.

## Verification

- Positive CLI fixture: workspace root declares `[workspace.dependencies] demo_ws_leaf = { package = "demo-ws-leaf", version = "=0.1.0" }`; member app or mid crate declares `demo_ws_leaf = { workspace = true }`; vendored registry source facts are ready; topology executes leaf before consumer and binds artifact digests.
- Negative CLI fixture: member declares `{ workspace = true }` but root entry is missing or uses unsupported inherited feature/default-feature behavior; Mantle reports deterministic blockers and zero affected `rustc` executions.

## Risks / boundaries

This is not Cargo's full workspace dependency resolver. The bounded fragment should stay explicit, receipt-backed, and fail-closed.
