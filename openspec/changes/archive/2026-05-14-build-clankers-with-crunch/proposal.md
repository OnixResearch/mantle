## Why

Crunch needs a concrete external Rust workspace target to prove that its bootstrap Rust/Cargo path can build useful software outside Crunch itself. `../../clankers/` is a high-value target because it is a large Rust workspace with path dependencies, git dependencies, private SSH source pins, build scripts, and Nix-based reference packaging. Attempting the whole binary first would mix Cargo source-closure problems with native dependency problems, so the change should define an incremental, evidence-backed build ladder.

## What Changes

- **Define**: Add an external Rust workspace build capability for Clankers, starting with a low-dependency crate and climbing toward the root `clankers` binary.
- **Constrain**: Require a fixed source/vendor closure and offline Cargo operation inside Crunch builds; no network fetches during derivation execution.
- **Verify**: Require each rung to record the exact Crunch derivation, Cargo command, source/vendor inputs, and smoke evidence before claiming success.
- **Defer**: Keep full Clankers VM checks, plugin bundles, source-built Rust, and onnxruntime-heavy feature closure out of the first success criteria unless needed by the root binary.

## Capabilities

### New Capabilities
- `external-rust-workspace-build`: Build an external Rust workspace through Crunch using a frozen Cargo/source closure.
- `clankers-crunch-build-ladder`: Incrementally build Clankers crates before the root `clankers` binary.

## Impact

- **Files**: New `packages/clankers/*.ncl` derivations or equivalent package-set files, source/vendor closure manifests, focused tests/receipts, and docs/notes for the build ladder.
- **APIs**: No public Crunch CLI API change required for the first rung; implementation may add narrow helpers for source/vendor closure validation if necessary.
- **Dependencies**: Initial dependency is existing `bootstrap/rust.ncl`; later rungs may add explicit Crunch packages for native build-script tools only when a crate requires them.
- **Testing**: `crunch eval`, shell syntax extraction, offline `crunch build`, Cargo `--locked --offline`, and output smoke checks.
