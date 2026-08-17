# Cairn validation evidence (2026-06-02)

## Commands

```text
COMMAND /home/brittonr/git/mantle $ cairn validate --root .
STATUS 0
```

Output:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

```text
COMMAND /home/brittonr/git/mantle $ cairn gate tasks frontend-artifact-spec-admission --root .
STATUS 0
```

Output:

```json
{
  "change": "frontend-artifact-spec-admission",
  "input_hash": "8ce51d6c6f8b448b4e02e869cee139be617746517f0494a99b7c9a1f81ec2011",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cc9b724e537511e8d7717766a9705730bd916d484220c2307899457b7c1a16aa",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
