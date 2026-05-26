# Proposal: Native registry patch-source topology execution

## Summary

Introduce an explicit bounded execution seam for local `[patch.crates-io]` registry replacements: execute the patched local source producer before its consumer from native Mantle receipts, without Cargo orchestration, resolver fallback, ambient registry caches, or network/index access.

## Motivation

Mantle already plans local patch-source registry replacements and can exercise them through the general unified topology path. The next useful proof is a named receipt boundary for this exact supported fragment, so reviews can distinguish explicit patch-source execution from broad topology execution and verify producer-first artifact binding.

## Scope

- require ready `native_registry_patch_source_planning` evidence before any `native_registry_patch_source_topology_execution` claim.
- execute a bounded topology containing one local `[patch.crates-io]` replacement and one supported consumer.
- run the patched local source producer before the consumer using explicit derivation args/env and digest-bound artifacts.
- emit a stable execution receipt with consumer package, patch source package ids, ordered unit execution receipts, blockers, bounded claim, and receipt hash.
- fail closed before `rustc` when source/package/unit evidence is not ready or when patch behavior requires unsupported registry/source/resolver behavior.
- keep normal `--execute-topology` as a general topology receipt and add the explicit CLI receipt behind a separate flag.

## Non-goals

- full Cargo resolver compatibility or version solving.
- non-crates.io patch registries, git patches, network/index access, `$CARGO_HOME`, ambient registry cache, or Cargo target-directory fallback.
- generalized multi-patch scheduling.

## Expected outcome

Mantle gains a reviewable Cairn package for explicit local patch-source registry topology execution, with implementation tasks, positive/negative CLI fixtures, deterministic blockers, and accepted spec synchronization after archive.
