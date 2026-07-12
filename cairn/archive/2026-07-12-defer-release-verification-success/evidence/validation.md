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

A broader substring run (pueue task 1045) selected the unrelated attestation test `attest_witness_show_and_release_verify_report_quorum_satisfied`; 51 release-command tests passed and that attestation test initially failed because its manually authored witness fixture retained the constructor's explicit `source_acquisition_mode = "unspecified"`. Integration commit `f1f4e9a7` now marks manually authored fixtures as `manual-operator-supplied`; the full release CLI suite subsequently passed in pueue task 1600.

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

No sync or archive command was run in the implementation worktree.

## Main-branch integration validation

All Cargo commands below used isolated target directory `/tmp/mantle-main-release-target` with `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`.

- Pueue task 1604: `cargo test -p crunch-release-core --lib` — **PASS**, `161 passed; 0 failed`.
- Pueue task 1559: `cargo test -p mantle --bin mantle release_evidence::` and `release_cmd::tests` — **PASS**, respectively `26 passed; 0 failed` and `16 passed; 0 failed`.
- Pueue task 1600: `cargo test -p mantle --test release_cli` — **PASS**, `118 passed; 0 failed`; this includes every final-decision positive/negative fixture and the previously stale witness fixture.
- Pueue task 1591: `cargo test -p mantle --bin mantle release_reproducibility::tests` — **PASS**, `12 passed; 0 failed` after binding the pre-existing determinism-normalization policy before proof promotion.
- Pueue task 1569: strict first-party core clippy (`cargo clippy -p crunch-release-core --lib --no-deps -- -D warnings`) — **PASS**.
- Pueue task 1606: root binary clippy completed at the repository warning baseline with **zero diagnostics in the touched release files**.
- Pueue task 1610: `cargo check -p mantle --bin mantle`, targeted rustfmt check, and `git diff --check` — **PASS**.

## Initial pre-sync Cairn evidence

Pueue task 1649 ran the authoritative Cairn CLI with `--root . --policy cairn-policy/generated/cairn-policy.json`.

### Validation

```text
{
  "change_issues": [],
  "changes": 17,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 42,
  "valid": true
}
```

### Proposal gate

```text
{
  "change": "defer-release-verification-success",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "46ed7afa34910fceb2c0d1dbbbdaab4be4ba454d97d7ae7d90dede06016491fc",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "ed81ae88929399aeca328e5189798c52b1395cb268ffe05f6cb129b60e4f2c6a",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

### Design gate

```text
{
  "change": "defer-release-verification-success",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "24f3208ecba16e33f400f3a9662bb3077c242486a8df16a5c775cdd6e1aa070f",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "b6471e287a55bab3a502d761d522ef7cb7fcc4530e9ffa760d45b08cea03841e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

### Tasks gate

```text
{
  "change": "defer-release-verification-success",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5eee9ee04a0006dfd9c7bb0901eef6e04ab8a86cdab54a42daea913c77c71995",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "7175c05ef6a0bce394ea2423d8ad8d13213de238feb9935638af395b4b277774",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

That first executed sync was a no-op because both package specs used standalone `## Requirements` headings. Before archive, they were corrected to Cairn `## ADDED Requirements` deltas and every gate was rerun.

## Final pre-sync Cairn evidence after delta repair

Pueue task 1665 reran the authoritative validation and all three gates.

### Validation

```text
{
  "change_issues": [],
  "changes": 17,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 42,
  "valid": true
}
```

### Proposal gate

```text
{
  "change": "defer-release-verification-success",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "83f1365a5b1734165d96643285047009badf84d31e295acc3e89801450f71128",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "de16bfc60b649c6bded3db8704e159e4a06c9c222b437308b5b5baf6c3b21c02",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

### Design gate

```text
{
  "change": "defer-release-verification-success",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a7976eb2a55c410febad017229f5a0eb2e0b90fb2125aed711496ddb07e4cb09",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "b09841c993abe96e5b1090e5eb950675f87c207abed2358537a53bd4ac2a7eaa",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

### Tasks gate

```text
{
  "change": "defer-release-verification-success",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "dc90a4c3dc56790f1990b01fc48eef61553f832aaf1cf69a94978daf4ed9e793",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "b38902e720f398d5fd286a9218a8e8ae95ad29a11b4d561f7ca4d49e44749928",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Executed sync evidence

After the delta-marker repair, the executed sync changed both accepted specs:

```text
change: defer-release-verification-success
operator-diagnostics before: a07752e9ec1d11a440ca49a78dbd4cd4b3326d27ec3caf152c6bbd9d48cbe748
operator-diagnostics after: c34a0d84a890c7ff3c657b852e8e86d347f2ef3460e53f446280f45ddcc857ec
release-provenance before: b6061a6b961aea5a40f3ca0671cc6df19b81c596b86fb702063d889a854b9d5b
release-provenance after: ed9e12750461ad32938a5f34e0f7ea891dbe4321d010620d0050596b429f8cd8
plan_hash: 73888f6b37f9453b86d0007805211def0f034078aae675d2fbf9590051436314
receipt_hash: f7d0238ac22bfe9b2df083bdef7c33e26e3bd4e4fb579127dca47e5cbbd2acc2
mutated: true
blocked: false
```

Pueue task 1677 validated the accepted specs after sync. Exact output:

```text
{
  "change_issues": [],
  "changes": 17,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 42,
  "valid": true
}
```

## Archive evidence

```text
change: defer-release-verification-success
archive path: ./cairn/archive/2026-07-12-defer-release-verification-success
input_hash: a14a86e57e46c9562938d311d6fb0fcd5217f2d93a2711a6f3071770d38f2c7f
plan_hash: 4311c2f095e16607b81bb385308a9b89edaaf0986813da54f75b0df032f2d26a
receipt_hash: ba8fe918c4573fc97b06d7b2489e4b42ec2f1a35aeff47c7e1d1b4c10d10c033
mutated: true
blocked: false
```

Pueue task 1682 ran the authoritative validation command after archive. Exact output:

```text
{
  "change_issues": [],
  "changes": 16,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 40,
  "valid": true
}
```
