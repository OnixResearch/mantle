# Proposal: Adopt nix-archive at the filesystem NAR boundary

## Why

Mantle computes Nix Archive (NAR) facts through adapted Snix code. Several filesystem paths first enter temporary castore services, then render back into NAR bytes for hashing.

The `cachix/nix-archive` crate provides byte-safe NAR encoding, hashing, decoding, and restoration without a Nix runtime. Its filesystem encoder preserves non-UTF-8 names and streams file payloads.

A full replacement is not safe. Mantle also needs asynchronous NAR rendering and ingest over castore services. The current `nix-archive` decode and restore APIs accept complete byte slices and do not replace that streaming boundary.

## What Changes

- Pin and review `nix-archive` version `0.1.0` and upstream commit `14362ab589daa4869bda744d4fbe26a1914b5491`.
- Add one shared Mantle NAR adapter for filesystem encoding, hashing, explicit case-hack policy, and parity decisions.
- Move selected filesystem NAR observations from temporary castore round trips to the shared adapter.
- Keep Snix NAR rendering and ingest for castore, remote cache, archive import, and other asynchronous streams.
- Add differential evidence across `nix-archive`, Snix, durable fixtures, generated trees, and `nix-store --dump` when Nix is available.
- Keep production decode and restore out of scope until they meet Mantle streaming, mutation, and publication rules.
- Record exact adapter, upstream, case-hack, hash, size, and parity facts in machine-readable evidence.

## Dependencies

- ADR 0071 defines the split filesystem and castore NAR boundary.
- `import-nario-v2-store-archives` remains a separate active change. Its production NAR payload path must stay streaming.
- `backport-snix-correctness-fixes` remains responsible for selected fixes in the adapted Snix tree.

## Non-Goals

- Replacing `snix-store` NAR rendering or ingest.
- Buffering complete cache or Nario payloads for `decode_events` or `restore_path`.
- Adding a new store archive format or changing Nario v2 framing.
- Claiming Nix, Snix, or `nix-archive` correctness from parity tests.
- Making NAR observations atomic filesystem snapshots.
- Adopting upstream trust, PathInfo, castore, publication, or release policy.

## Impact

- **Planned files**: workspace manifests, a shared NAR adapter crate, store verification, project source hashing, fixtures, tests, documentation, and evidence.
- **Compatibility**: selected filesystem NAR facts must remain byte-compatible. Castore and transport behavior must remain unchanged.
- **Platforms**: the dependency is Unix-only. Unsupported targets must fail at an explicit package boundary.
- **Offline builds**: lock and vendored Cargo inputs must remain complete for self-build and witness rebuild paths.
- **Current effect**: lifecycle planning and architecture records only. Mantle does not yet use `nix-archive` in production.
