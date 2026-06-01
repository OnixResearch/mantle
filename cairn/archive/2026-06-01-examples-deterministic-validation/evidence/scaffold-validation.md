# Scaffold validation

Change: `examples-deterministic-validation`

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

## cairn gate proposal examples-deterministic-validation --root .

```text
{
  "change": "examples-deterministic-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ea258a8a1bf79c0cbe19752d950c8f1927caa671f3c3329a61c9f879508155f7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ae6adae68e885680beb3f313aa20a7110c15b46f67080ced09eef17e34c78597",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate design examples-deterministic-validation --root .

```text
{
  "change": "examples-deterministic-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f65f6b916557569070e61acfeb94eb0f3b4b61a0875e3c1b6408b0a8a2bbd3e5",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7101573d46007912e1ebb0c53124c390d5e66203ce9370d2991dbbad881f6f1c",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate tasks examples-deterministic-validation --root .

```text
{
  "change": "examples-deterministic-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d9cb6210457c8e361432cc0117578f028f861f75c6a7f8a38a52e2e5489387d9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "db6973efbfe519621d2318f5c6fa838ca56e628dc7c6e9afb156fb3097eb7b06",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

