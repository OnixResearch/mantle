# Shared action-result validation

Task-ID: V1 V2 V3 V4 V5
Covers: build_correctness.shared_action_result_records, build_correctness.shared_action_result_admission, cache_substitution.shared_action_result_discovery

All Cargo commands used the isolated target and temp root requested for this change:

```text
CARGO_TARGET_DIR=/home/brittonr/.cache/mantle-drain-targets/shared-action-results
TMPDIR=/home/brittonr/.cache/mantle-drain-targets/shared-action-results/tmp
```

No validation used the shared `~/.cargo-target`.

## Passing checks

- `nix develop -c cargo test -p crunch-action-result-core`
  - unit result: `6 passed; 0 failed`
  - ADR 0024 architecture guard: `1 passed; 0 failed`
- `nix develop -c cargo test -p crunch-store --lib`
  - result: `213 passed; 0 failed`
  - includes 14 focused action-result tests and 9 GC tests.
- `nix develop -c cargo test -p crunch-build --lib`
  - result: `563 passed; 0 failed`
  - includes `shared_action_result_reuses_ca_output_without_ca_mapping_or_execution`, `failed_build_never_publishes_an_action_result`, and `fresh_client_fetches_http_action_result_and_objects_without_execution`.
- `nix develop -c cargo test -p crunch-pipeline --lib`
  - result: `18 passed; 0 failed`.
- `nix develop -c cargo test -p mantle --bin mantle build_correctness::tests::`
  - result: `12 passed; 0 failed`.
- `nix develop -c cargo test -p mantle --bin mantle action_result`
  - result: `2 passed; 0 failed` for bounded human plan/build rendering.
- Focused build-report and Rust-producer machine-contract tests passed.
- `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs`
  - `machine schema contract check: PASS (16 contracted, 45 classified)`.
- Typed Nickel policy positive export matched the checked-in generated JSON byte-for-byte.
- The negative Nickel fixture failed as required with `contract broken by the value of http_timeout_ms` at `http_timeout_ms = 0`.
- Leaf-file formatting check passed for every touched Rust file:
  - `rustfmt --check --edition 2024 --config skip_children=true <touched-files>`.
- Strict Clippy passed for `crunch-action-result-core` with `-D warnings`.
- Focused Clippy passed for `crunch-store` and `crunch-build` with `-D warnings` after allowing only pre-existing package findings outside this change. The unwaived findings are recorded below.

## Cairn receipts

Exact-policy validation used `--policy cairn-policy/generated/cairn-policy.json`.

- `cairn validate --root .`: valid, policy `mantle-default`, 8 changes and 35 specs, no issues.
- proposal gate: `PASS`, receipt `8240ed101fee54817b485f60d8a105a593fc895565863c91972b10c10044aa6f`.
- design gate: `PASS`, receipt `590741069b9b82bde5cac9109c3e44c99961c035447faf29d176e6db405f1f2f`.
- tasks gate: `PASS`, receipt `5b0a7ae8be598f62a34d1437e45fe457b597d588a0adea33c87434d1dc858c15`.

## 2026-07-14 closeout evidence

The previously bounded lint and Tracey blockers are resolved:

- The exact action-result core, store, build, and pipeline test commands were
  rerun and exited successfully. Root action-result/report tests are included in
  serial root binary suites that each passed `1487 passed; 0 failed`.
- The typed Nickel positive export matched
  `config/action-result-policy/generated/action-result-policy.json`; the
  zero-timeout negative fixture failed on `http_timeout_ms` as required.
- Machine-schema generation, adversarial self-test, and freshness validation
  passed with 16 contracted and 45 classified surfaces.
- Focused formatting, `git diff --check`, and
  `CARGO_INCREMENTAL=0 nix develop -c ./scripts/check-first-party-clippy.sh`
  passed.
- Cairn validation passed with 12 active changes, 39 specs, and no issues.
- Tracey `mantle-default` passed 134/134 with receipt
  `3c118cff47cda8a584d85d371958b4627fe6b5206c0b51170676f5b98195d9b1`.
  `remote-build-drain` passed 12/12 with no missing or dangling refs and receipt
  `0b39082b9305fcd79b9fb25a8a42b2bda5c97e15d35f001dda2edf0455fa5686`.
- Final post-checkoff gate receipts: proposal
  `ad8b88b1c492942d0bf1e74fbceb8b8f6a2e7c0e62c172cdcdfcf60074b83d15`,
  design `7ac2cd1b3caae96ba6d1c346a0acf7f3bdc9041200afe53bcc4601294702751c`,
  tasks `262df39944a7e195646ed6ef917f28391cc428cb88fd755e9d9d4f27a9dd566e`.

The broad all-target test probe also reached unrelated bwrap-backed
`attest_cli` builds and reported three empty-stderr build failures. Those tests
are outside this change's focused verification set; required action-result,
remote, Nickel, and serial root suites passed. Sync and archive are not claimed
by this pre-archive record.
