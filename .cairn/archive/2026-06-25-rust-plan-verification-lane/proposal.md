# Proposal: Rust-plan verification lane boundary

## Summary

Keep `mantle rust-plan` as an explicit verification lane while the practical project build workflow uses sandboxed offline Cargo. Rust-plan should produce bounded evidence and blockers, not silently become the default path for ordinary project builds.

## Motivation

Mantle's native Rust planner is advancing quickly, but it still makes bounded claims. Users need a stable project build path now, while maintainers need a rigorous lane for proving Cargo-free planner/executor coverage. Mixing those surfaces too early would create false expectations and accidental overclaims.

## Scope

- Define product-boundary rules for when `rust-plan` evidence is opt-in versus when it may be promoted into default project builds.
- Ensure reports classify native planner receipts as bounded verification evidence.
- Keep unsupported Cargo behavior fail-closed with deterministic blockers.
- Add docs and tests that route ordinary Rust project builds through the offline Cargo lane unless the operator explicitly asks for `rust-plan`.
- Define promotion criteria: representative compatibility rail evidence, positive/negative fixture coverage, non-claim wording, and current validation receipts.

## Non-goals

- No removal of existing `rust-plan` functionality.
- No claim that rust-plan is full Cargo-compatible.
- No automatic Cargo-free execution from `mantle build` until a later change satisfies explicit promotion criteria.
- No weakening of existing rust-plan fail-closed blockers.

## Target Spec Domains

- `rust-package-planning` for native Rust planner claim boundaries.
- `verification-evidence` for proof-before-claim and report wording.
