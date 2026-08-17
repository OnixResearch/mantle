# Proposal: Native rlib crate-type derivation planning

## Summary

Treat Cargo targets that report `rlib` as supported library derivation targets when planning reviewable Rust unit derivations.

## Motivation

Mantle's real dependency graph includes crates whose Cargo unit target kind is reported as `[cdylib, rlib]`. The current derivation rail blocked those units as unsupported even though the native build path only needs the `rlib` artifact to continue dependency topology execution.

## Scope

- Accept `rlib` as a library kind in Cargo oracle package/target comparison.
- Select `rlib` units as `target_kind = "lib"` for unit derivation planning.
- Keep pure `cdylib` or otherwise unsupported targets fail-closed.

## Non-goals

- Producing cdylib artifacts.
- Extending native linking semantics.
- Full Cargo crate-type parity.
