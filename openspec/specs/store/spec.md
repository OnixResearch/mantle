# Store Crate Specification

## Purpose

Defines the `crunch-store` crate: a standalone module owning all
store operations — service construction, caching, realization, queries,
and CA mapping persistence.

## ADDED Requirements

### Requirement: crunch-store crate

The workspace MUST contain a `crunch-store` crate that owns all
store-level operations. No other crate may directly instantiate
BlobService, DirectoryService, or PathInfoService implementations.

#### Scenario: Store crate compiles independently

- GIVEN the `crunch-store` crate
- WHEN `cargo build -p crunch-store` is run
- THEN it compiles without depending on crunch-eval, crunch-glue,
  or crunch-build

### Requirement: StoreHandle

The crate MUST provide a `StoreHandle` struct (or trait) that bundles
access to blob, directory, pathinfo, and optional remote pathinfo
services. Consumers receive a `StoreHandle` — they do not construct
or own individual services.

#### Scenario: Builder receives StoreHandle

- GIVEN a `Builder` configured with a `StoreHandle`
- WHEN the Builder needs to persist a PathInfo
- THEN it calls `store.put_pathinfo(pi)`, not `self.pathinfo_service.put(pi)`

#### Scenario: StoreHandle constructed from config

- GIVEN a state directory path and optional remote cache URL
- WHEN `StoreHandle::open(config)` is called
- THEN blob, directory, pathinfo, and remote services are initialized
- AND the handle is ready for use

### Requirement: CacheService trait

The crate MUST define a `CacheService` trait for cache checking:

```rust
#[async_trait]
pub trait CacheService {
    async fn check(
        &self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
    ) -> Result<Option<CacheHit>, Error>;
}
```

The implementation MUST:
1. Check local PathInfo + castore content probes
2. Fall back to remote binary cache on local miss
3. Write-through remote hits to local storage
4. Skip remote queries for FODs and unresolved CA derivations

#### Scenario: Local cache hit

- GIVEN PathInfo and castore content exist locally
- WHEN `cache.check()` is called
- THEN `Some(CacheHit)` is returned with no network call

#### Scenario: Remote substitution

- GIVEN no local PathInfo, but remote narinfo exists
- WHEN `cache.check()` is called
- THEN the NAR is fetched, PathInfo persisted locally, `Some(CacheHit)` returned

#### Scenario: Full miss

- GIVEN no local or remote PathInfo
- WHEN `cache.check()` is called
- THEN `None` is returned

### Requirement: RealizationService trait

The crate MUST define a `RealizationService` trait for exporting
castore content to the filesystem:

```rust
#[async_trait]
pub trait RealizationService {
    async fn export(
        &self,
        node: &Node,
        dest: &str,
    ) -> Result<(), Error>;
}
```

The implementation MUST:
- Write files, directories, and symlinks to disk
- Handle read-only filesystems gracefully (warn, don't fail)
- Enforce a depth limit on directory trees

#### Scenario: Export to writable store

- GIVEN a castore node and a writable output directory
- WHEN `realize.export(node, path)` is called
- THEN the files appear on disk at the given path

#### Scenario: Export to read-only store

- GIVEN a castore node and a read-only output directory
- WHEN `realize.export(node, path)` is called
- THEN a warning is logged and no error is returned

### Requirement: Store query operations

The crate MUST provide functions for:
- `list()` — stream all PathInfo records
- `info(path)` — detailed PathInfo for a specific path
- `verify(path)` — re-compute NAR hash and compare with stored value

These MUST NOT live in the binary crate.

#### Scenario: List from library

- GIVEN a StoreHandle
- WHEN `store.list()` is called from library code (not CLI)
- THEN all PathInfo records are streamed

### Requirement: CA mapping persistence

The `CaMappings` struct MUST move to crunch-store. It tracks
derivation→output path mappings for content-addressed derivations
across process restarts.

#### Scenario: CA mapping survives restart

- GIVEN a CA derivation built in session 1
- WHEN session 2 starts and checks cache
- THEN the CA mapping resolves the final output path without rebuilding

### Requirement: Native closure resolution

The system MUST resolve runtime closures using its own data, not
by shelling out to `nix-store -qR`.

For each source input that needs closure resolution:

1. Check local PathInfo (redb) for the `references` field
2. If not in local PathInfo, query the binary cache narinfo
3. Walk references transitively until the full closure is collected
4. If neither source is available, use the declared path only
   (no closure) and log a warning

#### Scenario: Crunch-built input closure from PathInfo

- GIVEN a crunch-built library `libfoo` with PathInfo in redb
- AND PathInfo.references lists `libbar` and `glibc`
- WHEN a derivation depends on `libfoo`
- THEN `libfoo`, `libbar`, and `glibc` are all mounted in the sandbox

#### Scenario: Nix seed input closure from narinfo

- GIVEN a Nix seed path `/nix/store/...-bash-5.2`
- AND the binary cache narinfo for that path lists 3 references
- WHEN a derivation depends on bash
- THEN bash and its 3 transitive references are mounted

#### Scenario: No closure data available

- GIVEN a seed path with no local PathInfo and no narinfo
- WHEN a derivation depends on it
- THEN only the declared path is mounted
- AND a warning is logged

#### Scenario: Static input needs no closure

- GIVEN a statically-linked binary from `crunch bootstrap --fetch`
- WHEN a derivation depends on it
- THEN PathInfo.references is empty
- AND only the declared path is mounted (correct behavior)

### Requirement: Cycle-safe closure walking

The closure walker MUST detect cycles in the reference graph and
terminate. Store paths with circular references MUST NOT cause
infinite loops.

#### Scenario: Circular reference

- GIVEN path A references B and B references A
- WHEN closure resolution runs for A
- THEN both A and B are included, and the walk terminates

### Requirement: Closure depth limit

The closure walker MUST enforce a maximum transitive depth (1024).
Paths beyond this depth MUST be skipped with a warning.

#### Scenario: Deep closure

- GIVEN a path with a transitive closure deeper than 1024
- WHEN closure resolution runs
- THEN the first 1024 levels are included
- AND a warning is logged about the depth limit

## MODIFIED Requirements

### Requirement: Builder service ownership (modified)

The `Builder` MUST NOT own BlobService, DirectoryService, or
PathInfoService directly. It MUST receive a `StoreHandle` and call
store operations through it.

The `Builder` struct MUST have at most two generic parameters:
the `BuildService` and the `StoreHandle`. Not four.

#### Scenario: Builder type signature

- GIVEN the Builder struct definition
- WHEN inspected
- THEN it has `Builder<BServ, Store>`, not `Builder<BS, DS, BServ, PIS>`
