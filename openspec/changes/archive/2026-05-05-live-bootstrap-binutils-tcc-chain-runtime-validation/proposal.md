# Complete binutils-tcc runtime validation

## Why

Parent change `live-bootstrap-binutils-tcc-chain` completed the implementation surface but its V2-V5 validation exceeded the local drain budget. Supplying `bubblewrap` via `nix shell nixpkgs#bubblewrap` fixed the build prerequisite, but the first `bzip2-tcc` epoch remained inside the Mes prerequisite build after about thirty minutes.

## What Changes

- Resume the binutils-tcc epoch validation with a longer-running or more granular strategy.
- Preserve the successful preflight setup: `crunch doctor --profile build --store .crunch-drain/store --state-dir .crunch-drain/binutils-tcc-state` under `nix shell nixpkgs#bubblewrap`.
- Record V2-V5 evidence for epoch builds, no-host-leakage audit, post-musl linkage, and binutils smoke output.

## Scope

In scope: validation evidence and any narrowly scoped runtime/cache fixes required to make validation finish.

Out of scope: adding new bootstrap packages beyond the binutils-tcc chain or changing the already implemented derivation ladder unless required to make the existing validation complete.
