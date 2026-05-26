# Proposal: Native Rust workspace member glob planning

## Summary

Support a bounded, deterministic one-level workspace member glob surface while retaining explicit blockers for ambiguous workspace discovery.

## Motivation

The native planner currently supports explicit workspace members only; many workspaces use `members = ["crates/*"]`. A narrow deterministic glob unlocks more fixtures without adopting Cargo workspace discovery wholesale.

## Scope

- expand supported one-level member globs such as `crates/*` deterministically under the workspace root.
- record expanded member evidence in native package planning receipts.
- reject recursive, parent-traversing, empty, duplicate, or ambiguous glob patterns with deterministic blockers.
- keep existing explicit-member behavior unchanged.

## Non-goals

- full Cargo workspace discovery compatibility.
- exclude/default-members resolver parity beyond explicit supported facts.
- filesystem traversal outside the workspace root.
- implicit package discovery.

## Expected outcome

Mantle gains a bounded, receipt-backed native Rust planning slice with positive and negative CLI coverage, validated Cairn gates, and accepted spec synchronization after archive.
