# Proposal: Native Rust registry target-cfg topology planning

## Summary

Add a bounded native planning seam for vendored registry dependency edges declared under explicit target-specific `cfg(...)` dependency tables. Mantle should represent supported target-cfg dependency selection from native manifest facts and ready registry source facts, then either feed the selected graph into topology execution or block deterministically before `rustc` when platform/cfg behavior would require hidden Cargo resolver semantics.

## Problem

Mantle now executes vendored-registry source, host, proc-macro, transitive, and bounded feature-selected dependency graphs through explicit native facts. Real registry crates also commonly use target-specific dependency tables such as `target.'cfg(unix)'.dependencies` or `target.'cfg(target_os = "linux")'.dependencies`.

Without an explicit native target-cfg seam, those edges are either omitted from Mantle's native graph or depend on Cargo resolver/platform behavior that is not represented in receipts.

## Scope

- Parse a narrow supported target-cfg dependency surface from native manifests.
- Support deterministic cfg predicates tied to the active Rust target triple where Mantle can decide them without Cargo.
- Require ready native registry source facts for selected vendored registry dependencies.
- Preserve unselected target-cfg dependencies as non-executed evidence or omit them from execution with explicit receipt material.
- Emit deterministic blockers for unsupported cfg expressions, target-specific feature behavior, ambiguous platform selection, or resolver-dependent behavior.
- Add positive and negative CLI JSON coverage.

## Non-goals

- Full Cargo cfg expression compatibility.
- Full platform resolver compatibility.
- Cross-target matrix execution.
- Network/index access, `$CARGO_HOME`, ambient registry caches, version solving, or Cargo build orchestration.
