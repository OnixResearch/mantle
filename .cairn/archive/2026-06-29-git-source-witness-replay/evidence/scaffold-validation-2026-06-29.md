# Scaffold validation for Git source witness replay

Date: 2026-06-29

This is a planning/scaffold-only Cairn change. It defines the Git-origin source reconstruction requirement, design, and implementation tasks. It does not claim implementation support yet; all implementation tasks remain unchecked.

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

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal git-source-witness-replay --root .

~~~text
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
  "input_hash": "1c89a15e45b6d69715b4ae4c8d0137979cf08bb86e8a52dfa7fd9aff583e39ff",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "06e12d58c52d73907f14a8ce86c2e5f32ee99bc6aa4df7a3479ce744710751a4",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
~~~

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate design git-source-witness-replay --root .

~~~text
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
  "input_hash": "da90b1545d2bf7dc637e0857a3827222a22da106459c7b2a8e24262e1d5e6750",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "8ac57831a92bc36246fa8e85c805f6d9be03dc141bb15d79eb558aeb53f90847",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
~~~

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks git-source-witness-replay --root .

~~~text
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
  "input_hash": "57bb85bdc1d67d08c0ad16bc630ff7db94606adac2e4df72bdfba7e84fbc54dd",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c6b9761ce0c17bf876d14e5638bb43f51bcc28db49a77bf890dd17aa66c5b5be",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
~~~

Exit status: 0
