# Cairn validation for independent source witness replay

Date: 2026-06-29

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root .

~~~text
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
~~~

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal independent-source-witness-replay --root .

~~~text
{
  "change": "independent-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b62b92b37d6716841398f0cb951d994d6ce84d4c3b94a9cc4250400e7bb947f8",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1f9db1a4db31f663ef30df1759847a29c594af1f840a8d86580e9e1dcd192e45",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
~~~

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate design independent-source-witness-replay --root .

~~~text
{
  "change": "independent-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f063b7aaf4ce8dcbfe84053b93c4bd5a2e4ac649907a1184f8f9fb9504761501",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1ba7f50d577a3667a552fc77fcb06a8385e8ca7af02f4deeead40b6c1ecba630",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
~~~

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks independent-source-witness-replay --root .

~~~text
{
  "change": "independent-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c5b4cb96ccd0a436b295fa7d26fd07ee27d9f1dc0c0e483bb990706a73b4722d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b5c7a9bb33c9322408892c26862ceefc88d0b1faecddf4c89bf4bb3d61ec0c3e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
~~~

Exit status: 0
