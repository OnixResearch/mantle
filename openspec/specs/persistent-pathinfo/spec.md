# Persistent PathInfoService Specification

## Purpose

Defines how build output metadata (PathInfo) is persisted across
process restarts, enabling accurate cache checks, store queries,
and CA derivation lookup.

## Requirements

### Requirement: PathInfo persisted after build

After a successful build, the system MUST persist a `PathInfo`
record for each output via `PathInfoService::put()`. The record
MUST contain:

- `store_path`: the output store path
- `node`: the castore root node (file/directory/symlink)
- `references`: runtime references found via refscan
- `nar_size`: NAR serialization size in bytes
- `nar_sha256`: SHA-256 hash of the NAR serialization
- `deriver`: the derivation store path that produced this output
- `ca`: the `CAHash` if this is a fixed-output or CA derivation
- `signatures`: empty for v1 (signing is future work)

#### Scenario: Build persists PathInfo

- GIVEN a successful build of derivation "hello"
- WHEN the build completes
- THEN `PathInfoService::get(digest)` returns the PathInfo
- AND the PathInfo survives process restart

#### Scenario: Cached outputs retain PathInfo

- GIVEN a previously built output with PathInfo in the database
- WHEN `mantle build` is run again and the output is cached
- THEN the existing PathInfo is returned without re-ingesting

### Requirement: Cache check uses PathInfoService and castore

The cache check MUST query `PathInfoService::get()` using the
output path's 20-byte digest. A cache hit requires BOTH:

1. `PathInfoService::get(digest)` returns `Some(path_info)`
2. The content referenced by `path_info.node` exists in the
   castore (blob service for files, directory service for dirs)

The cache check MUST NOT use filesystem existence checks. Build
outputs live in the castore; only root outputs requested by the
user are exported to disk (see castore-store spec). Intermediate
dependency outputs may never exist on disk.

If PathInfo exists but the castore content is missing, the system
MUST treat it as a cache miss and rebuild. It SHOULD log a warning
that the store is inconsistent.

If no PathInfo exists, the system MUST treat it as a cache miss
and rebuild.

#### Scenario: PathInfo + castore content = cache hit

- GIVEN PathInfo for `/nix/store/<hash>-hello` in the database
- AND the blob referenced by PathInfo.node exists in the blob service
- WHEN `mantle build` processes this derivation
- THEN the build is skipped (cache hit)
- AND `output_nodes` is populated from PathInfo for downstream use

#### Scenario: PathInfo but missing castore content = cache miss

- GIVEN PathInfo for `/nix/store/<hash>-hello` in the database
- BUT the blob referenced by PathInfo.node is absent from the blob service
- WHEN `mantle build` processes this derivation
- THEN the derivation is rebuilt
- AND a warning is logged about missing castore content

#### Scenario: No PathInfo = cache miss

- GIVEN no PathInfo in the database for an output
- WHEN `mantle build` processes this derivation
- THEN the derivation is rebuilt

### Requirement: Database location

The PathInfo database MUST be stored at a well-known location:

1. `$CRUNCH_STATE_DIR/pathinfo.redb` if `CRUNCH_STATE_DIR` is set
2. `$XDG_STATE_HOME/crunch/pathinfo.redb` if `XDG_STATE_HOME` is set
3. `$HOME/.local/state/crunch/pathinfo.redb` otherwise

The directory MUST be created automatically if it doesn't exist.

#### Scenario: First run creates database

- GIVEN no prior mantle state directory
- WHEN `mantle build` runs for the first time
- THEN the state directory and redb file are created

#### Scenario: Custom state dir

- GIVEN `CRUNCH_STATE_DIR=/tmp/mantle-test`
- WHEN `mantle build` runs
- THEN PathInfo is stored in `/tmp/mantle-test/pathinfo.redb`

### Requirement: RedbPathInfoService backend

The system MUST use `RedbPathInfoService` from vendored snix-store
as the storage backend. The database MUST be opened in read-write
mode.

The `RedbPathInfoService` stores PathInfo as protobuf-encoded bytes
keyed by the 20-byte output path digest. This is already implemented
in the vendored code.

#### Scenario: Database survives restart

- GIVEN a build that persists PathInfo to redb
- WHEN the mantle process exits and restarts
- THEN `PathInfoService::get()` returns the previously stored PathInfo

### Requirement: Builder accepts PathInfoService

`Builder::new()` MUST accept a `PathInfoService` implementation
(as a trait object or generic parameter). The Builder uses it for:

- Cache checks (before building)
- Persisting PathInfo (after building)
- Providing PathInfo for cached outputs (instead of re-ingesting
  from disk)

#### Scenario: Cache hit avoids re-ingest

- GIVEN a cached output with PathInfo in the database
- WHEN the Builder processes it
- THEN it returns the stored PathInfo directly, without calling
  `ingest_path` or `calculate_nar`

### Requirement: Store query subcommand

The CLI MUST provide a `mantle store` subcommand with:

- `mantle store list` — list all known store paths with name,
  deriver, NAR size
- `mantle store info <path>` — show full PathInfo for a store
  path (references, NAR hash, deriver, CA info)
- `mantle store verify [<path>]` — re-compute NAR hash and
  compare with stored value. Report mismatches.

#### Scenario: List known paths

- GIVEN three paths in the PathInfo database
- WHEN `mantle store list` is run
- THEN all three are printed with name, deriver, and NAR size

#### Scenario: Verify detects corruption

- GIVEN a stored PathInfo with NAR hash H
- AND the output on disk has been modified
- WHEN `mantle store verify` is run
- THEN the mismatch is reported

### Requirement: Concurrency safety

The redb database MUST support concurrent reads from multiple
processes. Write access MUST be serialized (redb provides this
via its write transaction model).

A second `mantle build` invocation while the first is running
MUST NOT corrupt the database. It MAY block on write transactions
(acceptable for v1).

#### Scenario: Concurrent reads

- GIVEN `mantle build` is running
- WHEN `mantle store list` is run simultaneously
- THEN the list command succeeds without blocking the build

### Requirement: Graceful degradation

The system MUST degrade gracefully when the PathInfo database cannot be
opened due to permissions or corruption by falling back to the v0 behavior
(filesystem-only cache checks, in-memory PathInfo). It MUST log a warning.

#### Scenario: Corrupt database

- GIVEN a corrupted `pathinfo.redb` file
- WHEN `mantle build` attempts to open it
- THEN a warning is logged
- AND builds proceed with in-memory PathInfo (no persistence)
