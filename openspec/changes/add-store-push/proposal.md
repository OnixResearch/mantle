## Why

crunch can substitute pre-built outputs from remote Nix binary caches but
cannot publish its own build results to one. Every machine that uses crunch
must either rebuild from source or depend on cache.nixos.org having the exact
path. This blocks CI/CD (build once, share everywhere), team workflows (one
builder, many consumers), and self-hosted caches.

All the building blocks already exist in the codebase:

- `PathInfo::to_narinfo()` renders a `NarInfo` struct with store path,
  nar hash/size, references, signatures, and CA metadata.
- `NarInfo` implements `Display`, producing the standard `.narinfo` text
  format that Nix consumers expect.
- `write_nar()` streams a castore node into the NAR archive format.
- `SigningKey::sign()` produces ed25519 signatures over the narinfo
  fingerprint.
- `StoreHandle` already exposes blob, directory, and pathinfo services.

The gap is a command that wires these together: iterate signed PathInfos,
render each as a `.narinfo` + `.nar` file pair, and write them to a
destination.

## What Changes

- **`crunch store push`** CLI subcommand. Accepts one or more store path
  selectors (or `--all`) and a `--to <dest>` target. Phase 1 supports a
  local directory target (flat file layout compatible with `nix-serve`,
  `harmonia`, nginx, S3 sync). HTTP PUT upload is a later extension.
- **`crunch_store::push` module.** Contains `export_paths_to_cache_dir()`:
  render narinfo + NAR for each selected PathInfo and write them to a
  directory. Writes `nix-cache-info` if absent.
- **`crunch_store::nar_render` helper.** Extracted from the test-only
  `render_nar_bytes()` into public API so push and future serve can share
  it.

## Capabilities

### New Capabilities
- `store-push-directory`: export signed PathInfo entries as a Nix binary
  cache directory layout (`.narinfo` + `.nar` files + `nix-cache-info`).
- `cli-store-push`: `crunch store push` subcommand for publishing build
  results.

### Modified Capabilities
- `store-query`: `StoreHandle` gains a public NAR rendering method.

## Impact

- **Files**: `crates/crunch-store/src/push.rs` (new),
  `crates/crunch-store/src/lib.rs` (re-export), `src/store_cmd.rs` (CLI
  dispatch), `src/main.rs` (StoreAction variant).
- **APIs**: `StoreHandle::render_nar()` promoted from test helper to
  public method. `export_paths_to_cache_dir()` as the library entry point.
- **Dependencies**: none new. `write_nar` from snix-store, `NarInfo`
  display from nix-compat, `tokio::fs` for async file writes — all
  already in the dependency tree.
- **Testing**: unit test with an in-memory store verifying narinfo
  content and NAR bytes round-trip; integration test building a path
  then pushing it and verifying `nix-store --verify-path` on the result.
