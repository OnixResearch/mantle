# Proposal: Native Rust dev-dependency test topology execution

## Summary

Introduce a bounded execution seam for dev-dependency test topology facts: execute one supported test-target topology that consumes a vendored-registry dev-dependency through explicit Mantle receipts, while blocking unsupported Cargo test surfaces before `rustc`.

## Motivation

Mantle now plans dev-dependency/test topology facts, but the accepted seam intentionally stops short of execution. Real crates commonly use dev-dependencies in test targets, so the next useful proof is a narrow Cargo-free execution path that keeps test-mode behavior explicit instead of silently falling back to Cargo's test runner or ambient caches.

## Scope

- require ready `native_rust_dev_dependency_test_topology_planning` evidence before any dev-dependency test execution claim.
- execute a bounded supported test-target topology whose registry dev-dependency source facts are ready and whose artifacts can be scheduled through existing explicit topology execution primitives.
- emit receipts that bind test target identity, dev-dependency source facts, producer/consumer artifact digests, toolchain identity, rustc args, and bounded claim/non-claim text.
- fail closed before `rustc` for unsupported test surfaces: doctest/run/bench modes, hidden Cargo harness behavior, missing/stale vendor material, missing test-unit derivations, or any resolver behavior outside the native fragment.
- prove that normal build topology execution still does not silently consume dev-dependency material.

## Non-goals

- full `cargo test` runner compatibility.
- doctest, run, bench, examples, or integration-test directory discovery beyond the bounded fixture.
- feature resolver parity for arbitrary test-only graphs.
- network, `$CARGO_HOME`, Cargo registry cache, or Cargo target-directory fallback.

## Expected outcome

Mantle gains a reviewable Cairn package for the first dev-dependency test topology execution proof, with implementation tasks, positive/negative CLI fixtures, deterministic blockers, and accepted spec synchronization after archive.
