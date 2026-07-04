## Why

The current Cargo-free fixed-point proof is blocked before a stage1 binary by a deterministic vendor checksum mismatch for `astral-tokio-tar@0.6.3`. That is useful diagnostic evidence, but it leaves the strongest Nix-free proof path unable to produce a fresh fixed-point result.

Mantle needs a focused change that repairs or refreshes vendored source material against `Cargo.lock`, proves the repair with the native source-planning classifier, and reruns the expensive fixed-point proof without overstating success if the frontier moves.

## What Changes

- Audit the declared Cargo source material for `astral-tokio-tar@0.6.3` and adjacent vendored crates.
- Repair `vendor-deps/`, checksum metadata, or lock/materialization declarations so native registry source planning agrees with `Cargo.lock`.
- Keep source-material validation fail-closed and deterministic when a vendored package drifts.
- Rerun the Cargo-free fixed-point proof and record either success evidence or the next exact blocker.
- Update operator proof docs with the new digest/status evidence.

## Impact

- **Files**: vendored source material or checksum metadata, proof scripts if needed, operator evidence docs, and archived proof evidence.
- **Testing**: native registry source planning checks, Cargo-free self-build classifier tests, and one expensive fixed-point proof rerun.

## Out of Scope

- Replacing the native Rust topology planner.
- Claiming full fixed-point success if the proof moves to a different blocker.
- Updating unrelated dependency versions for convenience.
