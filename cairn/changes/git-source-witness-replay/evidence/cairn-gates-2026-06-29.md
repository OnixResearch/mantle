# Cairn validation and gates for Git source witness replay

Date: 2026-06-29
Change: `git-source-witness-replay`

Note: The change remains active because the Aspen1 cross-machine replay task is blocked by missing `aspen1` host resolution in this environment. These gates validate the current active change artifacts, completed local tasks, and recorded blocker note.

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "change": "git-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "03c5eb08bc0f90503c3dd9c71335abcd0d6da8f63df5e4f7430b475c7245176d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6db3dfc2afcef277fdf1648c1674af4b7ff7f2a93f4a7f2bd83d267bdeb98035",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate design git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "change": "git-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "2c1361aeeb4464251a51e68d1746617e49be427566f6b3b6bceae645cf17eefb",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b2bc3a16154c7c29a1e76b18063b312d16724fe586e8435050a42610c2e7cbc2",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "change": "git-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b883f0aac1b96f9bd57d614738470dff4a55b392d04b9450700aa5f214f8e663",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "913aab9a28883ad14b36899742471a04c4cf9a39bc2f4344e6a2b5c65d66289c",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Exit status: 0
