# Proposal: Prove bounded Cargo-free Rust planning

## Problem

A Cargo-free command is not enough. Mantle needs audit-grade evidence that the native planner/executor can build a nontrivial Rust topology without Cargo participating in planning or build orchestration. This change is a bounded proof, not a Mantle/Crunch self-build claim.

## Change

Create a proof workflow that builds a generated multi-crate path workspace through the native planner/executor with Cargo disabled as a planner, records receipts, verifies outputs, and preserves auditable evidence. The proof must state its compatibility class and non-claims explicitly.

## Impact

- **Files**: proof script/test, audit bundle writer, documentation, CI/local check plumbing.
- **Testing**: fast preflight, full bounded proof, and negative Cargo-shim checks.
