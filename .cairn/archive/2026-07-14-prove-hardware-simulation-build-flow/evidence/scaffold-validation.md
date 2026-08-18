# Scaffold validation

These are planning/scaffold gates only. All implementation and verification tasks remain unchecked; this transcript does not prove the hardware flow exists.

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change_issues": [],
  "changes": 11,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 38,
  "valid": true
}

```

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal prove-hardware-simulation-build-flow --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change": "prove-hardware-simulation-build-flow",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "66eb34b41297cb0f7c3c18c9685a176024509f438b0edf1ca67393f0186b1240",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "0da818bed401bc79839abd989d61190f8ae4b00e8aef9bfbbb386612815228e1",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design prove-hardware-simulation-build-flow --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change": "prove-hardware-simulation-build-flow",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "53d14e5d33909c209544f8638b69b66a42989241328abe0b8b009884e916fad2",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "f38533af7f5b7a4bdc7de83cb200d6f32a77a195e421dca0d78362d9c560b18e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks prove-hardware-simulation-build-flow --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change": "prove-hardware-simulation-build-flow",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "06c2e85ba007a666d705a42e4eb7980a7c98cf1a604af887e34e0fe2cd2b827d",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "c9d98d9cf38c99f4068675ca2f8f36e7622912d69b7cfd9b7b33cd472db19bdf",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

