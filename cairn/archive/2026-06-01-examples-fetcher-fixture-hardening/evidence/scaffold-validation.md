# Scaffold validation

Change: `examples-fetcher-fixture-hardening`

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

## cairn gate proposal examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1f84f07fce42e4c923e2bbaf290e5027f7c16dde046b22a478eb4509f02fc0e2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9da9b2a5561446922e5213501e1f6c3f118dae28f75e8ee559bf68f67ea16794",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate design examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "653f1b23f4565489161c345c6326e642122ddd0eebe0db785e3cfa19cc089a68",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0f59c60847a216d7a7921c18e4b537ab7620ef850580aeb356dc723242f9493d",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate tasks examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "00b31de804708d49c0a9be275df6c04221571017ba56d785889b53c9ea0c1b03",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "815150c290f23de67f6262011826de6edfa444f4b8625a7d060bb9f2ff7484a5",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

