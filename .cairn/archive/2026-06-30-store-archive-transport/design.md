# Design: Store archive transport

## Architecture

The archive transport is a store feature, not a frontend feature. It sits beside the existing push/pull binary-cache paths and reuses Mantle's StoreHandle, PathInfo service, NAR renderer/ingester, artifact attestation helpers, and store mutation lock. The new archive code should be split into a pure planning/validation core and a thin I/O shell:

- Pure core: archive header validation, record ordering, record limit checks, store-prefix compatibility decisions, import action planning, skip-existing decisions from supplied local-state facts, compatibility-claim classification, and list rendering data.
- Shell: stdin/stdout/file reads and writes, NAR rendering/ingest, PathInfo service iteration and persistence, artifact sidecar reads/writes, store locks, output materialization, CLI argument parsing, and human/JSON output.

Archive parsing should be streaming. The importer reads a bounded header, then bounded per-path metadata records before payload streams. A path whose metadata proves it is already present and acceptable locally can be skipped without buffering the payload in memory. If the underlying stream cannot seek, the shell drains skipped payload bytes in bounded chunks without storing them.

## Format shape

Use a Mantle-owned format name and version such as `mantle-store-archive-v1`. The exact wire encoding can be postcard, CBOR, or another explicit binary envelope, but it must provide:

- a magic/domain separator and format version;
- archive-level store prefix and producer identity fields;
- deterministic root selector and path-record ordering;
- per-path metadata before payload bytes;
- bounded strings, lists, metadata bytes, and payload chunk sizes via named constants;
- PathInfo metadata including store path, references, NAR size/hash, signatures, deriver, CA field, and node identity;
- optional artifact/closure attestation refs or embedded sidecars when that support is implemented;
- a per-record payload length and digest evidence;
- a deterministic end marker or archive summary digest.

Use BLAKE3 for Mantle-owned archive summary or frame checksums. Preserve the existing PathInfo NAR hash fields required by Nix/Snix interoperability rather than replacing them.

## Export flow

`mantle store archive export` resolves selectors to PathInfo entries, walks the recursive closure through local PathInfo/closure facts, and builds a deterministic export plan. Missing closure facts should fail closed by default. A later explicit practical mode may record degraded closure evidence, but the first implementation should avoid silently shipping partial closures.

Export writes metadata before each NAR payload. If a selected path lacks a required signature and the operator has not selected an explicit unsigned escape hatch, export skips or fails according to the CLI mode and reports the unsigned paths. Root labels, recursive closure members, and emitted bytes should be reported in JSON and human summaries.

## Import flow

`mantle store archive import` validates archive version and store prefix before persistence. For each record it plans one of three actions from metadata and local-state facts:

1. skip because acceptable PathInfo and content already exist;
2. import by ingesting payload, verifying digest and signature policy, persisting PathInfo/sidecars, and materializing output when requested;
3. reject with deterministic diagnostics.

Import must never accept a record only because it appears in the archive. It verifies PathInfo signatures unless an explicit trust escape hatch is selected, rejects store-prefix mismatches, rejects content hash mismatches, rejects unsupported CA metadata, and preserves signed PathInfo invariants.

## List flow

`mantle store archive list` parses only archive and per-path metadata required to show contents. It does not mutate store state, does not materialize outputs, and does not require loading whole payloads. For non-seekable streams, it may drain payload bytes in bounded chunks to reach later records, but it must not buffer those payloads.

List output should include store path, NAR size, references count, signature names or count, CA marker when present, root membership, and enough archive metadata to diagnose store-prefix or compatibility problems.

## Compatibility posture

The format is nario-v2-inspired, not automatically nario-v2-compatible. Any `--format nario-v2` or compatibility statement must be gated by checked fixtures from an implementation known to produce that format. Without such fixtures, the CLI should name the native Mantle format and report Nix nario v2 byte compatibility as unsupported rather than guessed.

## Validation strategy

- Pure core positive tests for deterministic export plans, import skip/import/reject planning, limit acceptance at named boundaries, and list metadata extraction.
- Pure core negative tests for unsupported versions, prefix mismatch, duplicate records, malformed ordering, oversized metadata/list counts, missing root records, unsigned records under strict policy, and unsupported compatibility claims.
- Store-level positive tests for export/import round trip, recursive closure preservation, skip-existing import that does not reingest payload content, and list without mutation.
- Store-level negative tests for hash mismatch, signature mismatch, tampered metadata, truncated payload, unsupported CA metadata, missing closure facts, and non-seekable skipped payload draining.
- CLI tests proving stdout/stderr/JSON behavior for export, import, list, and failure summaries.
