# Proposal: Native registry workspace-dependency topology execution

## Summary

Introduce an explicit bounded execution seam for vendored-registry dependencies inherited through `[workspace.dependencies]`: execute the inherited registry producer before the workspace member consumer from native Mantle receipts, without Cargo orchestration, resolver fallback, ambient registry caches, or network/index access.

## Motivation

Mantle already plans workspace-inherited vendored-registry dependency topology evidence and can exercise it through the general unified topology path. The next useful proof is a named receipt boundary for this exact supported fragment, so downstream reviews can distinguish explicit workspace-dependency execution from broad topology execution and verify the producer-first artifact binding contract.

## Scope

- require ready `native_registry_workspace_dependency_topology_planning` evidence before any `native_registry_workspace_dependency_topology_execution` claim.
- execute a bounded topology containing one workspace member with selected `{ workspace = true }` vendored-registry dependency facts.
- run vendored-registry dependency producers before the workspace member consumer using explicit derivation args/env and digest-bound artifacts.
- emit a stable execution receipt with workspace root, member package, inherited dependency package ids, ordered unit execution receipts, blockers, bounded claim, and receipt hash.
- fail closed before `rustc` when planning/source/unit evidence is not ready or when workspace inheritance requires unsupported member-side features/default-feature/platform/resolver behavior.
- keep normal `--execute-topology` as a general topology receipt and add the explicit CLI receipt behind a separate flag.

## Non-goals

- full Cargo resolver compatibility or version solving.
- network/index access, `$CARGO_HOME`, ambient registry cache, or Cargo target-directory fallback.
- generalized multi-member scheduling or arbitrary workspace-dependency graphs.

## Expected outcome

Mantle gains a reviewable Cairn package for explicit workspace-inherited vendored-registry topology execution, with implementation tasks, positive/negative CLI fixtures, deterministic blockers, and accepted spec synchronization after archive.
