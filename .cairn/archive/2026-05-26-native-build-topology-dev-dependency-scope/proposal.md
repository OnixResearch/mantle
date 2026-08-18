# Native build topology dev-dependency scope

## Summary

Keep normal build-mode Rust package/target and unit topology planning from being blocked by dev-dependency-only option surfaces.

## Motivation

Mantle self planning had a ready `unit_derivation_graph`, but broader native package/target planning still reported many `unsupported-dev-dependency-options` blockers from test-only dependencies. Those blockers are not build-mode inputs for `--execute-topology`; treating them as build blockers makes the normal build rail less useful and conflates it with the explicit dev-dependency test topology rail.

## Proposed Change

- Parse dev-dependencies only as bounded path or declared registry source material for explicit dev-test rails.
- Ignore unsupported dev-dependency feature/default-feature/optional/target/workspace options for normal build-mode package/target facts.
- Scope native unit graph planning to Cargo's build unit package set so test-only/dev-only packages do not introduce normal build dependency edges.
- Preserve the explicit dev-dependency test topology rail for dev/test execution claims.

## Non-goals

- No Cargo orchestration fallback.
- No network/cache fallback.
- No widening of normal topology execution into a dev-test execution claim.
- No support for arbitrary dev-dependency feature resolution in build mode.
