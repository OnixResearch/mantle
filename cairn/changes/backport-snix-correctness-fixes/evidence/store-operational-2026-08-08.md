# Store services and operational alignment

## Store services

The redb batch putter now owns an `Arc` to the database. It creates the write transaction, opens the table, mutates the table, and commits inside one blocking worker.

`PathInfoCache::list()` and `list_with_layer()` now delegate only to the writable near service. They do not enumerate or backfill the far service.

Tests cover successful batch writes, read-only transaction errors, concurrent batch callers, successful near writes and listing, empty near state with far-only data, and a far service that panics if listed.

## Ingestion and tracing

Filesystem ingestion now uses `copy_buf` with the default bounded `BufReader`. It does not add the upstream fixed 128 KiB capacity because this change has no current evidence that the larger fixed size is required.

Tracing now combines configured and additional layers before it applies the environment filter. A deterministic test proves that both layers receive enabled events and both reject disabled events.

Tests cover normal files, empty files, short reads, injected copy errors before finalization, enabled tracing events, disabled tracing events, and additional-layer visibility.

## Deferred-trigger recheck

The virtiofs used-length fix stays deferred. No non-vendor manifest enables the affected route, and no local used-length call exists.

The read-only redb builder fix stays deferred. Both local read-only services use the default builder directly and do not apply non-default builder configuration.

The upstream ledger now maps CL 31157 to filesystem ingestion and CL 31150 to tracing.

## Focused validation

```text
cargo test -p snix-castore --lib directoryservice::redb::tests
6 passed; 0 failed

cargo test -p snix-store --lib pathinfoservice::cache::test
4 passed; 0 failed

cargo test -p snix-castore --lib import::fs::tests
3 passed; 0 failed

cargo test -p snix-tracing --lib
1 passed; 0 failed

git diff --check
Finished successfully.
```
