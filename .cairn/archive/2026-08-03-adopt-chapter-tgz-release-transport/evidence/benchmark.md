# Representative benchmark

Date: 2026-08-03

## Fixture

The ignored benchmark uses a verified release-evidence fixture with a deterministic 64 MiB source archive. The transport has five chapters and a 67,121,685-byte compressed archive.

Each reported value is the median of three samples. The test uses a test-only `IndependentRead` implementation backed by `FileExt::read_at`.

## Command

```text
/home/brittonr/.cargo-target/debug/deps/mantle-d1c7829ae6533b75 representative_release_transport_benchmark --ignored --nocapture --test-threads=1
```

## Result

```json
{"archive_bytes":67121685,"chapter_count":5,"control_chapter_access_median_micros":108,"generic_gzip_read_median_micros":97851,"inspect_median_micros":432582,"kind":"mantle-release-chapter-transport-benchmark-v1","pack_median_micros":6835366,"parallel_chapter_read_median_micros":7962,"payload_bytes":67114274,"sample_count":3,"sequential_chapter_read_median_micros":8678,"source_bytes":67108864,"unpack_median_micros":484988}
```

The sequential and parallel chapter paths returned the same 67,114,274 payload bytes.

Parallel chapter reads improved the median from 8,678 microseconds to 7,962 microseconds. This is an 8.3 percent improvement.

## Decision

Production parallel decompression remains disabled. Its activation threshold is a median improvement of at least 20 percent on representative bundles, with equal output bytes and no weaker validation.

The current chapter plan keeps the large source archive in one chapter. This limits useful parallel work. A later source-bundle format can revisit chapter granularity with separate evidence.
