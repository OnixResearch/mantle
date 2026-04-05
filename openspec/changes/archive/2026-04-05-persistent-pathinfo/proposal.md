## Why

`Builder` currently stores `PathInfo` in an in-memory `HashMap<String, PathInfo>`
(`built_outputs`). When the process exits, all knowledge of what was
built is lost. The files remain in the store, but crunch can't answer:

- Was this path built by crunch or is it leftover from Nix?
- What derivation produced this path?
- What are this path's runtime references?
- Was this path a CA derivation, and what was its content hash?

The cache check (`all_outputs_exist`) works around this by probing
the filesystem — if the output path exists as a file/directory, it's
"cached." But this is lossy: it doesn't know references, deriver,
NAR hash, or CA info. And it breaks entirely for CA derivations,
where the mapping from derivation identity to output path must be
recorded.

snix-store already ships `RedbPathInfoService` — a PathInfoService
backed by a single redb file. The code is in our vendored crate.
We just don't use it.

## What Changes

- **Wire `RedbPathInfoService` into `Builder`.** After a successful
  build, persist `PathInfo` to a redb database in addition to the
  in-memory map. On startup, the database is the source of truth
  for what's been built.

- **Cache checks query PathInfoService.** Replace the filesystem
  probe (`path.exists()`) with a `PathInfoService::get()` lookup
  plus a filesystem existence check. This gives us reference info,
  deriver, and NAR hash without re-ingesting from disk.

- **Database location.** The redb file lives at
  `$XDG_STATE_HOME/crunch/pathinfo.redb` (or `$CRUNCH_STATE_DIR`),
  alongside the existing `logs/` directory.

- **CLI `crunch store` subcommand.** Read-only queries: list known
  paths, show path info, verify NAR hashes.

## Capabilities

### New Capabilities

- `persistent-pathinfo`: PathInfo persisted to redb, survives
  process restart
- `store-query`: CLI subcommand for inspecting stored path info

### Modified Capabilities

- `sandboxed-build`: cache check uses PathInfoService instead of
  filesystem-only probe

## Impact

- **Files**: `crates/crunch-build/src/orchestrate.rs`,
  `src/main.rs`, new `src/store.rs` (CLI subcommand)
- **APIs**: `Builder::new()` gains a `PathInfoService` parameter.
  Cache check uses `PathInfoService::get()`.
- **Dependencies**: `redb` (already a transitive dep via
  snix-store and snix-castore)
- **Testing**: Tests that check cache behavior need a
  PathInfoService (redb in-memory or real file). Existing
  mock tests can use `LruPathInfoService` or a test wrapper.
