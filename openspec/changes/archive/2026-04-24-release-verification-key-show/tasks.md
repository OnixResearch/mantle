# Tasks: release-verification-key-show

## Phase 1: Export trusted public keys from existing signing keys

- [x] I1 Add a side-effect-free key-loading helper that resolves explicit and
      default signing-key paths but rejects missing keys instead of generating
      a new one. [covers=release.verification.tech.trustedkey.export.cli]
- [x] I2 Add `crunch attest key-show [--signing-key <path>]` with human and
      `--json` output that prints the exact verifier token and source path.
      [covers=release.verification.tech.trustedkey.export.cli]

## Phase 2: Cover the CLI and docs

- [x] I3 Add `tests/release_cli.rs` coverage for explicit-path success,
      default-config success, and missing-key failure for `attest key-show`.
      [covers=release.verification.tech.trustedkey.export.cli]
- [x] I4 Update `README.md` and `docs/operator-workflows.md` so the witnessed
      self-hosting workflow shows how release signers and witness rebuilders use
      `crunch attest key-show` to obtain the exact
      `--trusted-public-key <name:base64>` values passed to
      `crunch attest release-verify`. [covers=release.evidence.workflow.witnessed.keyexchange]

## Validation

- [x] V1 Run `cargo test -p crunch --test release_cli attest_key_show_ --
      --nocapture` and quote the output proving explicit-path success,
      default-config success, and missing-key failure. [covers=release.verification.tech.trustedkey.export.cli] [evidence=openspec/changes/release-verification-key-show/evidence/V1-key-show-cli.md]
  - Evidence summary: `cargo test -p crunch --test release_cli
    attest_key_show_ -- --nocapture` -> `test result: ok. 3 passed; 0 failed;
    0 ignored; 0 measured; 29 filtered out; finished in 0.01s`, covering
    `attest_key_show_explicit_signing_key_prints_trusted_public_key`,
    `attest_key_show_uses_default_config_signing_key`, and
    `attest_key_show_missing_signing_key_fails_without_generation`.
- [x] V2 Run `openspec validate release-verification-key-show`,
      `openspec_gate stage=design change=release-verification-key-show`, and
      `openspec_gate stage=tasks change=release-verification-key-show` after
      the docs update. [covers=release.evidence.workflow.witnessed.keyexchange] [evidence=openspec/changes/release-verification-key-show/evidence/V2-gates.md]
  - Evidence summary: `openspec validate release-verification-key-show` ->
    `Change 'release-verification-key-show' is valid`,
    `openspec_gate stage=design change=release-verification-key-show` ->
    `PASS`, `openspec_gate stage=tasks change=release-verification-key-show`
    -> `PASS`, and `cargo test -p crunch --test release_cli attest_ --
    --nocapture` -> `test result: ok. 18 passed; 0 failed; 0 ignored; 0
    measured; 14 filtered out; finished in 0.06s`.
