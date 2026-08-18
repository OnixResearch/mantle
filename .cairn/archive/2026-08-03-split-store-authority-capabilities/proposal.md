# Change: Split store authority into narrow capabilities

## Why

`crunch-build::Builder` owns the full `crunch_store::StoreHandle`, and `Builder::store_handle()` exposes that handle to pipeline callers. The same value can read and write castore data, query remote caches, persist outputs, register roots, import sources, publish action results, replace action-result stores, and run garbage collection.

This makes store authority implicit. A later method added to `StoreHandle` also becomes available to build code unless review finds every caller. Mantle needs compile-time boundaries that keep build realization, source admission, root retention, and store administration separate.

## What Changes

- Replace the builder-owned `StoreHandle` with concrete, narrow capability values.
- Give build realization only high-level castore, cache, substitution, output-admission, and build-session operations.
- Give pipeline reporting a read-only output lookup capability and root retention a separate capability.
- Keep source import, garbage collection, repair, backend replacement, and publisher configuration in shell-owned capabilities.
- Remove raw writable service escape paths from `crunch-build` and `crunch-pipeline`.
- Add positive runtime tests and negative compile-time or source-policy tests for the authority boundary.
- Preserve store formats, report schemas, PathInfo signatures, output identities, and supported build behavior.

## Dependencies

- Coordinate implementation with `persist-rust-unit-castore-results` and `share-rust-unit-action-results`, because those changes add store and action-result call paths.
- Reuse admitted path and identity types from `extend-nominal-types-to-trust-boundaries` when available. This change does not duplicate that value-admission work.

## Non-Goals

- Replacing castore, PathInfo, Snix service traits, or store persistence formats.
- Changing cache trust, output signing, substitution, root-retention, or garbage-collection policy.
- Adding a generic capability framework or an async-trait matrix.
- Claiming that a Rust type proves host filesystem isolation or sandbox correctness.

## Impact

- **Affected specs:** `build-correctness`
- **Affected code:** `crunch-store`, `crunch-build`, `crunch-pipeline`, and focused CLI store wiring
- **Compatibility:** wire formats and accepted build results remain unchanged
- **Testing:** focused store/build/pipeline tests, compile-fail examples, negative API guards, first-party quality rails, and Cairn gates
