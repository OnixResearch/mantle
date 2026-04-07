## Why

The fetcher is a special case in the orchestrator. `Builder::prepare_build()`
checks `is_builtin_fetcher(derivation)` and branches into
`build_fetcher()` — a 100+ line method that downloads, verifies,
ingests, and persists, all inside the Builder struct. The orchestrator
knows about URLs, tarballs, git clones, and hash verification.

snix's architecture uses `BuildService` as the abstraction boundary.
The orchestrator calls `build_service.do_build(request)` and gets back
a `BuildResult`. It doesn't care whether the build ran in bwrap, OCI,
WASM, or was a fetch. The dispatcher is the same for all build types.

crunch's `Builder::prepare_build()` has two paths: the normal sandbox
path (which goes through `BuildService::do_build`) and the fetcher path
(which bypasses it entirely). This means:

- The orchestrator has fetch-specific knowledge it shouldn't need
- Adding a new fetch type (e.g., fetchCargo, fetchNpm) requires modifying
  the Builder
- Fetch caching and fetch retry logic can't be tested without a full
  Builder
- The `BuildService` trait is undermined — it's not the universal
  dispatch point the architecture says it should be

## What Changes

Wrap the fetcher in a `BuildService` implementation:

1. **`FetchBuildService`**: implements `BuildService::do_build()`. When
   called with a fetch request encoded in the existing `BuildRequest`
   fields (`command_args[0] == "builtin:fetchurl"` plus fetch parameters
   in `environment_vars`), it performs the download and extraction,
   ingests the result into castore, and returns a `BuildResult` with the
   output node.
2. **`DispatchBuildService`**: a composite BuildService that checks
   `command_args[0]` and delegates to either `FetchBuildService` or the
   underlying sandbox BuildService. The orchestrator only ever calls
   `dispatch.do_build()`.
3. **Remove `is_builtin_fetcher` check from `prepare_build()`** — the
   orchestrator treats all derivations the same. The dispatch happens
   inside the BuildService layer.
4. **Move fetcher hash verification into the shared post-build path** —
   `finish_build()` verifies flat and recursive fixed-output hashes,
   persists `PathInfo`, and handles mismatch cleanup for both fetchers
   and sandbox builds.

## Capabilities

### New Capabilities
- `FetchBuildService`: BuildService implementation for fetcher derivations
- `DispatchBuildService`: composite that routes fetch vs sandbox builds

### Modified Capabilities
- `Builder::prepare_build()`: no fetcher special case
- `Builder`: no longer owns fetch logic

### Removed Capabilities
- `Builder::build_fetcher()`: replaced by FetchBuildService

## Impact

- **Files**: new files `fetch_build_service.rs`, `dispatch_build_service.rs`
  in `crates/crunch-build/src/`; modified `orchestrate.rs` (removed
  `build_fetcher`), `fetcher.rs` (pub(crate) helpers), `lib.rs`
  (module registration)
- **APIs**: BuildService remains the same; new implementations added
- **Dependencies**: none
- **Testing**: FetchBuildService and DispatchBuildService testable
  independently with mock HTTP and mock sandbox services
