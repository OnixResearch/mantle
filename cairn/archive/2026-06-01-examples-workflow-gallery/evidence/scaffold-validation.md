# Scaffold validation

Change: `examples-workflow-gallery`

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

## cairn gate proposal examples-workflow-gallery --root .

```text
{
  "change": "examples-workflow-gallery",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "523b8b1049ebf51505985c9f95a3d75d98a9a7cd916a80ffc5235c6674c12b91",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "709718538ad352a47548e9992615a634ea50fd5d1bd55d3dca9dc846e0e8c10a",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate design examples-workflow-gallery --root .

```text
{
  "change": "examples-workflow-gallery",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "9c8d91cb1f38e1c84adade7953e8d3b15889fe5d9a11478ae8961617d657fa39",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "fe02a849790cd359afe5d4b791aa207233506cea4977a0af6d674e1c50783857",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate tasks examples-workflow-gallery --root .

```text
{
  "change": "examples-workflow-gallery",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "20dd38b99c7cb5626bc743be536564047ae8ec87c94614426b9340e15b7601b9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "08f113d1db7806cc7eec3661cf7295ff189361ff3340475c1ca4740a57a12611",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

