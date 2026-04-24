# Tasks: witnessed self-hosting workflow

## Phase 1: Scaffold verification policy from the CLI

- [x] I1 Add a pure policy-profile core that maps `self-proof-only` and
      `single-witness` plus explicit signer/identity inputs into deterministic
      `ReleasePolicy` / revocation-file payloads without doing filesystem I/O.
      [covers=release.verification.social.policy.scaffold.cli]
- [x] I2 Add `crunch attest policy-init <verification-dir>` with human and
      `--json` output, explicit `--profile`, `--trusted-release-signer`,
      `--trusted-witness-identity`, and no-clobber-by-default `--force`
      semantics. [covers=release.verification.social.policy.scaffold.cli]

## Phase 2: Prove the operator workflow

- [x] I3 Add positive and negative `tests/release_cli.rs` coverage for
      `attest policy-init`: prove both supported profiles serialize the
      expected files and prove existing `policy.json` / `revocations.json`
      reject overwrite without `--force`. [covers=release.verification.social.policy.scaffold.cli]
- [x] I4 Add an end-to-end release CLI workflow test that starts from a valid
      release-evidence bundle, runs `release attest`, runs `attest
      policy-init --profile single-witness`, publishes a matching witness via
      `attest witness-create`, then proves `attest release-verify` reports
      `technical_class=external-witness-match`, `policy_status=satisfied`, and
      `final_class=quorum-satisfied`. [covers=release.evidence.workflow.witnessed.selfhosting,release.verification.social.policy.scaffold.cli]

## Phase 3: Document the bounded witnessed-self-hosting rail

- [x] I5 Update `README.md` and `docs/operator-workflows.md` so the witnessed
      self-hosting workflow shows the exact command chain (`release verify`,
      `release attest`, `attest policy-init`, `attest witness-create`,
      `attest release-verify`) and keeps the claim bounded to external witness
      agreement under configured policy rather than full-source bootstrap or
      global reproducibility. [covers=release.evidence.workflow.witnessed.selfhosting]

## Validation

- [x] V1 Run `cargo test -p crunch --test release_cli attest_policy_init_ --
      --nocapture` and quote the output proving both policy profiles succeed
      and the no-clobber negative case fails correctly. [covers=release.verification.social.policy.scaffold.cli] [evidence=openspec/changes/witnessed-self-hosting-workflow/evidence/V1-policy-init-cli.md]
  - Evidence summary: `cargo test -p crunch --test release_cli
    attest_policy_init_ -- --nocapture` -> `test result: ok. 3 passed; 0
    failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.04s`,
    covering `attest_policy_init_self_proof_only_writes_policy_files`,
    `attest_policy_init_single_witness_writes_policy_files`, and
    `attest_policy_init_rejects_existing_policy_without_force`.
- [x] V2 Run `cargo test -p crunch --test release_cli witnessed_self_hosting_
      -- --nocapture`, then rerun `openspec validate
      witnessed-self-hosting-workflow`, `openspec_gate stage=design
      change=witnessed-self-hosting-workflow`, and `openspec_gate stage=tasks
      change=witnessed-self-hosting-workflow`. [covers=release.evidence.workflow.witnessed.selfhosting,release.verification.social.policy.scaffold.cli] [evidence=openspec/changes/witnessed-self-hosting-workflow/evidence/V2-witnessed-workflow-and-gates.md]
  - Evidence summary: `cargo test -p crunch --test release_cli
    witnessed_self_hosting_ -- --nocapture` -> `test result: ok. 1 passed; 0
    failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.19s`,
    `openspec validate witnessed-self-hosting-workflow` -> `Change
    'witnessed-self-hosting-workflow' is valid`,
    `openspec_gate stage=design change=witnessed-self-hosting-workflow` ->
    `PASS`, and `openspec_gate stage=tasks
    change=witnessed-self-hosting-workflow` -> `PASS`.
