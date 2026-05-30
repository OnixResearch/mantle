# Proposal: Prove Cargo-free self-build topology

## Why

Mantle can now prove a generated path-workspace topology without Cargo, but that proof stops before the repo can build its own CLI. Operators need a stricter next rung: run Mantle's native Rust planner/executor over the checked-out Mantle workspace with Cargo replaced by a failing shim, capture the receipt, and smoke-check the produced CLI binary. This is still not a full Crunch bootstrap or release fixed-point proof.

## What Changes

- Extend Cargo-free native planning so declared vendored registry and captured git sources can participate without Cargo metadata or unit graph calls.
- Add a self-build proof mode that runs `rust-plan --no-cargo-oracle --execute-topology` on the current Mantle workspace with Cargo forbidden.
- Record durable evidence: receipts, command streams, source/tool identities, output digests, Cargo-forbidden marker status, blockers, and CLI smoke output.
- State non-claims explicitly: no Cargo invocation, no network source fetch, no Crunch fixed-point/bootstrap claim, no release-quality reproducibility claim.

## Impact

- **Files**: `src/rust_plan.rs`, `scripts/prove-cargo-free-rust-plan.sh`, `tests/rust_plan_cli.rs`, `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: focused native planner tests, failing-Cargo-shim CLI/proof tests, full self-build proof or captured blocker bundle, and `cairn validate --root .`.
