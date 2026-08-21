# Design: Pilot transactional reconciliation core

## Context

The shared core is published on Radicle at revision `eb2bd3441753af97bfcb247cef7cc22d72675b62`. Mantle owns GC plan meaning; the core owns generic compare-and-swap planning and persistence classification.

## Decisions

### Decision: One immutable pin, two dependency systems

Cargo and the Nix flake select the same Radicle source and revision. No ambient sibling paths.

### Decision: Plan identity binding only

The pilot maps an exact GC plan identity into the shared planner input and observes stale classification for a different plan. GC semantics, store writes, and evidence authority stay in Mantle.

## Claim Boundary

A passing pilot proves composition and identity binding. It does not prove Redb durability behavior, GC correctness, or release eligibility.

## Test Design

Positive: current plan classifies as current through the shared planner.
Negative: a changed plan identity classifies as stale.
