# Implementation evidence — Store archive transport

Date: 2026-06-30

## Implemented

- Added `crates/crunch-store/src/archive.rs` with a Mantle-native `mantle-store-archive-v1` streaming archive envelope.
- Added pure import-action planning, header/frame validation, store-prefix binding, deterministic closure planning, duplicate detection, named limits, native-format compatibility wording, and BLAKE3 payload digests.
- Added store-backed export/list/import functions that render NAR payloads after metadata, verify BLAKE3/SHA-256/NAR size/node identity on import, verify trusted signatures unless `trust_unsigned`, skip already-present paths, persist PathInfo, and materialize imported outputs.
- Added fail-closed handling for conflicting local PathInfo metadata before skip decisions.
- Added header/end record-count validation so malformed archives cannot list/import with mismatched declared counts.
- Added `mantle store archive export|import|list` CLI wiring with human and JSON report support and stdin/stdout archive stream support via `-`.

## Verification coverage added

- Archive unit tests cover recursive export/import, deterministic closure ordering, metadata-before-payload listing, existing-path skip, strict unsigned export rejection, bad magic, header/end count mismatch, store-prefix mismatch, untrusted signatures, tampered payload, truncated payload, unsupported CA metadata, conflicting local PathInfo, missing closure facts, and non-seekable bounded payload draining.
- `tests/store_archive_cli.rs` covers CLI human output, JSON output, stdin/stdout streaming, import skip reports, strict unsigned behavior, and native nario-compatibility wording.

## Current evidence

See `completion-2026-06-30.md` for the final command transcripts and passing results.
