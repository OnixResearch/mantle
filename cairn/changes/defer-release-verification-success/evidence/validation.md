# Defer release verification success — validation evidence

Date: 2026-07-12

## Baseline

Before implementation, the focused `release_cli` required-reproducibility rejection fixture passed (1 test), and Cairn validation plus proposal/design/tasks gates passed for `defer-release-verification-success`.

During the later live-consumer audit, the pre-edit `scripts/check-real-release-determinism-receipt.rs --self-test` passed, while the pre-edit `scripts/summarize-real-release-determinism.rs --self-test` failed with `missing string field effect_policy_version` (pueue tasks 857 and 860). The summary fixture was stale before this change touched it; the v2 migration updates it to the current deterministic-proof and release-verification contracts.

## Implementation evidence

- `crates/crunch-release-core/src/verification_decision.rs` is the pure terminal-decision core. One contributor macro generates the closed contributor enum, stable order, fact/requirement records, and exhaustive getters. Adding a contributor therefore extends the ordered list and forces every record literal to map its fact and requirement.
- The core distinguishes mandatory/required/advisory/not-selected policy from satisfied/absent/rejected/not-evaluated facts, bounds diagnostics, fails closed on overflow, and returns one accepted or policy-rejected decision.
- `src/release_cmd.rs` is the imperative shell. It loads and normalizes all safely evaluable facts, aggregates once, renders the immutable completed decision, emits human success only for acceptance, emits one JSON v2 payload, and selects process status from final validity.
- `tests/release_cli.rs` covers accepted human/JSON output and independent reproducibility, deterministic-proof, provider-proof, stack-provenance, external-role, and StageX rejection paths. Rejected JSON assertions also prove top-level diagnostic order equals the flattened blocking-check order.
- `scripts/check-real-release-determinism-receipt.rs` and `scripts/summarize-real-release-determinism.rs` now consume/produce `mantle-release-verify-v2` fixtures and require accepted final validity instead of treating payload presence as success.
- `README.md` and `docs/operator-workflows.md` document the terminal verdict and v1-to-v2 migration rule.

## Focused validation

- Pueue task 1086: `cargo test -p crunch-release-core --lib --no-fail-fast` — **PASS**, `150 passed; 0 failed`.
- Pueue task 1089: `cargo check -p crunch-release-core --target wasm32-unknown-unknown` — **PASS**.
- Pueue task 1094: `cargo test -p mantle --bin crunch release_verify_renderers_preserve_completed_decision_and_terminal_verdict --no-fail-fast` — **PASS**, `1 passed; 0 failed`.
- Pueue task 1058: `cargo test -p mantle --test release_cli release_verify_ --no-fail-fast -- --test-threads=1 --skip attest_witness_show_and_release_verify_report_quorum_satisfied` — **PASS**, `51 passed; 0 failed`.
- Pueue task 1059: `cargo test -p mantle --test release_cli release_create_and_verify_external_evidence_sidecar -- --exact --test-threads=1` — **PASS**, `1 passed; 0 failed`.
- Pueue task 1181: `cargo test -p mantle --test release_cli release_verify_json_rejects_external_role_count_above_fixed_limit -- --exact --test-threads=1` — **PASS**, proving the named role bound rejects overflow through the public JSON contract.
- Pueue task 1201: receipt checker and summarizer `--self-test` commands — **PASS** for both, including negative rejection of false final validity and reordered checks.
- Pueue task 1099: `rustfmt --check` over all touched Rust files plus `git diff --check` — **PASS**.

A broader substring run (pueue task 1045) selected the unrelated attestation test `attest_witness_show_and_release_verify_report_quorum_satisfied`; 51 release-command tests passed and that attestation test failed because it observed `source_acquisition_mode = "unspecified"` instead of its expected `"manual-operator-supplied"`. The requested release-command slice is proven by tasks 1058 and 1059; this unrelated attestation assertion remains a non-claim and was not changed here.

## Cairn validation and gates

Canonical tool provider: `path:/home/brittonr/git/OnixResearch/cairn#cairn`.

- Pueue task 1114: `cairn validate --root /tmp/mantle-defer` — **PASS**, `valid: true`, 43 specs and 18 changes validated with no issues.
- Pueue task 1120: proposal gate — **PASS**, receipt `02d04945a661a3a2705ba9cb96f1e12af8414e586ff726192ff68635f93b4252`.
- Pueue task 1124: design gate — **PASS**, receipt `abbcc8578690513ccbd52b50a0f08d88d8376e19c5b4fc7c5d81e8efa7cde34b`.
- Pueue task 1125: tasks gate — **PASS**, receipt `58e73d31f6e317f3eaba68b6361dcbc05054721294a5d4096a322ad653fc9167`.

After every task was checked and linked to this evidence file, the completed packet was validated again:

- Pueue task 1153: validation — **PASS**, `valid: true`, no issues.
- Pueue task 1154: proposal gate — **PASS**, receipt `186583542619fc1bca0590f0db185031cf3403e9f506b942e279455e5d587ac9`.
- Pueue task 1155: design gate — **PASS**, receipt `382775b91259b40494975b81aea6216dc7487010ee3abd0438fd1e8fa47fbe27`.
- Pueue task 1160: tasks gate — **PASS**, receipt `a6421407b30a56ad9d9b2b2ec4541b1e70ca69dfeae93f1ed78b69b484ff4f5e`.

No sync or archive command was run, by request.
