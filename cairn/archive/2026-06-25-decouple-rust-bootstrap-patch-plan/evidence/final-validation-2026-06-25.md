# Final validation — decouple Rust bootstrap patch plan — 2026-06-25

Task-ID: V2
Covers: rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary, rust_package_planning.source_built_toolchain_closure.provider_contract_independence

These commands were run after V3 provider rerun evidence was recorded and after
all tasks were checked complete.

## git diff --check

```text
command: git diff --check
```

## Cairn validate

```json
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

## Proposal gate

```json
command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal decouple-rust-bootstrap-patch-plan --root .
{
  "change": "decouple-rust-bootstrap-patch-plan",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "edf4ff66add95b2e756dee5711a55fa39ebf83d8902a59e913b891089732f7d9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6f85f233b2cfcf492e692f70457a91fea5dfa3f54124870c40c965550b7cf491",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Design gate

```json
command: nix run path:/home/brittonr/git/cairn#cairn -- gate design decouple-rust-bootstrap-patch-plan --root .
{
  "change": "decouple-rust-bootstrap-patch-plan",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "019b396d15c8d7670fd3663b53186edb6ca5081570ff09dae15a6c756aa1a0cb",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4b0f864b25dc63a1488e062042d22828ebbd00f6ac18b7b86d161570a42d6641",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Tasks gate

```json
command: nix run path:/home/brittonr/git/cairn#cairn -- gate tasks decouple-rust-bootstrap-patch-plan --root .
{
  "change": "decouple-rust-bootstrap-patch-plan",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "62397685a2a0bd92fb5955d52caf257aad436dfa5807d582f0e73ef6db47913c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "99021b6ebcf0f8249027cad0cc82156dfe8edea5e88f004844dcba9821dc22df",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
