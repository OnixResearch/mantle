# Proposal: Native Rust dev-dependency test topology planning

## Summary

Introduce a bounded planning seam for dev-dependency/test topology facts, starting fail-closed and then proving a narrow supported test dependency shape.

## Motivation

Current native planning blocks `dev-dependencies`; test surfaces matter, but have broader mode/profile implications. This change should add explicit evidence and blockers before any broad test execution claim.

## Scope

- model dev-dependency/test-mode facts separately from build topology facts.
- retain fail-closed blockers for unsupported test/run/doctest modes until explicitly supported.
- add a narrow supported fixture for a registry dev-dependency used by a test target when receipt evidence is complete.
- verify no normal build topology claim silently consumes dev-dependency material.

## Non-goals

- full Cargo test runner compatibility.
- doctest/run/bench mode execution.
- feature resolver parity for test-only graphs.
- network/cache fallback or version solving.

## Expected outcome

Mantle gains a bounded, receipt-backed native Rust planning slice with positive and negative CLI coverage, validated Cairn gates, and accepted spec synchronization after archive.
