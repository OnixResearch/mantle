# Native dependency feature edge scope

## Summary

Scope native Rust dependency-edge planning to the feature/default-feature selections Cargo actually exposes in the captured build unit graph, so unselected optional default dependencies do not block normal build topology execution.

## Motivation

The current Mantle self `rust-plan --execute-topology` probe reaches native package planning but still reports package blockers such as `hashbrown@0.14.5 -> ahash` even though Cargo did not select that optional default dependency for the current build graph. Treating every dependency package as if its manifest default feature is selected over-claims source requirements and hides the next real topology blocker.

## Scope

- Derive package selected-feature facts from the captured Cargo unit graph for the current invocation.
- Use those facts when deciding optional normal, build, and target-cfg dependencies.
- Allow dependency-edge `features` and `default-features` metadata when source resolution is otherwise explicit, because selected package features come from the unit graph oracle for this bounded phase.
- Preserve fail-closed blockers for dependency source kinds outside the declared path/vendored-registry fragment.

## Non-goals

- Implement Cargo's full feature resolver without the unit graph oracle.
- Claim full Cargo compatibility for all dependency kinds or target modes.
- Execute examples, benches, doctests, or general test topology through the normal build rail.
