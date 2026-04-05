## Why

Every `crunch build` rebuilds everything from source. A derivation whose
output already exists on cache.nixos.org still gets built locally. This
makes crunch impractical for anything beyond trivial examples — building
a stdenv package pulls in bash, coreutils, gcc, glibc, each taking
minutes, when cached NARs are available in seconds.

snix-store already has `NixHTTPPathInfoService` which fetches `.narinfo`,
downloads + decompresses the NAR, ingests it into castore, and returns
a `PathInfo`. The machinery exists. crunch just doesn't call it.

## What Changes

- **`check_cache` gains a remote fallback.** After the local pathinfo
  miss, query a remote `NixHTTPPathInfoService`. On hit: NAR is
  downloaded, ingested into castore, PathInfo persisted locally, output
  node cached in `output_nodes`. The build is skipped.
- **CLI `--substituters` flag.** Comma-separated list of cache URLs
  (default: `https://cache.nixos.org`). Passed into Builder construction.
- **CLI `--no-substitute` flag.** Disables remote lookup entirely.
- **`Builder` holds an optional remote PathInfoService.** Only
  constructed when substituters are configured. Typed as
  `Option<Arc<dyn PathInfoService>>` to avoid infecting the generic
  params.

The local pathinfo (redb) remains the write-through cache. A remote
hit writes into redb so subsequent builds for the same path never
hit the network again.

## Capabilities

### New Capabilities
- `binary-cache-substitution`: fetch pre-built outputs from remote
  Nix binary caches instead of building locally.
- `cli-substituter-config`: configure cache URLs and disable
  substitution from the command line.

### Modified Capabilities
- `cache-check`: `check_cache` gains a remote fallback path after
  local miss.

## Impact

- **Files**: `crates/crunch-build/src/orchestrate.rs` (Builder,
  check_cache), `src/main.rs` (CLI flags, NixHTTPPathInfoService
  construction).
- **APIs**: `Builder::new` / `Builder::with_state_dir` gain an
  optional remote pathinfo param. Existing callers pass `None`.
- **Dependencies**: `reqwest` (already in snix-store), `url` crate
  (already in snix-store). No new deps.
- **Testing**: unit test with mock HTTP server returning narinfo +
  NAR, integration test verifying cache hit skips build.
