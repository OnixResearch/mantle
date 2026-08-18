# Native self target-cfg predicate scope

## Summary

Extend Mantle's native target-cfg evaluator with a bounded, deterministic predicate subset needed by Mantle's own vendored dependency graph.

## Motivation

Mantle's self `rust-plan --execute-topology` now reaches native host/package graph checks, but package planning still reports many `unsupported-target-cfg-surface` blockers for common Cargo target predicates such as `cfg(not(windows))`, `cfg(any(...))`, `cfg(all(...))`, and target key-value predicates. These are normal build-relevant dependency selection surfaces, not execution behavior, so they should be represented explicitly rather than left as hidden Cargo resolver behavior.

## Scope

- Support deterministic evaluation of bounded `cfg(...)` expressions:
  - `unix`, `windows`
  - target key-values for os, arch, family, vendor, env, ABI, endian, pointer width, and atomic width
  - nested `not`, `any`, and `all`
  - exact target triple table names
- Treat unknown cfg feature/backend flags as false only when they are known optional selector names in the bounded fragment.
- Preserve fail-closed blockers for malformed or unknown cfg syntax.
- Add regression tests for positive selection, negative not-selection, exact triple tables, and unsupported malformed syntax.

## Non-goals

- Full rustc cfg grammar or build-script-emitted cfg evaluation.
- Feature resolver replacement beyond existing selected-feature facts.
- Widening normal topology execution semantics.
