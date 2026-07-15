# Scaffold validation

These are planning/scaffold gates only. All implementation and verification tasks remain unchecked; this transcript does not prove an external dispatcher adapter or production cluster support exists.

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

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal adapt-external-batch-dispatchers --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change": "adapt-external-batch-dispatchers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3c8b442488b1bd6ef9ea5588e1ca06e6e2212fdb16e1b76170c15099f8599556",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "de5a44e24231af9b252feb668aeaa44d1180f2f1860ceab5da39b560e9aa38b8",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design adapt-external-batch-dispatchers --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change": "adapt-external-batch-dispatchers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1517a1b295676e95a31848a4b3b74b8c50186c794607940a37f56a7dba3d6085",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "2a85a38122ab1729bfd501c149108536c9e5778d865e8b6ddf93569fc28255c6",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks adapt-external-batch-dispatchers --root /home/brittonr/git/OnixResearch/mantle`

```text
{
  "change": "adapt-external-batch-dispatchers",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "50b6fc810ebc935f67a9fbe455aaacba870bf6384b3da0f961bc2afdeab0a9b8",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "613401f61779e15c4c13bada2bb20f9d8a518bbfb956b377e263c13506f241a9",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

