# Host/target split Cairn artifact validation

Task-ID: host-target-split-cairn-2026-06-12
Covers: rust_package_planning.source_built_rust_seed_closure

## Scope

Records the Cairn-only proposal/design/spec/tasks update for the next source-built Rust route slice: separate compiler-host artifacts from the musl target sysroot before rerunning the real route.

## Transcript

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6dcada096988a849caf9b538604be4532f979aeb976b8cc64958ed66288eaa8e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7397d761cf5ed5b7fbb55b760f92fc93a84559fad55dd415d7ffe3fd516e8671",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "7dcea871da69243dd740bbb3e731d0caa7f89e269882c69642050b2d8509ae82",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "a87a281510d08af91eee5a960f4a29055ceebe3537bad92a1bf84b0bf535d8d0",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "0d4c2ac06ec6cecd7be83c08fac6458a1a79f5257a8ed0fc0e51fa86ca97ce71",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "49ca237b88ee71efdcc535b5b936d113207b6ed9d00f48124834ab752761669b",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-task-reference validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "cbfbfdbaf096a179572640182532feca203483d6684ced64eb71abd2b9aa1646",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cff83ba074be74e07b7b7119a0d833f12bdee64b4109ae92439d774b801d3bb0",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
