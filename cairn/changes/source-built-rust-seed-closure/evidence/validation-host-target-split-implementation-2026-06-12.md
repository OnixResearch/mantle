# Validation for host/target split implementation (2026-06-12)

## validate

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

exit_status: 0
```

## gate-proposal

```text
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
  "input_hash": "eb9e3e73bfd67414f61d2b113481682109ec4b7f366673aa5bba0964b1338551",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "2914e7bb42d2576386c3740f41b18dcad9386dfd13fbb63061352019d023cabc",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## gate-design

```text
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
  "input_hash": "49295c1b1da5372bdba08f9175fdee6cc9ecbdeb88b4df84c2c18925cf85b650",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d75dfa216d6fa99a299bbc3166c63802fe1f7e0a03fe7c8d501e231e57c11144",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## gate-tasks

```text
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
  "input_hash": "6d49ca0454784cec3b33e115b52cf9a0c71d3e33aac5391841875c7642097a13",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "f1aea0ff11bc8e2cbc2a54521a3581c4e462228f82ac0f2087e4373dbc5c771c",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

