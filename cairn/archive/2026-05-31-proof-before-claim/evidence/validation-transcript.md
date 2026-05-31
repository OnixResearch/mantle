# Validation transcript

Task-ID: proof-before-claim-validation
Covers: verification_evidence.proof_before_claim
Date: 2026-05-31T12:23:00Z

## cairn validate --root .

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
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
  "specs_validated": 4,
  "valid": true
}
```

## cairn gate proposal proof-before-claim --root .

Output:

```json
{
  "change": "proof-before-claim",
  "input_hash": "9c8fd4f3610ae0017612da8df9b8609ba713ef8d8a377eb6fdde16efcf19590e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "5960dab4c9c933547a2e3da1ce9fa70a7af30d99e5510d687efd50ef2ca467d5",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## cairn gate design proof-before-claim --root .

Output:

```json
{
  "change": "proof-before-claim",
  "input_hash": "3a89c6e421c4540298fbe415c1cc6c974ef6a74aad164b3211ccb7f779205f89",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e8751d5850481a5db68293d0c340dc1f9389458db081bed62551a0ac2933c0ef",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## cairn gate tasks proof-before-claim --root .

Output:

```json
{
  "change": "proof-before-claim",
  "input_hash": "ba5ad41ccfc045d052216909d7822ad1adc834e3892ac00fef0653ac7b1cb058",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0c0efb9db5a76eaa7fbe06b70d75ab6c4de0eb7d45627ebc917a820a6e60e615",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-archive cairn validate --root .

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```
