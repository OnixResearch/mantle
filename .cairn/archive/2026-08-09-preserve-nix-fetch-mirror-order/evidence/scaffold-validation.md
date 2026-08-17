# Scaffold validation

The change is a validated lifecycle scaffold. All implementation and verification tasks remain unchecked.

Validation used the Cairn revision pinned by this repository: `fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8`.

## Repository validation

Command:

```text
nix run github:onixresearch/cairn/fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 14,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 48,
  "valid": true
}
```

## Proposal gate

Command:

```text
nix run github:onixresearch/cairn/fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8#cairn -- gate proposal preserve-nix-fetch-mirror-order --root .
```

Output:

```json
{
  "change": "preserve-nix-fetch-mirror-order",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f70e7a038e06c0a8d417addef39a550a3e4fdcf5836c90726a3c1f6d921dd82b",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "31311ed59859b17aa3da30518cfd4053edafa0443ca657459c44956a08495ac4",
  "receipt_hash": "b67acefb80369b9d3ae5909d787405de7297dfedc0f4b816c01eade6839998be",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Design gate

Command:

```text
nix run github:onixresearch/cairn/fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8#cairn -- gate design preserve-nix-fetch-mirror-order --root .
```

Output:

```json
{
  "change": "preserve-nix-fetch-mirror-order",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "65a427585bfcc105e863d9c79bddca0592e35705b6e56e5bae333c4542a10016",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "31311ed59859b17aa3da30518cfd4053edafa0443ca657459c44956a08495ac4",
  "receipt_hash": "e6978aabdf2c3384af3dbbda04e049fef7aa904fb9c6d53cb43f28bc615d0121",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Tasks gate

Command:

```text
nix run github:onixresearch/cairn/fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8#cairn -- gate tasks preserve-nix-fetch-mirror-order --root .
```

Output:

```json
{
  "change": "preserve-nix-fetch-mirror-order",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "74fd17dca1cb92e7bcc81798768c94f9fbacb8c3347497211dee5c5f191def02",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "31311ed59859b17aa3da30518cfd4053edafa0443ca657459c44956a08495ac4",
  "receipt_hash": "f9260ff346409aba9875effdfa10f1480a2b18e49cbac57414f91863d8f249ac",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Fresh local Cairn blocker

The current local Cairn checkout could not validate this repository. It rejected the checked-in generated policy before reading the change:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

This scaffold does not refresh repository policy. That policy migration is separate from the ordered-mirror change.
