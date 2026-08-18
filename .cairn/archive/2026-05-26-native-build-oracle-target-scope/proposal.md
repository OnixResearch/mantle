# Native build oracle target scope

## Summary

Scope native package/target planning's Cargo oracle comparison to build-relevant target kinds for the normal build topology.

## Motivation

Mantle self planning currently reports many `unsupported-cargo-oracle-target-kind` blockers from Cargo metadata targets such as examples and benches. Those targets are not part of the normal build unit graph used by `rust-plan --execute-topology`; treating them as package-planning blockers hides the next real build-mode blockers and conflates normal build topology with future example/bench/test rails.

## Proposed Change

- Treat Cargo oracle example/bench-only target kinds as out of scope for normal build package/target planning.
- Keep comparing supported build-relevant target kinds (`lib`, `rlib`, `bin`, `custom-build`, `proc-macro`, bounded `test`) against native facts.
- Preserve explicit future rails as the place to claim example/bench execution.
- Add regression coverage proving out-of-scope Cargo oracle targets do not block normal build-mode planning.

## Non-goals

- No example/bench execution support.
- No broad Cargo target-kind compatibility claim.
- No Cargo orchestration fallback.
