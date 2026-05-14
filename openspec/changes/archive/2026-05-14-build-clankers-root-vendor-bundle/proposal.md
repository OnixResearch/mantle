# Build Clankers Root Vendor Bundle

## Why

`build-clankers-with-crunch` proved small Clankers rungs with fixed/offline Cargo sources, but the root `clankers` binary expands the Cargo closure to 1,169 vendored crates and roughly 1.2 GiB of vendor data before compression. The current committed-source/per-crate-input pattern is too large to land directly in git and too unwieldy for a generated Nickel derivation.

## What Changes

- Add a scalable fixed-input representation for the root `clankers` source and Cargo vendor closure.
- Preserve the existing invariants: offline Cargo, fixed source/vendor closure, sandbox-local `CARGO_HOME`, deterministic `CARGO_TARGET_DIR`, and no build-time network.
- Add `packages/clankers/clankers.ncl` only after the root closure representation is reproducible and not a monolithic large git artifact.
- Record root build evidence, output path, binary digest, and smoke output.

## Impact

- Blocks completion of the parent root-binary and smoke tasks until this change lands.
- Enables the parent change to carry the small-rung proof while explicitly deferring the large-root packaging design.
