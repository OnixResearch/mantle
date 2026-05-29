# Proposal: Prove Mantle builds itself without Cargo as planner

## Problem

A Cargo-free command is not enough. Mantle needs release-quality evidence that it can build a meaningful Rust workspace, ideally itself or the Crunch/Mantle workspace, without Cargo participating in planning or build orchestration.

## Change

Create a proof workflow that builds Mantle/Crunch through the native planner/executor with Cargo disabled as a planner, records receipts, verifies outputs, and preserves auditable evidence.

## Impact

- **Files**: proof script/test, audit bundle writer, documentation, CI/local check plumbing.
- **Testing**: ignored/full proof plus fast preflight and negative Cargo-shim checks.
