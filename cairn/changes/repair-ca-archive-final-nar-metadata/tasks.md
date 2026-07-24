## Phase 1: Root cause and pure identity rules

- [ ] [serial] I1 Record the exact retained Bison archive mismatch and bind it to the pre-rewrite versus final-NAR data flow. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [ ] [serial] I2 Add pure CA-finalization metadata and CA-derived store-path identity helpers with explicit standard and marker-normalized forms. r[store_transports.archive_import_idempotent]

## Phase 2: Builder and archive shells

- [ ] [serial] I3 Recompute and persist final NAR size/SHA-256 after marker-to-final-path rewriting while preserving marker-normalized CA path identity. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [ ] [serial] I4 Preflight archive export against fresh final NAR facts before writing any archive bytes. r[store_transports.archive_export_closure]
- [ ] [serial] I5 Separate archive import CA path-identity validation from final payload/NAR/node validation and persist only after all checks pass. r[store_transports.archive_import_idempotent]

## Phase 3: Positive and negative proof

- [ ] [serial] V1 Add pure and store-level positive tests for distinct marker CA/final NAR identities and negative tests for stale final NAR facts, wrong CA-derived paths, payload/NAR/node tampering, and no partial persistence/output. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [ ] [serial] V2 Reproduce the old exact mismatch, prove corrected newly persisted CA metadata round-trips, and record the old-state fail-closed migration boundary. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [ ] [serial] V3 Run focused archive/builder/CLI tests, Tiger Style, first-party quality, machine-contract, blocker, Cairn, Tracey, and full Nix gates. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [ ] [serial] V4 Run the authenticated offline full-source fixed-point proof from committed final source and bind exact stage identities. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
- [ ] [serial] V5 Sync, inspect, archive, and commit the accepted requirements and exact evidence. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
