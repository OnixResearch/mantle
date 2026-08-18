## Phase 1: Manifest and release creation

- [x] [serial] Add optional release-manifest source acquisition metadata for external source archives. r[verification_evidence.independent_source_witness_replay]
  - Positive test: a manifest whose source acquisition digest equals `source_archive.digest_blake3` canonicalizes and verifies.
  - Negative test: a source acquisition digest that differs from `source_archive.digest_blake3` is rejected.
  - Evidence: `cairn/changes/independent-source-witness-replay/evidence/focused-validation-2026-06-29.md` (`cargo test -p crunch-release-core manifest::tests::`, 22 passed).
- [x] [serial] Add release creation CLI/request plumbing to record an external source archive URL. r[verification_evidence.independent_source_witness_replay]
  - Positive test: release evidence creation records the URL plus matching source digest.
  - Negative test: an empty/unsupported source acquisition URL is rejected before writing a manifest.
  - Evidence: `focused-validation-2026-06-29.md` (`release_evidence::tests::`, 15 passed; `release_create_accepts_source_acquisition_url_flag`, 1 passed).

## Phase 2: Witness rebuild gate

- [x] [serial] Add `mantle release witness-rebuild --require-independent-source` and plan-time fail-closed validation. r[verification_evidence.independent_source_witness_replay]
  - Positive test: a request with source acquisition metadata plans an independent-source replay.
  - Negative test: requiring independent source on a legacy request fails before workflow launch.
  - Evidence: `focused-validation-2026-06-29.md` (`witness_rebuild::tests::`, 30 passed; `release_witness_rebuild_accepts_require_independent_source_flag`, 1 passed).
- [x] [serial] Fetch, verify, and extract the external source archive before launching the workflow. r[verification_evidence.independent_source_witness_replay]
  - Positive test: a `file://` external archive with the expected BLAKE3 digest is used instead of the copied bundle archive.
  - Negative test: digest mismatch reports expected and fetched digests and does not extract or launch.
  - Evidence: `focused-validation-2026-06-29.md` (`prepare_scratch_fetches_independent_source_archive_when_required`, `source_acquisition_rejects_digest_mismatch_before_extraction`).
- [x] [serial] Record source acquisition status in witness audit metadata for success and prelaunch failure diagnostics. r[verification_evidence.independent_source_witness_replay]
  - Positive test: successful audit names independent-source mode, URL, digest, fetched path, and status.
  - Negative test: missing metadata fails before workflow launch; digest mismatch prelaunch audit names the source acquisition failure.
  - Evidence: `focused-validation-2026-06-29.md` (`source_acquisition_success_audit_records_verified_fetch`, `source_acquisition_prelaunch_failure_audit_records_error`).

## Phase 3: Validation and evidence

- [x] [serial] Run focused core, release evidence, and witness rebuild tests. r[verification_evidence.independent_source_witness_replay]
  - Evidence: exact command output recorded in `cairn/changes/independent-source-witness-replay/evidence/focused-validation-2026-06-29.md`.
- [x] [serial] Run Cairn validation and proposal/design/tasks gates. r[verification_evidence.independent_source_witness_replay]
  - Evidence: `cairn/changes/independent-source-witness-replay/evidence/cairn-validation-2026-06-29.md` records `cairn validate`, `cairn gate proposal`, `cairn gate design`, and `cairn gate tasks` output.
- [x] [serial] Record the bounded claim and non-claims for this slice. r[verification_evidence.independent_source_witness_replay]
  - Evidence: `focused-validation-2026-06-29.md` states this slice proves external source archive acquisition, not raw Git tag reconstruction.
