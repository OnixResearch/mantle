## Phase 1: Root cause and pure identity rules

- [x] [serial] I1 Record the exact retained Bison archive mismatch and bind it to the pre-rewrite versus final-NAR data flow. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
  - Evidence: `evidence/baseline.md` records the retained path, stale marker-normalized SHA-256, observed final SHA-256, exact old import error, and source diagnosis.
- [x] [serial] I2 Add pure CA-finalization metadata and CA-derived store-path identity helpers with explicit standard and marker-normalized forms. r[store_transports.archive_import_idempotent]
  - Evidence: `final_ca_path_info_facts` separates marker CA from final NAR facts; `ca_path_identity_matches` accepts exact marker-normalized or ordinary reference-aware derivations and rejects all other CA metadata.

## Phase 2: Builder and archive shells

- [x] [serial] I3 Recompute and persist final NAR size/SHA-256 after marker-to-final-path rewriting while preserving marker-normalized CA path identity. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
  - Evidence: single- and multi-output CA shells now render rewritten final nodes before PathInfo assembly while retaining marker hashes only in `PathInfo.ca`.
- [x] [serial] I4 Preflight archive export against fresh final NAR facts before writing any archive bytes. r[store_transports.archive_export_closure]
  - Evidence: closure planning fresh-renders every selected final node before `ARCHIVE_MAGIC`; the retained stale Bison state exits 3 with a zero-byte destination.
- [x] [serial] I5 Separate archive import CA path-identity validation from final payload/NAR/node validation and persist only after all checks pass. r[store_transports.archive_import_idempotent]
  - Evidence: import validates the CA-derived path independently, ingests without an expected CA content hash, and then requires payload BLAKE3, final NAR SHA-256/size, and exact node equality before persistence.

## Phase 3: Positive and negative proof

- [x] [serial] V1 Add pure and store-level positive tests for distinct marker CA/final NAR identities and negative tests for stale final NAR facts, wrong CA-derived paths, payload/NAR/node tampering, and no partial persistence/output. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
  - Evidence: focused CA/archive suites cover single and multi self-reference, real builder-to-archive round trip, marker and reference-aware CA forms, stale export/local-hit rejection, wrong/unsupported CA, truncated/tampered payloads, signature/prefix failures, and zero export bytes.
- [x] [serial] V2 Reproduce the old exact mismatch, prove corrected newly persisted CA metadata round-trips, and record the old-state fail-closed migration boundary. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
  - Evidence: `evidence/validation.md` records the retained-state zero-byte rejection and corrected builder-origin PathInfo archive round trip; existing signed stale facts remain rebuild-or-explicit-migration debt.
- [x] [serial] V3 Run focused archive/builder/CLI tests, Tiger Style, first-party quality, machine-contract, blocker, Cairn, Tracey, and full Nix gates. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
  - Evidence: `evidence/validation.md` records focused package/CLI tests, final first-party quality, 21/50 machine-contract freshness, zero blocker findings with 437 suppressions, valid Cairn, 145/145 Tracey coverage, Tiger Style success, and full Nix (`4048 passed`, `all checks passed!`).
- [x] [serial] V4 Run the authenticated offline full-source fixed-point proof from committed final source and bind exact stage identities. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
  - Evidence: pueue task `427` passed in 11,163.89 seconds from clean commit `8744faa7`; both stage binaries are BLAKE3 `eac97c4e997b3ab498be10156f6b58caebf812da6af4112f08e7ad57777f138c`, with 61 overrides per stage, zero live fetches, and strict stage2 with zero fallbacks.
- [ ] [serial] V5 Sync, inspect, archive, and commit the accepted requirements and exact evidence. r[store_transports.archive_export_closure] r[store_transports.archive_import_idempotent]
