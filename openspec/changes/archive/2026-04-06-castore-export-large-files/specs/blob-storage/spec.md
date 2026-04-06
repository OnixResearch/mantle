# Blob Storage Specification

## Purpose

Defines requirements for the blob service used in production crunch builds.
Replaces the implicit "everything in RAM" model with persistent, disk-backed
blob storage.

## Requirements

### Requirement: Disk-Backed Blob Storage

The production blob service MUST store blob data on the local filesystem,
not in process memory.

#### Scenario: Large build output persists

- GIVEN a build that produces a 50 MiB binary output
- WHEN the build completes and the process exits
- THEN the blob data is present on disk in the blob storage directory

#### Scenario: Blob survives process restart

- GIVEN a previously-built derivation with PathInfo in redb and blobs on disk
- WHEN a new crunch process starts and checks the cache
- THEN `castore_has_content` returns true without rebuilding

### Requirement: Blob Storage Location

The blob storage directory MUST be `{state_dir}/blobs/` where `state_dir`
is `~/.local/state/crunch/` (the same directory containing `pathinfo.redb`).

#### Scenario: Default location

- GIVEN no explicit configuration
- WHEN crunch initializes the blob service
- THEN blobs are stored under `~/.local/state/crunch/blobs/`

#### Scenario: Directory auto-creation

- GIVEN the blob storage directory does not exist
- WHEN crunch initializes the blob service
- THEN the directory is created automatically

### Requirement: Bounded Memory Usage

The blob service MUST NOT accumulate all blob data in process memory.
Memory usage for blob I/O SHOULD be proportional to the I/O buffer size
(64 KiB), not to total blob data size.

#### Scenario: Memory stays bounded during large build

- GIVEN a build producing 500 MiB of output files
- WHEN the output is ingested into the blob service
- THEN process RSS growth is less than 100 MiB above baseline
  (not 500 MiB as with MemoryBlobService)

### Requirement: Content-Addressed Deduplication

The blob service MUST deduplicate blob chunks by content hash. Identical
file content across different builds MUST be stored once.

#### Scenario: Shared library dedup

- GIVEN two derivations that both produce the same 10 MiB shared library
- WHEN both builds complete
- THEN the blob storage directory contains one copy of each unique chunk,
  not two full copies

### Requirement: Test Isolation

Unit and integration tests in `crunch-build` MUST continue using
`MemoryBlobService`. Tests MUST NOT write to `~/.local/state/crunch/`.

#### Scenario: Test blob service is in-memory

- GIVEN any test in `crates/crunch-build/`
- WHEN the test creates a `Builder`
- THEN the blob service is `MemoryBlobService`, not `ObjectStoreBlobService`

### Requirement: Bootstrap Blob Persistence

The `crunch bootstrap --fetch` command MUST use the same persistent blob
service as `crunch build`, so fetched tarballs are cached across bootstrap
and build invocations.

#### Scenario: Bootstrap fetch reuses blobs

- GIVEN `crunch bootstrap --fetch` has fetched a tarball
- WHEN `crunch build` runs a derivation using the same tarball content
- THEN the blob data is already present (cache hit), no re-download

## REMOVED Requirements

(none)
