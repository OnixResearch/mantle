# Tasks

## Contract

- [x] [serial] Define the Mantle-native archive header, per-path record, metadata-before-payload ordering, store-prefix binding, named limit constants, and unsupported-version diagnostics. r[store_transports.streaming_archive_format]
- [x] [serial] Define recursive export selection, closure walk behavior, deterministic record ordering, missing-closure failure semantics, and unsigned-path policy. r[store_transports.archive_export_closure]
- [x] [serial] Define import action planning for skip/import/reject, bounded skipped-payload handling, trust policy checks, PathInfo persistence, sidecar preservation, and materialization mode. r[store_transports.archive_import_idempotent]
- [x] [serial] Define archive list semantics, read-only guarantees, non-seekable stream handling, and human/JSON metadata shape. r[store_transports.archive_list_inspection]
- [x] [serial] Define compatibility-claim policy for native Mantle archives versus fixture-proven external nario versions. r[store_transports.archive_compatibility_claims]

## Implementation

- [x] [serial] Implement pure archive core types and validators for headers, path records, feature flags, store-prefix matching, record ordering, list bounds, compatibility claims, and import action planning. r[store_transports.streaming_archive_format] r[store_transports.archive_import_idempotent] r[store_transports.archive_compatibility_claims]
- [x] [serial] Implement the export shell that resolves selectors, walks closure PathInfo, renders NAR payloads after metadata, preserves signatures/CA fields/attestation refs, and reports skipped or missing entries deterministically. r[store_transports.archive_export_closure]
- [x] [serial] Implement the import shell that reads archives from file/stdin, validates records, skips acceptable existing paths without reingest, ingests and verifies missing paths, persists signed PathInfo/sidecars, materializes outputs, and respects the store mutation lock. r[store_transports.archive_import_idempotent]
- [x] [serial] Implement the list shell that reads file/stdin, renders path metadata without mutation, and keeps payload skipping bounded for non-seekable streams. r[store_transports.archive_list_inspection]
- [x] [serial] Add `mantle store archive export|import|list` CLI help, human summaries, and JSON output without claiming external nario byte compatibility. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent] r[store_transports.archive_list_inspection] r[store_transports.archive_compatibility_claims]

## Verification

- [x] [serial] Add pure positive tests for deterministic export plans, valid header/record parsing, skip/import/reject planning, and archive list metadata extraction. r[store_transports.streaming_archive_format] r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent] r[store_transports.archive_list_inspection]
- [x] [serial] Add pure negative tests for unsupported versions, malformed record order, duplicate records, oversized fields/lists/chunks, prefix mismatch, unsigned strict-policy records, and unproven compatibility claims. r[store_transports.streaming_archive_format] r[store_transports.archive_import_idempotent] r[store_transports.archive_compatibility_claims]
- [x] [serial] Add store-level positive tests for recursive export/import round trip, existing-path skip without reingest, NAR metadata preservation, and read-only list behavior. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent] r[store_transports.archive_list_inspection]
- [x] [serial] Add store-level negative tests for hash mismatch, signature mismatch, wrong store prefix, truncated payload, unsupported CA metadata, missing closure facts, and non-seekable skipped-payload draining. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [x] [serial] Add CLI tests for `mantle store archive export|import|list` human output, JSON output, stdin/stdout streaming, strict unsigned behavior, and compatibility wording. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent] r[store_transports.archive_list_inspection] r[store_transports.archive_compatibility_claims]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation claims, then record focused implementation evidence before checking tasks complete. r[store_transports.streaming_archive_format]
