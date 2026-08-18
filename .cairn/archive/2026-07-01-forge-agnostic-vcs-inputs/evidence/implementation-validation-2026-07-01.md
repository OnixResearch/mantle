# Implementation Validation — Forge-agnostic VCS inputs

Date: 2026-07-01

## cargo checks

- `cargo test -p crunch-project-core`: task 1252, 155 passed.
- `cargo test -p crunch-project`: task 1272, integration tests passed.
- `cargo test -p crunch-project refresh_adapter::tests`: task 1254, 4 passed.
- `cargo test -p mantle --bin crunch project_resolve::tests`: task 1284, 11 passed.
- `cargo fmt -p crunch-project-core -p crunch-project -p mantle --check && git diff --check`: task 1270 passed.

## cairn validate

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 13,
  "valid": true
}

```

## cairn gate proposal

```text
{
  "change": "forge-agnostic-vcs-inputs",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "065026c3f73023051a6f8e1620fc0c916a2e4e348c95466b2380e42e11c77542",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cb9c0a38ff3cd645895f42b93c2d56a5a59d5b93a60723e612b675ff439d814e",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate design

```text
{
  "change": "forge-agnostic-vcs-inputs",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f0f696d3fdfb033e41189bcdf7d53a9705f6ae95b0ee3be93c8e3edf642fda28",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e76b9966c5da3641b2c4e8d54e0493283c78d50dfa84ef25b845dd8ddc7dc0fd",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate tasks

```text
{
  "change": "forge-agnostic-vcs-inputs",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e9088f39ec854bb4fdbab38bdbc64e508bf70c57669edaac148153526dcf843d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "023ba6629b8848aca550a0d14d45ec6239a9a604204495c840bf6d606b901a0d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```
