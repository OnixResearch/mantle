## Why

Store operations are scattered across `main.rs` (service construction,
store queries), `orchestrate.rs` (cache checking, PathInfo persistence,
castore content probes, disk export), `export.rs` (castore→filesystem),
and `ca_mapping.rs` (persistent CA mapping). There is no `crunch-store`
crate. The `Builder` god struct reaches directly into BlobService,
DirectoryService, and PathInfoService.

This means:

- `Builder` is 2700 lines because it owns and operates all store services
- Store initialization logic lives in `main.rs` free functions, unreachable
  as a library
- Cache logic, realization, and store queries can't be tested independently
- Adding GC or store composition requires touching the Builder internals
- No trait boundary between the build engine and the store — the Builder
  calls `.put()`, `.get()`, `.open_read()` directly on concrete services

snix has separate `snix-store` and `snix-castore` crates with clean trait
boundaries. crunch vendors them but treats them as internal implementation
details of a single struct.

## What Changes

Extract a `crunch-store` crate that owns:

1. **Service initialization** — constructing the blob, directory, and
   pathinfo services from config (state dir, backend selection)
2. **Cache checking** — local PathInfo + castore content probes, remote
   binary cache fallback
3. **Realization** — exporting castore nodes to the filesystem
4. **Store queries** — list, info, verify operations
5. **CA mapping persistence** — the JSON-based drv→output path mapping

The `Builder` takes a `StoreHandle` (or trait) instead of four separate
service generics. Store operations become method calls on the handle,
not inline code in the orchestrator.

## Capabilities

### New Capabilities
- `crunch-store`: standalone crate for all store operations
- `StoreHandle`: unified interface to blob + directory + pathinfo + cache
- `CacheService`: trait for cache checking (local + remote substitution)
- `RealizationService`: trait for castore→disk export

### Modified Capabilities
- `Builder`: takes `StoreHandle` instead of `BS, DS, BServ, PIS` generics
- `main.rs`: delegates service construction to `crunch-store`

## Impact

- **Files**: new `crates/crunch-store/`, modified `crates/crunch-build/`,
  modified `src/main.rs`
- **APIs**: Builder's type signature changes; store operations move behind
  traits
- **Dependencies**: `crunch-build` depends on `crunch-store` (not on
  `snix-castore`/`snix-store` directly)
- **Testing**: store operations become independently testable with mock
  implementations
