# Proposal: reuse-rust-topology-outputs

## Summary

Add bounded reuse evidence for Rust unit outputs produced by Mantle's explicit `rust-plan --execute-topology` rail.

## Motivation

Mantle can now execute supported mixed Rust topologies without Cargo orchestration, but every run currently behaves like a fresh `rustc` execution. The next Cargo-replacement boundary is proving that repeated topology runs can explain rebuild versus reuse from explicit receipt material, while failing closed when cached output evidence is stale or incomplete.

## Scope

- Persist per-unit execution receipts next to declared output artifacts.
- Reuse prior unit outputs only when the prior receipt matches the current explicit inputs, toolchain identity, dependency/host artifact digests, declared outputs, and current output BLAKE3 digests.
- Return deterministic blockers before `rustc` when prior cached output evidence is stale or missing required artifacts.
- Add CLI coverage for a successful repeated `--execute-topology` reuse and a stale cached artifact blocker.

## Non-goals

- General cache storage or substitution.
- Cross-output-root reuse.
- Parallel scheduling.
- Full Cargo scheduler/cache parity.
- Reusing unsupported target kinds or unit modes.
