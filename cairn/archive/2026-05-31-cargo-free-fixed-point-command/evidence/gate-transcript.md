# Cairn validation and gate transcript

Command run from repo root:

```sh
CAIRN=$(ls /nix/store/*-cairn-0.1.0/bin/cairn 2>/dev/null | sort | tail -1)
"$CAIRN" validate --root .
"$CAIRN" gate proposal cargo-free-fixed-point-command --root .
"$CAIRN" gate design cargo-free-fixed-point-command --root .
"$CAIRN" gate tasks cargo-free-fixed-point-command --root .
```

Observed output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 2,
  "valid": true
}
{
  "change": "cargo-free-fixed-point-command",
  "input_hash": "50c7f78fd6e7072622f4711ed6ae3b62f2eadcc5c11f5b2f29d45c1c1714b045",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4e978d59d5a9bdd840bf06a1a995bdf4086194322a428486ba0dc1907a7075ab",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "cargo-free-fixed-point-command",
  "input_hash": "f8fc87ffe3ac26b394844caeca5629217d0264571022147fb4d038757c1caffe",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "094151a35970f96faa2589f28fe58e57b868c261a9e8c1aef2b88fbc68a7f8fd",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "cargo-free-fixed-point-command",
  "input_hash": "c42266aa4928d74eebf1e4f271f7d18cc1bd4c4aa8828185029a39fd112d884f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7fcca7780c54fb4d0136bb115b505c21bbbe8a49f14d0c383d0bc0da96d7f2f1",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
