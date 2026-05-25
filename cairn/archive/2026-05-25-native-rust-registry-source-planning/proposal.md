## Why

Mantle's native Rust planner now handles a bounded local/path subset through native package facts, host-unit graph planning, unified topology execution, and explicit output reuse. That proves the execution rail, but it still leaves third-party source material at the wrong trust boundary: registry and vendored crates are either outside the supported native fragment or can be confused with ambient Cargo cache behavior.

Real Mantle/Crunch builds need many crates whose source identity comes from `Cargo.lock`, registry checksums, and checked-in vendor trees. Before Mantle can make broader Cargo-free Rust build claims, it needs a narrow, reviewable source-planning seam that turns those dependency sources into explicit Mantle-owned facts and fails closed when declared material is missing or stale.

## What Changes

- Add a native Rust registry/vendored source-planning contract for packages resolved from `Cargo.lock` plus declared local vendor/source material.
- Bind registry package identity to lockfile package coordinates, checksum material, vendor-relative source roots, and deterministic BLAKE3 source-tree digests.
- Compare supported native registry source facts with retained Cargo oracle material, while keeping Cargo as evidence only.
- Fail closed for missing lockfile checksums, missing/unreadable vendor roots, digest mismatches, unsupported source kinds, and any attempt to rely on ambient Cargo registry/git/target caches.
- Add focused positive and negative `rust_plan` / `rust_plan_cli` coverage before expanding unit graph execution across registry-backed crates.

## Impact

- **Files**: likely `src/rust_plan.rs`, `tests/rust_plan_cli.rs`, focused fixture helpers, and this Cairn change package.
- **Testing**: focused Rust planner/CLI tests, `cargo fmt --check`, Cairn validate, proposal/design/tasks gates, and `git diff --check` before implementation closeout.
