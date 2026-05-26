# Proposal: Native Rust registry workspace dependency feature inheritance

## Summary

Extend bounded workspace dependency inheritance to supported feature/default-feature selections on vendored-registry dependencies.

## Motivation

Basic `[workspace.dependencies]` plus member `{ workspace = true }` is now modeled, while inherited `features` and `default-features` intentionally block. Many real workspaces centralize feature selections at the workspace root.

## Scope

- parse supported workspace dependency `features = [...]` and `default-features = false|true` facts.
- combine inherited workspace feature facts with existing selected-feature/optional-dependency planning.
- require ready source facts for activated vendored-registry packages.
- emit receipt evidence for inherited feature selections and deterministic blockers for ambiguous/unsupported inheritance.

## Non-goals

- full Cargo feature resolver compatibility.
- implicit feature unification across unrelated members.
- all-features/no-default-features CLI behavior beyond explicit supported facts.
- network/cache fallback or version solving.

## Expected outcome

Mantle gains a bounded, receipt-backed native Rust planning slice with positive and negative CLI coverage, validated Cairn gates, and accepted spec synchronization after archive.
