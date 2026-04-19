## Why

crunch can push build results to a binary cache directory (`crunch store push`)
and can substitute from remote HTTP caches during builds, but has no way to
import paths from a local binary cache directory back into the local store.
This means:

- Build artifacts pushed on machine A cannot be imported on machine B without
  standing up an HTTP server and using `--substituters`.
- CI pipelines that share caches via rsync, S3 sync, USB, or artifact
  archives cannot feed results directly into a local crunch store.
- The push→pull round-trip cannot be verified entirely within crunch.

All primitives exist:

- `NarInfo::parse_with_store_dir()` parses narinfo text with configurable
  store prefix.
- `ingest_nar_and_hash()` streams a NAR archive into castore (blob +
  directory services) and verifies the hash.
- `persist_and_export_signed_output()` persists a PathInfo with signature
  and exports the output to disk.
- `nix_compat::narinfo::VerifyingKey` / signature verification are used
  by `NixHTTPPathInfoService` and `store_verify_signatures`.

The gap is a function that scans `.narinfo` files in a directory, reads and
verifies each, ingests the referenced NAR, and persists the resulting PathInfo.

## What Changes

- **`crunch_store::pull` module.** Contains
  `import_paths_from_cache_dir(handle, source, options) -> PullReport`:
  scan `*.narinfo` files, parse each, resolve the NAR file path from the
  narinfo `URL` field, ingest the NAR into castore, construct PathInfo,
  verify signatures against trusted keys, and persist.
- **`crunch store pull` CLI subcommand.** Accepts `--from <path>` (source
  cache directory), optional positional store paths to filter, `--all` for
  everything, `--trusted-public-keys` for signature verification, and
  `--trust-unsigned` to import unsigned entries.
- **`PullReport` struct** with counts: imported, skipped (already present),
  skipped (untrusted signature), skipped (hash mismatch), total bytes
  ingested.

## Capabilities

### New Capabilities
- `store-pull-directory`: import PathInfo entries from a flat Nix binary
  cache directory layout (`.narinfo` + `.nar` files) into the local store.
- `cli-store-pull`: `crunch store pull` subcommand for importing cached
  build results.

### Modified Capabilities
- None. Push is unchanged. The pull module is a new peer to push.

## Impact

- **Files**: `crates/crunch-store/src/pull.rs` (new),
  `crates/crunch-store/src/lib.rs` (re-export), `src/store_cmd.rs` (CLI
  dispatch), `src/main.rs` (StoreAction variant).
- **APIs**: `import_paths_from_cache_dir()` as the library entry point.
  `PullReport`, `PullOptions`, `PulledPath` as public types.
- **Dependencies**: none new. `NarInfo::parse_with_store_dir` from
  nix-compat, `ingest_nar_and_hash` from snix-store, tokio::fs — all
  already in the dependency tree.
- **Testing**: unit test with push→pull round-trip verifying PathInfo
  matches; unit test for untrusted signature rejection; unit test for
  NAR hash mismatch; integration test building, pushing, pulling into
  a fresh store, and verifying the imported path.
