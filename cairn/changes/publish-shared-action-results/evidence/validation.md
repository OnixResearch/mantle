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

## Bounded blockers

V5 remains unchecked because the requested repository-wide rails are not all green for pre-existing, out-of-scope reasons:

1. `nix develop -c cargo fmt --check -p crunch-action-result-core -p crunch-store -p crunch-build -p crunch-pipeline -p mantle` reaches unrelated pre-existing formatting drift, including `tests/trust_policy_offline_rail.rs`. The leaf-file check over every file touched by this change passes.
2. Unwaived package-wide `crunch-store` Clippy reaches pre-existing warnings/findings in `archive.rs`, `completeness.rs`, existing `handle.rs` documentation, `export.rs`, and `publisher.rs`.
3. Unwaived package-wide `crunch-build` Clippy reaches ten pre-existing findings in `distributed/remote_telemetry.rs`, `dynamic_plan.rs`, `network_policy.rs`, and `worker.rs`. No finding points at the new action-result adapter or orchestration code; focused runs pass after allowing those exact existing categories.
4. `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .` fails against the existing `mantle-default` profile, which scans the accepted `release-provenance` spec and seven release evidence files rather than this active change. It reports 134 requirements, 56 referenced markers, 17 dangling markers, and 78 missing release-provenance markers. The exact command, policy/receipt hashes, and complete missing/dangling sets are preserved in `tracey-blocker.json`.

These blockers are not action-result implementation failures and were not repaired because doing so would churn unrelated active release/format/lint work. No archive or sync was attempted.
