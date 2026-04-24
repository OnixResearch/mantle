# Tasks: cross-machine witness handoff

## Phase 1: Export portable witness-request directories

- [x] I1 Add a pure request-layout core that derives deterministic export
      paths and metadata for `request.json`, the copied release-evidence
      bundle, and the copied release-verification seed directory from the
      verified release identifier. [covers=release.evidence.workflow.witnessed.request-export]
- [x] I2 Add `crunch release witness-export` with human and `--json` output,
      verified-bundle + signed-release-attestation preflight, public-only copy
      semantics, and release-identifier mismatch rejection.
      [covers=release.evidence.workflow.witnessed.request-export]

## Phase 2: Import returned witness sidecars safely

- [x] I3 Add a pure witness-import validation core that classifies candidate
      imports as exact duplicates, missing signatures, release-digest
      mismatches, or conflicting duplicates before any filesystem writes.
      [covers=release.verification.tech.witness.import.cli]
- [x] I4 Add `crunch attest witness-import <verification-dir> <source>` with
      directory and single-file input support, idempotent exact-duplicate
      handling, and fail-closed copy semantics into `<verification-dir>/witnesses/`.
      [covers=release.verification.tech.witness.import.cli]

## Phase 3: Prove and document the cross-machine workflow

- [x] I5 Add positive and negative `tests/release_cli.rs` coverage for
      `release witness-export` and `attest witness-import`, including release-id
      mismatch, missing signature, wrong release digest, and conflicting
      duplicate witness identity cases.
      [covers=release.evidence.workflow.witnessed.request-export,release.verification.tech.witness.import.cli]
- [x] I6 Add an end-to-end release CLI workflow test that starts from a valid
      release-evidence bundle, runs `release attest`, runs `release
      witness-export`, creates a witness from the exported request directory in
      a separate temp work area and `CRUNCH_CONFIG_DIR`, imports the returned
      witness sidecars into the publisher verification dir, and proves `attest
      release-verify` reports `technical_class=external-witness-match`,
      `policy_status=satisfied`, and `final_class=quorum-satisfied`.
      [covers=release.evidence.workflow.witnessed.request-export,release.verification.tech.witness.import.cli]
- [x] I7 Update `README.md` and `docs/operator-workflows.md` so the
      witnessed-self-hosting docs show the exact publisher -> witness ->
      publisher command chain (`release verify`, `release attest`, `release
      witness-export`, `attest witness-create`, `attest witness-import`,
      `attest release-verify`) and keep the claim bounded to external witness
      agreement under configured policy.
      [covers=release.evidence.workflow.witnessed.crossmachine.docs]

## Validation

- [x] V1 Run `cargo test -p crunch --test release_cli witness_export_ --
      --nocapture` and quote the output proving the request export layout works
      and the release-id mismatch path fails correctly.
      [covers=release.evidence.workflow.witnessed.request-export]
      [evidence=openspec/changes/cross-machine-witness-handoff/evidence/V1-witness-export.md]
  - Evidence summary: `cargo test -p crunch --test release_cli witness_export_
    -- --nocapture` -> `test result: ok. 2 passed; 0 failed; 0 ignored; 0
    measured; 38 filtered out; finished in 0.03s`, covering
    `witness_export_writes_request_directory` and
    `witness_export_rejects_release_id_mismatch`.
- [x] V2 Run `cargo test -p crunch --test release_cli witness_import_ --
      --nocapture` and quote the output proving matching imports succeed while
      missing signatures, wrong release digests, and conflicting duplicates
      fail correctly.
      [covers=release.verification.tech.witness.import.cli]
      [evidence=openspec/changes/cross-machine-witness-handoff/evidence/V2-witness-import.md]
  - Evidence summary: `cargo test -p crunch --test release_cli witness_import_
    -- --nocapture` -> `test result: ok. 4 passed; 0 failed; 0 ignored; 0
    measured; 36 filtered out; finished in 0.03s`, covering
    `witness_import_accepts_directory_source_and_skips_exact_duplicates`,
    `witness_import_rejects_missing_signature_sidecar`,
    `witness_import_rejects_wrong_release_attestation_digest`, and
    `witness_import_rejects_conflicting_duplicate_identity`.
- [x] V3 Run `cargo test -p crunch --test release_cli cross_machine_witness_handoff_ --
      --nocapture`, then rerun `openspec validate cross-machine-witness-handoff`,
      `openspec_gate stage=design change=cross-machine-witness-handoff`, and
      `openspec_gate stage=tasks change=cross-machine-witness-handoff`.
      [covers=release.evidence.workflow.witnessed.request-export,release.verification.tech.witness.import.cli,release.evidence.workflow.witnessed.crossmachine.docs]
      [evidence=openspec/changes/cross-machine-witness-handoff/evidence/V3-cross-machine-workflow.md]
  - Evidence summary: `cargo test -p crunch --test release_cli
    cross_machine_witness_handoff_ -- --nocapture` -> `test result: ok. 1
    passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in
    0.08s`, `openspec validate cross-machine-witness-handoff` -> `Change
    'cross-machine-witness-handoff' is valid`,
    `openspec_gate stage=design change=cross-machine-witness-handoff` ->
    `VERDICT: PASS`, and `openspec_gate stage=tasks
    change=cross-machine-witness-handoff` -> `VERDICT: PASS`.
