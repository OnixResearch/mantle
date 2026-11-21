# Tasks: Admit a Trellis-owned Verus producer job in the verification bridge

## Phase 1: Producer job observation and binding

- [x] [serial] T1.1 Add the `trellis-job OWNER_OUTPUT PRODUCER_EXPORT SOURCE_REVISION OUTPUT` CLI mode and `consume_trellis_job` with a profile-derived material plan. r[verification_bridge.trellis_job_mode]
- [x] [serial] T1.2 Add `observe_trellis_job` with the three schema checks, profile/provider/source/artifact/receipt links, and the producer-report identity. r[verification_bridge.trellis_job_mode]
- [x] [serial] T1.3 Add `bind_trellis_job` by parameterizing the shared producer job binder with expected schema, owner, and receipt rail field. r[verification_bridge.trellis_job_mode]
- [x] [serial] T1.4 Add the new stable error variants and diagnostics. r[verification_bridge.trellis_job_mode]

## Phase 2: Verification

- [x] [serial] T2.1 Add the synthetic positive observation and export-replay tests. r[verification_bridge.trellis_job_mode]
- [x] [serial] T2.2 Add the negative tests for mutated profile/receipt/report, wrong job schema/owner/rail, network allowed, graph root/closure, revision, tampered bytes, and missing report/evidence. r[verification_bridge.trellis_job_mode]
- [x] [serial] T2.3 Keep the recorded `octet`, `octet-verified`, `trellis`, and `octet-job` tests passing unchanged. r[verification_bridge.trellis_job_mode]
- [x] [serial] T2.4 Run focused tests, clippy, fmt, and `./scripts/check-no-std-core.sh`; record the transcripts and the step-12 `origin/main` baseline. r[verification_bridge.trellis_job_mode]
- [x] [serial] T2.5 Record ADR 0168, update the bridge document, and run `cairn validate` plus the proposal, design, and tasks gates. r[verification_bridge.trellis_job_mode]
