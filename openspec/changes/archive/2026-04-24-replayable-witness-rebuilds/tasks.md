# Tasks: replayable witness rebuilds

## Phase 1: Derive a deterministic witness rebuild plan

- [x] I1 Add a pure request-to-plan core that reads the exported request
      layout, release attestation, and release-evidence manifest into one owned
      `WitnessRebuildPlan`, including supported-workflow checks, expected
      output names, expected BLAKE3 digests, and deterministic audit/output
      paths.
      [covers=release.verification.tech.witness.rebuild.cli]
- [x] I2 Extend the release-side shell to expose `crunch release
      witness-rebuild <request-dir>` with the witness metadata already required
      by `crunch attest witness-create`, fail-closed request validation, and
      reuse of the existing witness-attestation machinery.
      [covers=release.verification.tech.witness.rebuild.cli]

## Phase 2: Make the witness-side workflow runnable and auditable

- [x] I3 Add the checked-in `./scripts/rebuild-witness-request.sh` helper with
      explicit acceptance criteria: discover `bwrap`, resolve a real static
      busybox-backed `SNIX_BUILD_SANDBOX_SHELL`, absolutize that shell path,
      treat the exported request directory as immutable, derive the scratch root
      from `CRUNCH_WITNESS_SCRATCH_DIR` or the sibling path
      `<request-dir>.work/`, rewrite `TMPDIR` and `CARGO_TARGET_DIR` under that
      root, invoke `crunch release witness-rebuild`, and keep any `--check`
      mode explicitly preflight-only.
      [covers=release.evidence.workflow.witnessed.selfhosting,release.verification.tech.witness.rebuild.cli]
- [x] I4 Persist witness-side rebuild evidence alongside the generated witness
      sidecars, including the replayed workflow identity, selected scratch
      paths, command transcript, and rebuilt output digests, and ensure audit
      timestamps bracket the rebuild subprocess instead of reusing a known
      post-hoc duration bug unchanged.
      [covers=release.verification.tech.witness.rebuild.cli,release.evidence.workflow.witnessed.selfhosting]

## Phase 3: Prove and document the full publisher -> witness-runner -> publisher rail

- [x] I5 Add positive and negative `tests/release_cli.rs` coverage for
      `crunch release witness-rebuild`, including unsupported request
      layout/workflow rejection, rebuilt-output mismatch rejection, helper
      `--check` staying preflight-only, and audit `meta.json` timestamp
      bracketing for a non-trivial rebuild.
      [covers=release.verification.tech.witness.rebuild.cli]
- [x] I6 Add an end-to-end release CLI workflow test that starts from a valid
      release-evidence bundle, runs `release attest`, runs `attest
      policy-init --profile single-witness`, runs `release witness-export`,
      runs `release witness-rebuild` in a separate witness work area, imports
      the returned sidecars with `attest witness-import`, and proves `attest
      release-verify` reports `technical_class=external-witness-match`,
      `policy_status=satisfied`, and `final_class=quorum-satisfied`.
      [covers=release.evidence.workflow.witnessed.selfhosting,release.evidence.workflow.witnessed.crossmachine.docs,release.verification.tech.witness.rebuild.cli]
- [x] I7 Update `README.md`, `docs/operator-workflows.md`, and any witness-side
      helper docs so the witnessed-self-hosting workflow shows the exact
      publisher -> witness-runner -> publisher chain and keeps the claim bounded
      to external witness agreement under configured policy.
      [covers=release.evidence.workflow.witnessed.crossmachine.docs]

## Validation

- [x] V1 Run `cargo test -p crunch --test release_cli witness_rebuild_ --
      --nocapture` and capture the output proving the happy path succeeds,
      helper `--check` stays preflight-only, audit timestamps bracket the
      rebuild, and unsupported layout/workflow plus output-mismatch negatives
      fail correctly.
      [covers=release.verification.tech.witness.rebuild.cli]
      [evidence=openspec/changes/archive/2026-04-24-replayable-witness-rebuilds/evidence/V1-witness-rebuild-cli.md]
      Evidence summary: `bash -n scripts/rebuild-witness-request.sh` passed and
      `cargo test -p crunch --test release_cli witness_rebuild_ -- --nocapture`
      passed 6 tests covering helper preflight, happy path, timestamp audit,
      unsupported workflow, output-name drift, and digest mismatch.
- [x] V2 Run `cargo test -p crunch --test release_cli
      witnessed_self_hosting_rebuild_ -- --nocapture` and capture the output
      proving the publisher -> witness-runner -> publisher workflow reaches
      `external-witness-match` and `quorum-satisfied`.
      [covers=release.evidence.workflow.witnessed.selfhosting,release.evidence.workflow.witnessed.crossmachine.docs,release.verification.tech.witness.rebuild.cli]
      [evidence=openspec/changes/archive/2026-04-24-replayable-witness-rebuilds/evidence/V2-end-to-end-witness-rebuild.md]
      Evidence summary: `cargo test -p crunch --test release_cli
      witnessed_self_hosting_rebuild_ -- --nocapture` passed 1 end-to-end test
      and asserted `technical_class=external-witness-match`,
      `policy_status=satisfied`, and `final_class=quorum-satisfied`.
- [x] V3 Run `openspec validate replayable-witness-rebuilds` and
      `openspec_gate stage=design change=replayable-witness-rebuilds` after the
      implementation lands.
      [covers=release.evidence.workflow.witnessed.selfhosting,release.evidence.workflow.witnessed.crossmachine.docs,release.verification.tech.witness.rebuild.cli]
      [evidence=openspec/changes/archive/2026-04-24-replayable-witness-rebuilds/evidence/V3-openspec-gates.md]
      Evidence summary: `openspec validate replayable-witness-rebuilds`
      succeeded and `openspec_gate stage=design
      change=replayable-witness-rebuilds` returned `VERDICT: PASS`.
