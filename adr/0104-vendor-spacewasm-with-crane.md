# ADR 0104: Vendor SpaceWasm with Crane

**Status:** Accepted

## Context

The SpaceWasm reference package imported its Cargo lock through
`rustPlatform.importCargoLock`. That path fetched selected crates through the
blocked `crates.io/api` endpoint. The reference source, lockfile, Rust 1.91.1
toolchain, targets, and feature selections were still valid.

Mantle already uses Crane for reachable `static.crates.io` component vendoring.
A manual Cargo directory configuration cannot point at Crane's hashed vendor
root because Crane emits its own exact source mapping in `config.toml`.

## Decision

Pass the component-scoped Crane library into `nix/spacewasm-reference.nix`.
Build the vendor closure with `vendorCargoDeps` from the pinned upstream lock.
Copy Crane's generated `config.toml` into each offline SpaceWasm build.

Keep the NASA source revision, upstream lockfile, Rust version, host and wasm
targets, package commands, and feature selections unchanged. Keep the complete
Crane vendor directory in the reference dependency-closure artifact.

## Consequences

- Locked crates use the reachable Crane/static-crates transport.
- Offline Cargo resolves Crane's hashed vendor directories correctly.
- The dependency-closure and final bundle identities change because their
  vendor representation changes.
- Rebuild verification must confirm the new bundle is deterministic.
- This decision does not change SpaceWasm semantics or claim upstream
  correctness.
